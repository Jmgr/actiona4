use std::{collections::HashSet, mem};

use super::{
    ActionTree, BranchKind, ClipboardNode, ClipboardTree, DropMode, Error, Metadata, Node, NodeId,
    NodePayload,
};
use crate::{
    actions::{ActionDefinition, ActionInstance, Branching, WithCommonParameters, WithDefinition},
    parameters::duration::DurationValue,
};

impl ActionTree {
    fn get_action(&self, node_id: NodeId) -> Result<&ActionInstance, Error> {
        self.get_node(node_id)?
            .payload
            .try_as_action_ref()
            .ok_or(Error::NotAnAction(node_id))
    }

    fn get_action_mut(&mut self, node_id: NodeId) -> Result<&mut ActionInstance, Error> {
        self.get_node_mut(node_id)?
            .payload
            .try_as_action_mut()
            .ok_or(Error::NotAnAction(node_id))
    }

    #[must_use]
    pub fn can_drop(&self, selection_ids: &[NodeId], target_id: NodeId, mode: DropMode) -> bool {
        if selection_ids.is_empty() {
            return false;
        }

        let mut seen_ids = HashSet::with_capacity(selection_ids.len());
        for &source_id in selection_ids {
            let Ok(node) = self.get_node(source_id) else {
                return false;
            };
            if !seen_ids.insert(source_id) || !node.can_be_moved() {
                return false;
            }
        }

        for (index, &source_id) in selection_ids.iter().enumerate() {
            if source_id == target_id || self.is_ancestor(source_id, target_id) {
                return false;
            }

            for &other_id in &selection_ids[index + 1..] {
                if self.is_ancestor(source_id, other_id) || self.is_ancestor(other_id, source_id) {
                    return false;
                }
            }
        }

        self.can_place_at_target(target_id, mode)
    }

    /// Appends a new action as the last child of `parent_id` and returns its id.
    pub fn append_new_action(
        &mut self,
        definition: &ActionDefinition,
        parent_id: NodeId,
    ) -> Result<NodeId, Error> {
        self.append_action_instance((definition.create_instance)(), parent_id)
    }

    /// Appends an existing action instance as the last child of `parent_id` and
    /// returns its id.
    pub fn append_action_instance(
        &mut self,
        action_instance: ActionInstance,
        parent_id: NodeId,
    ) -> Result<NodeId, Error> {
        {
            let parent = self.get_node(parent_id)?;

            if !parent.can_have_children() {
                return Err(Error::NodeCannotHaveChildren(parent_id));
            }
        }

        let parent_depth = self.get_node(parent_id)?.depth();
        let action_depth = parent_depth + 1;

        let branches = action_instance.branches();
        let action = Node {
            parent_id: Some(parent_id),
            children: Vec::new(), // We add branch children later in this function
            payload: NodePayload::Action(action_instance),
            metadata: Metadata {
                depth: action_depth,
                ..Default::default()
            },
        };
        let action_id = self.map.insert(action);

        let branches = branches
            .into_iter()
            .map(|kind| Node::new_branch(kind, action_id, action_depth + 1))
            .map(|node| self.map.insert(node))
            .collect::<Vec<_>>();

        self.map[action_id].children = branches;

        let start_row = match self.next_sibling_or_ancestor(parent_id) {
            Some(after) => self.row_of[after],
            None => self.rows.len(),
        };

        let parent = self.get_node_mut(parent_id)?;
        parent.children.push(action_id);

        self.update_downstream_rows_from(action_id, start_row);

        Ok(action_id)
    }

