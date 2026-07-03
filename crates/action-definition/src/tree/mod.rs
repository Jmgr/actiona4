mod error;
mod metadata;
mod mutation;
mod node;
mod traversal;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

pub use error::Error;
pub use node::{BranchKind, Metadata, Node, NodeId, NodePayload, Static};
use serde::{Deserialize, Serialize};
use slotmap::{SecondaryMap, SlotMap};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClipboardTree {
    roots: Vec<ClipboardNode>,
}

impl ClipboardTree {
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ClipboardNode {
    payload: NodePayload,
    metadata: Metadata,
    children: Vec<ClipboardNode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DropMode {
    AppendChild,
    Before,
    After,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionTree {
    map: SlotMap<NodeId, Node>,
    root: NodeId,
    label_map: HashMap<String, NodeId>, // fast label => node lookup
    rows: Vec<NodeId>,                  // fast row => node lookup
    row_of: SecondaryMap<NodeId, usize>, // fast node => row lookup
}

impl Default for ActionTree {
    fn default() -> Self {
        let mut map = SlotMap::default();

        let root_id = map.insert(Node::new_root());

        let mut row_of = SecondaryMap::default();
        row_of.insert(root_id, 0);

        let rows = vec![root_id];

        Self {
            map,
            root: root_id,
            label_map: Default::default(),
            rows,
            row_of,
        }
    }
}
