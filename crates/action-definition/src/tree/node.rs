use serde::{Deserialize, Serialize};
use slotmap::new_key_type;
use strum::{Display, EnumIs, EnumTryAs};

use crate::actions::ActionInstance;

#[derive(Clone, Debug, Deserialize, Display, EnumIs, EnumTryAs, PartialEq, Serialize)]
pub enum BranchKind {
    Yes,
    No,
    Cancel,
    True,
    False,
    Named(String),
}

#[derive(Clone, Debug, Deserialize, EnumIs, EnumTryAs, Serialize)]
pub enum Static {
    Root,
    Branch(BranchKind),
}

#[derive(Clone, Debug, Deserialize, EnumIs, EnumTryAs, Serialize)]
pub enum NodePayload {
    Static(Static),
    Action(ActionInstance),
}

new_key_type! {
    pub struct NodeId;
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Metadata {
    pub(super) label: Option<String>,
    pub(super) comment: Option<String>,
    /// Depth of the node in the tree: the root is `0`, its children `1`, and so on.
    #[serde(skip)]
    pub(super) depth: usize,
    /// Whether the node's children are hidden (collapsed) in the UI.
    pub(super) collapsed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    pub(super) parent_id: Option<NodeId>,
    pub(super) children: Vec<NodeId>,
    pub(super) payload: NodePayload,
    pub(super) metadata: Metadata,
}

impl Node {
    pub(super) fn new_root() -> Self {
        Self {
            parent_id: None,
            children: Default::default(),
            payload: NodePayload::Static(Static::Root),
            metadata: Metadata {
                depth: 0,
                ..Default::default()
            },
        }
    }

    pub(super) fn new_branch(kind: BranchKind, parent_id: NodeId, depth: usize) -> Self {
        Self {
            parent_id: Some(parent_id),
            children: Default::default(),
            payload: NodePayload::Static(Static::Branch(kind)),
            metadata: Metadata {
                depth,
                ..Default::default()
            },
        }
    }

    pub const fn can_have_children(&self) -> bool {
        self.payload.is_static()
    }

    pub const fn can_be_removed(&self) -> bool {
        self.payload.is_action()
    }

    pub const fn can_be_moved(&self) -> bool {
        self.payload.is_action()
    }

    pub const fn is_selectable(&self) -> bool {
        self.payload.is_action()
    }

    pub const fn is_action(&self) -> bool {
        self.payload.is_action()
    }

    pub const fn is_static(&self) -> bool {
        self.payload.is_static()
    }

    pub const fn is_root(&self) -> bool {
        matches!(&self.payload, NodePayload::Static(Static::Root))
    }

    pub const fn is_branch(&self) -> bool {
        matches!(&self.payload, NodePayload::Static(Static::Branch(_)))
    }

    /// The node's payload (its static role or action instance).
    pub const fn payload(&self) -> &NodePayload {
        &self.payload
    }

    /// The node's children, in order.
    pub fn children(&self) -> &[NodeId] {
        &self.children
    }

    pub fn label(&self) -> Option<&str> {
        self.metadata.label.as_deref()
    }

    pub fn comment(&self) -> Option<&str> {
        self.metadata.comment.as_deref()
    }

    /// The node's depth in the tree: the root is `0`, its children `1`, and so on.
    pub const fn depth(&self) -> usize {
        self.metadata.depth
    }

    /// Whether the node has any children.
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Whether the node's children are hidden (collapsed) in the UI.
    pub const fn is_collapsed(&self) -> bool {
        self.metadata.collapsed
    }
}
