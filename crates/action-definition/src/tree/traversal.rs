use std::iter;

use super::{ActionTree, Error, Node, NodeId};
use crate::actions::ActionInstance;

impl ActionTree {
    /// The id of the root node, where execution begins.
    #[must_use]
    pub const fn root(&self) -> NodeId {
        self.root
    }

    /// Iterates over the node's ancestors, nearest first, ending at the root.
    /// Yields nothing for the root or an unknown node.
    pub fn ancestors(&self, node_id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        let mut current_id = self.map.get(node_id).and_then(|node| node.parent_id);
        iter::from_fn(move || {
            let id = current_id?;
            current_id = self.map.get(id).and_then(|node| node.parent_id);
            Some(id)
        })
    }

    /// Whether `ancestor` is a strict ancestor of `descendant`.
    #[must_use]
    pub fn is_ancestor(&self, ancestor_id: NodeId, descendant_id: NodeId) -> bool {
        self.ancestors(descendant_id).any(|id| id == ancestor_id)
    }

    #[must_use]
    pub fn rows(&self) -> &[NodeId] {
        &self.rows
    }

    /// Returns the node's index in the tree-wide preorder row list.
    ///
    /// The hidden root is row `0`, so visible line numbers start at `1`.
    pub fn node_row(&self, node_id: NodeId) -> Result<usize, Error> {
        self.row_of
            .get(node_id)
            .copied()
            .ok_or(Error::InvalidNode(node_id))
    }

    /// Returns the nodes to display, in preorder, skipping the descendants of any
    /// collapsed node (the collapsed node itself is still shown).
    #[must_use]
    pub fn visible_rows(&self) -> Vec<NodeId> {
        let root = self.map.get(self.root).expect("dangling root in tree");
        let mut visible_ids = Vec::with_capacity(self.rows.len().saturating_sub(1));
        let mut stack = root.children.iter().rev().copied().collect::<Vec<_>>();

        while let Some(node_id) = stack.pop() {
            visible_ids.push(node_id);
            let node = self.map.get(node_id).expect("dangling id in tree");
            if !node.is_collapsed() {
                stack.extend(node.children.iter().rev().copied());
            }
        }

        visible_ids
    }

    /// Iterates over `root_id` and all its descendants in preorder.
    fn descendants(&self, root_id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        let mut stack = vec![root_id];

        iter::from_fn(move || {
            let id = stack.pop()?;
            let node = self.map.get(id).expect("dangling id in tree");
            // reverse so children pop in original order
            stack.extend(node.children.iter().rev().copied());
            Some(id)
        })
    }

    fn actions(&self) -> impl Iterator<Item = (NodeId, &'_ ActionInstance)> + '_ {
        self.map.iter().filter_map(|(node_id, node)| {
            node.payload
                .try_as_action_ref()
                .map(|action_instance| (node_id, action_instance))
        })
    }

    fn actions_mut(&mut self) -> impl Iterator<Item = (NodeId, &'_ mut ActionInstance)> + '_ {
        self.map.iter_mut().filter_map(|(node_id, node)| {
            node.payload
                .try_as_action_mut()
                .map(|action_instance| (node_id, action_instance))
        })
    }

    /// Returns the parent id of a known non-root node.
    pub(super) fn get_parent(&self, node_id: NodeId) -> NodeId {
        let node = &self.map[node_id];
        node.parent_id.expect("non-root node should have a parent")
    }

    /// Returns the node's index in the tree-wide preorder row list (root is row 0).
    pub(super) fn get_node_row(&self, node_id: NodeId) -> usize {
        self.row_of
            .get(node_id)
            .copied()
            .expect("known node should have a row")
    }

    /// Returns a reference to a node by id.
    pub fn get_node(&self, node_id: NodeId) -> Result<&Node, Error> {
        self.map.get(node_id).ok_or(Error::InvalidNode(node_id))
    }

    /// Returns a mutable reference to a node by id.
    pub(super) fn get_node_mut(&mut self, node_id: NodeId) -> Result<&mut Node, Error> {
        self.map.get_mut(node_id).ok_or(Error::InvalidNode(node_id))
    }

    /// Returns the next node after `id` in a preorder traversal, or `None` at the end of the tree.
    #[must_use]
    pub fn next_in_preorder(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(&first) = self.map[node_id].children.first() {
            return Some(first);
        }
        self.next_sibling_or_ancestor(node_id)
    }

    /// Returns the first node in preorder after `id`'s subtree — its next sibling, or the next sibling of an ancestor.
    #[must_use]
    pub fn next_sibling_or_ancestor(&self, node_id: NodeId) -> Option<NodeId> {
        let mut current_id = node_id;
        while let Some(parent_id) = self.map[current_id].parent_id {
            let sibling_ids = &self.map[parent_id].children;
            let pos = sibling_ids
                .iter()
                .position(|&c| c == current_id)
                .expect("child not found in parent's children");
            if let Some(&next_id) = sibling_ids.get(pos + 1) {
                return Some(next_id);
            }
            current_id = parent_id;
        }
        None
    }
}