    /// Inserts a new action relative to `target_id` according to `mode`
    /// (before/after a sibling, or as last child). Mirrors `append_new_action`
    /// but supports all three `DropMode` variants.
    pub fn place_new_action(
        &mut self,
        definition: &ActionDefinition,
        target_id: NodeId,
        mode: DropMode,
    ) -> Result<NodeId, Error> {
        let (parent_id, insert_index) = self.placement(target_id, mode)?;

        let parent_depth = self.get_node(parent_id)?.depth();
        let action_depth = parent_depth + 1;

        let action_instance = (definition.create_instance)();
        let branches = action_instance.branches();
        let action = Node {
            parent_id: Some(parent_id),
            children: Vec::new(),
            payload: NodePayload::Action(action_instance),
            metadata: Metadata {
                depth: action_depth,
                ..Default::default()
            },
        };
        let action_id = self.map.insert(action);

        let branch_ids = branches
            .into_iter()
            .map(|kind| Node::new_branch(kind, action_id, action_depth + 1))
            .map(|node| self.map.insert(node))
            .collect::<Vec<_>>();

        self.map[action_id].children = branch_ids;
        self.map[parent_id].children.insert(insert_index, action_id);
        self.update_rows();

        Ok(action_id)
    }

    /// Returns the definition of an action at a node, or None if the node is not an action.
    pub fn definition(&self, node_id: NodeId) -> Result<Option<&'static ActionDefinition>, Error> {
        let node = self.get_node(node_id)?;

