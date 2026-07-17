use serde::{Deserialize, Serialize};
use slotmap::new_key_type;
use strum::{Display, EnumIs, EnumTryAs};

use crate::actions::ActionInstance;

#[derive(Clone, Debug, Deserialize, Display, EnumIs, EnumTryAs, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchKind {
    Body,
    Ok,
    Yes,
    No,
    Cancel,
    True,
    False,
    Timeout,
    Pressed,
    Released,
    Default,
    Named(String),
}

#[derive(Clone, Debug, Deserialize, EnumIs, EnumTryAs, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Static {
    Root,
    Branch(BranchKind),
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, EnumIs, EnumTryAs, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodePayload {
    Static(Static),
    Action(ActionInstance),
}

new_key_type! {
    pub struct NodeId;
}

#[derive(Clone, Debug, Default)]
pub struct Metadata {
    pub(super) label: Option<String>,
    pub(super) comment: Option<String>,
    /// Depth of the node in the tree: the root is `0`, its children `1`, and so on.
    pub(super) depth: usize,
    /// Whether the node's children are hidden (collapsed) in the UI.
    pub(super) collapsed: bool,
}

impl Serialize for Metadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            #[derive(Serialize)]
            struct HumanReadable<'a> {
                #[serde(skip_serializing_if = "Option::is_none")]
                label: Option<&'a str>,
                #[serde(skip_serializing_if = "Option::is_none")]
                comment: Option<&'a str>,
                #[serde(skip_serializing_if = "is_false")]
                collapsed: bool,
            }

            HumanReadable {
                label: self.label.as_deref(),
                comment: self.comment.as_deref(),
                collapsed: self.collapsed,
            }
            .serialize(serializer)
        } else {
            (&self.label, &self.comment, self.collapsed).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Metadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            #[derive(Default, Deserialize)]
            #[serde(default)]
            struct HumanReadable {
                label: Option<String>,
                comment: Option<String>,
                collapsed: bool,
            }

            let HumanReadable {
                label,
                comment,
                collapsed,
            } = HumanReadable::deserialize(deserializer)?;
            Ok(Self {
                label,
                comment,
                collapsed,
                ..Default::default()
            })
        } else {
            let (label, comment, collapsed) = Deserialize::deserialize(deserializer)?;
            Ok(Self {
                label,
                comment,
                collapsed,
                ..Default::default()
            })
        }
    }
}

impl Metadata {
    pub(super) const fn is_empty(&self) -> bool {
        self.label.is_none() && self.comment.is_none() && !self.collapsed
    }
}

const fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug)]
pub struct Node {
    pub(super) parent_id: Option<NodeId>,
    pub(super) children: Vec<NodeId>,
    pub(super) payload: NodePayload,
    pub(super) metadata: Metadata,
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            #[derive(Serialize)]
            struct HumanReadable<'a> {
                parent_id: Option<NodeId>,
                #[serde(skip_serializing_if = "Option::is_none")]
                children: Option<&'a [NodeId]>,
                payload: &'a NodePayload,
                #[serde(skip_serializing_if = "Option::is_none")]
                metadata: Option<&'a Metadata>,
            }

            HumanReadable {
                parent_id: self.parent_id,
                children: (!self.children.is_empty()).then_some(&self.children),
                payload: &self.payload,
                metadata: (!self.metadata.is_empty()).then_some(&self.metadata),
            }
            .serialize(serializer)
        } else {
            (
                &self.parent_id,
                &self.children,
                &self.payload,
                &self.metadata,
            )
                .serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            #[derive(Deserialize)]
            struct HumanReadable {
                parent_id: Option<NodeId>,
                #[serde(default)]
                children: Vec<NodeId>,
                payload: NodePayload,
                #[serde(default)]
                metadata: Metadata,
            }

            let HumanReadable {
                parent_id,
                children,
                payload,
                metadata,
            } = HumanReadable::deserialize(deserializer)?;
            Ok(Self {
                parent_id,
                children,
                payload,
                metadata,
            })
        } else {
            let (parent_id, children, payload, metadata) = Deserialize::deserialize(deserializer)?;
            Ok(Self {
                parent_id,
                children,
                payload,
                metadata,
            })
        }
    }
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

    #[must_use]
    pub const fn can_have_children(&self) -> bool {
        self.payload.is_static()
    }

    #[must_use]
    pub const fn can_be_removed(&self) -> bool {
        self.payload.is_action()
    }

    #[must_use]
    pub const fn can_be_moved(&self) -> bool {
        self.payload.is_action()
    }

    #[must_use]
    pub const fn is_selectable(&self) -> bool {
        self.payload.is_action()
    }

    #[must_use]
    pub const fn is_action(&self) -> bool {
        self.payload.is_action()
    }

    #[must_use]
    pub const fn is_static(&self) -> bool {
        self.payload.is_static()
    }

    #[must_use]
    pub const fn is_root(&self) -> bool {
        matches!(&self.payload, NodePayload::Static(Static::Root))
    }

    #[must_use]
    pub const fn is_branch(&self) -> bool {
        matches!(&self.payload, NodePayload::Static(Static::Branch(_)))
    }

    /// The node's payload (its static role or action instance).
    #[must_use]
    pub const fn payload(&self) -> &NodePayload {
        &self.payload
    }

    /// The node's children, in order.
    #[must_use]
    pub fn children(&self) -> &[NodeId] {
        &self.children
    }

    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.metadata.label.as_deref()
    }

    #[must_use]
    pub fn comment(&self) -> Option<&str> {
        self.metadata.comment.as_deref()
    }

    /// The node's depth in the tree: the root is `0`, its children `1`, and so on.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.metadata.depth
    }

    /// Whether the node has any children.
    #[must_use]
    pub const fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Whether the node's children are hidden (collapsed) in the UI.
    #[must_use]
    pub const fn is_collapsed(&self) -> bool {
        self.metadata.collapsed
    }
}
