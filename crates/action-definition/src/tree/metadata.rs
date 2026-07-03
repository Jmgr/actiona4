use super::{ActionTree, Error, NodeId};

impl ActionTree {
    pub fn node_by_label(&self, label: &str) -> Option<NodeId> {
        self.label_map.get(label).copied()
    }

    pub fn set_node_label(
        &mut self,
        node_id: NodeId,
        label: impl Into<String>,
    ) -> Result<(), Error> {
        let label: String = label.into();
        if label.is_empty() {
            return self.clear_node_label(node_id);
        }

        if self
            .label_map
            .get(&label)
            .is_some_and(|&existing| existing != node_id)
        {
            return Err(Error::DuplicateLabel(label));
        }

        let node = self.get_node_mut(node_id)?;
        let old_label = node.metadata.label.replace(label.clone());
        if let Some(old_label) = old_label {
            self.label_map.remove(&old_label);
        }
        self.label_map.insert(label, node_id);

        Ok(())
    }

    pub fn clear_node_label(&mut self, node_id: NodeId) -> Result<(), Error> {
        let node = self.get_node_mut(node_id)?;

        if let Some(label) = node.metadata.label.take() {
            self.label_map.remove(&label);
        }

        Ok(())
    }

    pub fn set_node_comment(
        &mut self,
        node_id: NodeId,
        comment: impl Into<String>,
    ) -> Result<(), Error> {
        let comment: String = comment.into();
        if comment.is_empty() {
            return self.clear_node_comment(node_id);
        }

        let node = self.get_node_mut(node_id)?;
        node.metadata.comment = Some(comment);

        Ok(())
    }

    pub fn clear_node_comment(&mut self, node_id: NodeId) -> Result<(), Error> {
        let node = self.get_node_mut(node_id)?;
        node.metadata.comment = None;

        Ok(())
    }

    /// Sets whether the node's children are hidden (collapsed) in the UI.
    pub fn set_node_collapsed(&mut self, node_id: NodeId, collapsed: bool) -> Result<(), Error> {
        let node = self.get_node_mut(node_id)?;
        node.metadata.collapsed = collapsed;

        Ok(())
    }

    /// Flips the node's collapsed state, returning the new value.
    pub fn toggle_node_collapsed(&mut self, node_id: NodeId) -> Result<bool, Error> {
        let node = self.get_node_mut(node_id)?;
        node.metadata.collapsed = !node.metadata.collapsed;

        Ok(node.metadata.collapsed)
    }
}