        Ok(node
            .payload
            .try_as_action_ref()
            .map(|action| action.definition()))
    }

    /// Reads an action parameter's stored value as JSON.
    ///
    /// `param_id` is the parameter's id (the action struct's field name). The
    /// returned value is the field's serialized form. Errors if the node is
    /// not an action or has no such parameter.
    pub fn action_parameter(
        &self,
        node_id: NodeId,
        param_id: &str,
    ) -> Result<serde_json::Value, Error> {
        let action = self.get_action(node_id)?;
        let mut json = serde_json::to_value(action)
            .map_err(|e| Error::ParameterSerialization(e.to_string()))?;
        json.get_mut(param_id)
            .map(serde_json::Value::take)
            .ok_or_else(|| Error::UnknownParameter(param_id.to_owned()))
    }

    /// Returns an action's timeout setting.
    pub fn timeout(&self, node_id: NodeId) -> Result<Option<DurationValue>, Error> {
        let action = self.get_action(node_id)?;
        Ok(*action.timeout())
    }

    /// Sets an action's timeout setting and reconciles its timeout branch.
    pub fn set_timeout(
        &mut self,
        node_id: NodeId,
        timeout: Option<DurationValue>,
    ) -> Result<(), Error> {
        self.get_action_mut(node_id)?.set_timeout(timeout);
        self.reconcile_action_branches(node_id)
    }

    /// Returns an action's pause-before setting.
    pub fn pause_before(&self, node_id: NodeId) -> Result<Option<DurationValue>, Error> {
        let action = self.get_action(node_id)?;
        Ok(*action.pause_before())
    }

    /// Sets an action's pause-before setting.
    pub fn set_pause_before(
        &mut self,
        node_id: NodeId,
        pause_before: Option<DurationValue>,
    ) -> Result<(), Error> {
        self.get_action_mut(node_id)?.set_pause_before(pause_before);
        Ok(())
    }

    /// Returns an action's pause-after setting.
    pub fn pause_after(&self, node_id: NodeId) -> Result<Option<DurationValue>, Error> {
        let action = self.get_action(node_id)?;
        Ok(*action.pause_after())
    }

    /// Sets an action's pause-after setting.
    pub fn set_pause_after(
        &mut self,
        node_id: NodeId,
        pause_after: Option<DurationValue>,
    ) -> Result<(), Error> {
        self.get_action_mut(node_id)?.set_pause_after(pause_after);
        Ok(())
    }

    /// Writes an action parameter's value from JSON, replacing the action
    /// instance with the patched version.
    ///
    /// The instance is serialized, the `param_id` field replaced with `value`,
    /// and the result deserialized back — so `value` must be a valid encoding
    /// for that parameter's type. Errors if the node is not an action, has no
    /// such parameter, or the patched value no longer deserializes.
    pub fn set_action_parameter(
        &mut self,
        node_id: NodeId,
        param_id: &str,
        value: serde_json::Value,
    ) -> Result<(), Error> {
        let action = self.get_action_mut(node_id)?;
        let mut json = serde_json::to_value(&*action)
            .map_err(|e| Error::ParameterSerialization(e.to_string()))?;
        let slot = json
            .get_mut(param_id)
            .ok_or_else(|| Error::UnknownParameter(param_id.to_owned()))?;
        *slot = value;
        *action = serde_json::from_value(json)
            .map_err(|e| Error::ParameterSerialization(e.to_string()))?;
        self.reconcile_action_branches(node_id)?;
        Ok(())
    }

    pub fn move_nodes(
        &mut self,
        selection_ids: &[NodeId],
        target_id: NodeId,
        mode: DropMode,
    ) -> Result<(), Error> {
        if !self.can_drop(selection_ids, target_id, mode) {
            return Err(Error::DropNotAllowed);
        }

        let moved = selection_ids.to_vec();
        let new_parent = match mode {
            DropMode::AppendChild => target_id,
            DropMode::Before | DropMode::After => self.get_parent(target_id),
        };

        for &node_id in &moved {
            let old_parent = self.get_parent(node_id);
            let siblings = &mut self.map[old_parent].children;
            let index = siblings
                .iter()
                .position(|&child| child == node_id)
                .expect("child not found in old parent's children");
            siblings.remove(index);
        }

        let insert_index = match mode {
            DropMode::AppendChild => self.map[new_parent].children.len(),
            DropMode::Before | DropMode::After => {
                let target_index = self.map[new_parent]
                    .children
                    .iter()
                    .position(|&child| child == target_id)
                    .expect("target not found in new parent's children");
                target_index + usize::from(mode == DropMode::After)
            }
        };

        {
            let siblings = &mut self.map[new_parent].children;
            for (offset, &node_id) in moved.iter().enumerate() {
                siblings.insert(insert_index + offset, node_id);
            }
        }

        let new_depth = self.map[new_parent].depth() + 1;
        for &node_id in &moved {
            self.map[node_id].parent_id = Some(new_parent);
            self.update_subtree_depth(node_id, new_depth);
        }

        self.update_rows();

        Ok(())
    }

    pub fn copy_subtrees(&self, selection_ids: &[NodeId]) -> Result<ClipboardTree, Error> {
        if selection_ids.is_empty() {
            return Err(Error::DropNotAllowed);
        }

        let mut seen_ids = HashSet::with_capacity(selection_ids.len());
        for &node_id in selection_ids {
            let node = self.get_node(node_id)?;
            if !seen_ids.insert(node_id) || !node.is_action() {
                return Err(Error::DropNotAllowed);
            }
        }

        for (index, &node_id) in selection_ids.iter().enumerate() {
            for &other_id in &selection_ids[index + 1..] {
                if self.is_ancestor(node_id, other_id) || self.is_ancestor(other_id, node_id) {
                    return Err(Error::DropNotAllowed);
                }
            }
        }

        let roots = self
            .ordered_selection(selection_ids)
            .into_iter()
            .map(|node_id| self.copy_node(node_id))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ClipboardTree { roots })
    }

    #[must_use]
    pub fn can_paste_subtrees(
        &self,
        clipboard: &ClipboardTree,
        target_id: NodeId,
        mode: DropMode,
    ) -> bool {
        !clipboard.is_empty()
            && clipboard.roots.iter().all(|node| node.payload.is_action())
            && self.can_place_at_target(target_id, mode)
    }

    pub fn paste_subtrees(
        &mut self,
        clipboard: &ClipboardTree,
        target_id: NodeId,
        mode: DropMode,
    ) -> Result<Vec<NodeId>, Error> {
        if !self.can_paste_subtrees(clipboard, target_id, mode) {
            return Err(Error::DropNotAllowed);
        }

        let (parent_id, insert_index) = self.placement(target_id, mode)?;
        let depth = self.map[parent_id].depth() + 1;
        let mut roots = Vec::with_capacity(clipboard.roots.len());

        for (offset, clipboard_node) in clipboard.roots.iter().enumerate() {
            let new_id = self.insert_clipboard_node(clipboard_node, parent_id, depth);
            self.map[parent_id]
                .children
                .insert(insert_index + offset, new_id);
            roots.push(new_id);
        }

        self.update_rows();

        Ok(roots)
    }

    /// Removes a node and its entire subtree. Errors if the id is unknown or the node is static.
    pub fn remove(&mut self, node_id: NodeId) -> Result<(), Error> {
        let node = self.get_node(node_id)?;

        if !node.can_be_removed() {
            return Err(Error::NodeCannotBeRemoved(node_id));
        }

        let parent_id = self.get_parent(node_id);
        let start_row = self.get_node_row(node_id);

        // The node that should occupy `start_row` after the deletion (if any).
        // Captured before mutation so the tree is still intact.
        let resume = self.next_sibling_or_ancestor(node_id);

        // Drop the whole subtree from the slotmap and side indices.
        let mut stack = vec![node_id];
        while let Some(node_id) = stack.pop() {
            self.row_of.remove(node_id);
            if let Some(removed) = self.map.remove(node_id) {
                stack.extend(removed.children.iter().rev().copied());
                if let Some(label) = removed.metadata.label {
                    self.label_map.remove(&label);
                }
            }
        }

        // Detach from parent's children.
        let siblings = &mut self.map[parent_id].children;
        if let Some(pos) = siblings
            .iter()
            .position(|&sibling_id| sibling_id == node_id)
        {
            siblings.remove(pos);
        }

        // Renumber the suffix.
        match resume {
            Some(resume) => self.update_downstream_rows_from(resume, start_row),
            None => self.rows.truncate(start_row),
        }

        Ok(())
    }

    /// Renumbers rows from `start_id`'s current row through the end of the tree.
    fn update_downstream_rows(&mut self, start_id: NodeId) {
        let start_row = self.row_of[start_id];
        self.update_downstream_rows_from(start_id, start_row);
    }

    /// Renumbers rows starting at `start_row`, walking preorder from `start_id` to the end of the tree.
    fn update_downstream_rows_from(&mut self, start_id: NodeId, start_row: usize) {
        self.rows.truncate(start_row);
        let mut cur = Some(start_id);
        let mut row = start_row;
        while let Some(id) = cur {
            self.rows.push(id);
            self.row_of.insert(id, row);
            row += 1;
            cur = self.next_in_preorder(id);
        }
    }

    fn update_subtree_depth(&mut self, root_id: NodeId, depth: usize) {
        self.map[root_id].metadata.depth = depth;
        let children = self.map[root_id].children.clone();
        for child in children {
            self.update_subtree_depth(child, depth + 1);
        }
    }

    #[must_use]
    pub fn can_place_at_target(&self, target_id: NodeId, mode: DropMode) -> bool {
        let Ok(target) = self.get_node(target_id) else {
            return false;
        };

        match mode {
            DropMode::AppendChild => target.can_have_children(),
            DropMode::Before | DropMode::After => target.is_action(),
        }
    }

    fn placement(&self, target_id: NodeId, mode: DropMode) -> Result<(NodeId, usize), Error> {
        if !self.can_place_at_target(target_id, mode) {
            return Err(Error::DropNotAllowed);
        }

        let parent_id = match mode {
            DropMode::AppendChild => target_id,
            DropMode::Before | DropMode::After => self.get_parent(target_id),
        };

        let insert_index = match mode {
            DropMode::AppendChild => self.map[parent_id].children.len(),
            DropMode::Before | DropMode::After => {
                let target_index = self.map[parent_id]
                    .children
                    .iter()
                    .position(|&child| child == target_id)
                    .expect("target not found in new parent's children");
                target_index + usize::from(mode == DropMode::After)
            }
        };

        Ok((parent_id, insert_index))
    }

    fn ordered_selection(&self, selection_ids: &[NodeId]) -> Vec<NodeId> {
        let selected = selection_ids.iter().copied().collect::<HashSet<_>>();
        self.rows()
            .iter()
            .copied()
            .filter(|id| selected.contains(id))
            .collect()
    }

    fn copy_node(&self, node_id: NodeId) -> Result<ClipboardNode, Error> {
        let node = self.get_node(node_id)?;
        Ok(ClipboardNode {
            payload: node.payload.clone(),
            metadata: node.metadata.clone(),
            children: node
                .children
                .iter()
                .copied()
                .map(|child| self.copy_node(child))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    fn insert_clipboard_node(
        &mut self,
        clipboard_node: &ClipboardNode,
        parent_id: NodeId,
        depth: usize,
    ) -> NodeId {
        let mut metadata = clipboard_node.metadata.clone();
        metadata.depth = depth;
        if let Some(label) = metadata.label.as_deref() {
            metadata.label = Some(self.unique_pasted_label(label));
        }

        let node = Node {
            parent_id: Some(parent_id),
            children: Vec::new(),
            payload: clipboard_node.payload.clone(),
            metadata,
        };
        let node_id = self.map.insert(node);

        if let Some(label) = self.map[node_id].metadata.label.clone() {
            self.label_map.insert(label, node_id);
        }

        let children = clipboard_node
            .children
            .iter()
            .map(|child| self.insert_clipboard_node(child, node_id, depth + 1))
            .collect();
        self.map[node_id].children = children;

        node_id
    }

    fn reconcile_action_branches(&mut self, node_id: NodeId) -> Result<(), Error> {
        let desired = { self.get_action(node_id)?.branches() };
        let depth = self.map[node_id].depth() + 1;
        let mut existing = mem::take(&mut self.map[node_id].children);
        let mut children = Vec::with_capacity(desired.len());

        for kind in desired {
            let existing_index = existing.iter().position(|&child_id| {
                self.branch_kind(child_id)
                    .is_some_and(|branch| branch == &kind)
            });

            let branch_id = match existing_index {
                Some(index) => existing.remove(index),
                None => self.map.insert(Node::new_branch(kind, node_id, depth)),
            };

            self.map[branch_id].parent_id = Some(node_id);
            self.update_subtree_depth(branch_id, depth);
            children.push(branch_id);
        }

        for stale_branch in existing {
            self.remove_subtree(stale_branch);
        }

        self.map[node_id].children = children;
        self.update_rows();

        Ok(())
    }

    fn branch_kind(&self, node_id: NodeId) -> Option<&BranchKind> {
        let node = self.map.get(node_id)?;
        let NodePayload::Static(super::Static::Branch(kind)) = node.payload() else {
            return None;
        };
        Some(kind)
    }

    fn remove_subtree(&mut self, node_id: NodeId) {
        let mut stack = vec![node_id];
        while let Some(node_id) = stack.pop() {
            self.row_of.remove(node_id);
            if let Some(removed) = self.map.remove(node_id) {
                stack.extend(removed.children.iter().rev().copied());
                if let Some(label) = removed.metadata.label {
                    self.label_map.remove(&label);
                }
            }
        }
    }

    fn unique_pasted_label(&self, label: &str) -> String {
        if self.node_by_label(label).is_none() {
            return label.to_owned();
        }

        let base = format!("{label}_copy");
        if self.node_by_label(&base).is_none() {
            return base;
        }

        let mut suffix = 2;
        loop {
            let candidate = format!("{base}_{suffix}");
            if self.node_by_label(&candidate).is_none() {
                return candidate;
            }
            suffix += 1;
        }
    }

    /// Rebuilds the entire row index from scratch by walking the tree in preorder.
    fn update_rows(&mut self) {
        self.row_of.clear();

        let mut all_node_ids = Vec::with_capacity(self.map.len());
        let mut stack = vec![self.root];

        while let Some(node_id) = stack.pop() {
            all_node_ids.push(node_id);
            let node = self.map.get(node_id).expect("dangling id in tree");
            // reverse so children pop in original order
            stack.extend(node.children.iter().rev().copied());
        }

        for (row, &node_id) in all_node_ids.iter().enumerate() {
            self.row_of.insert(node_id, row);
        }
        self.rows = all_node_ids;
    }
}
