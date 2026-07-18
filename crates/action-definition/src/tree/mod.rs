mod error;
mod metadata;
mod mutation;
mod node;
mod traversal;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

pub use error::Error;
pub use node::{BranchKind, Metadata, Node, NodeId, NodePayload, Static};
use serde::{Deserialize, Deserializer, Serialize, de};
use slotmap::{SecondaryMap, SlotMap};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClipboardTree {
    roots: Vec<ClipboardNode>,
}

impl ClipboardTree {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ClipboardNode {
    payload: NodePayload,
    metadata: Metadata,
    children: Vec<Self>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DropMode {
    AppendChild,
    Before,
    After,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActionTree {
    map: SlotMap<NodeId, Node>,

    root: NodeId,

    #[serde(skip)]
    label_map: HashMap<String, NodeId>, // fast label => node lookup

    #[serde(skip)]
    rows: Vec<NodeId>, // fast row => node lookup

    #[serde(skip)]
    row_of: SecondaryMap<NodeId, usize>, // fast node => row lookup
}

#[derive(Deserialize)]
struct SerializedActionTree {
    map: SlotMap<NodeId, Node>,
    root: NodeId,
}

impl<'de> Deserialize<'de> for ActionTree {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let SerializedActionTree { map, root } = SerializedActionTree::deserialize(deserializer)?;
        let mut tree = Self {
            map,
            root,
            label_map: HashMap::default(),
            rows: Vec::default(),
            row_of: SecondaryMap::default(),
        };
        tree.rebuild_derived_state().map_err(de::Error::custom)?;
        Ok(tree)
    }
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
            label_map: HashMap::default(),
            rows,
            row_of,
        }
    }
}

impl ActionTree {
    fn rebuild_derived_state(&mut self) -> Result<(), String> {
        self.label_map.clear();
        self.rows.clear();
        self.row_of.clear();

        let root = self
            .map
            .get(self.root)
            .ok_or_else(|| "tree root does not exist".to_owned())?;
        if !root.is_root() {
            return Err("tree root node must have root payload".to_owned());
        }
        if root.parent_id.is_some() {
            return Err("tree root node must not have a parent".to_owned());
        }

        let mut visited = HashSet::with_capacity(self.map.len());
        let mut stack = vec![(self.root, None, 0)];

        while let Some((node_id, expected_parent, depth)) = stack.pop() {
            if !self.map.contains_key(node_id) {
                return Err(format!("tree contains dangling child id {node_id:?}"));
            }
            if !visited.insert(node_id) {
                return Err(format!(
                    "tree contains a cycle or duplicate child id {node_id:?}"
                ));
            }

            let row = self.rows.len();
            self.rows.push(node_id);
            self.row_of.insert(node_id, row);

            let (label, children) = {
                let node = self
                    .map
                    .get_mut(node_id)
                    .expect("node existence checked above");
                if node.parent_id != expected_parent {
                    return Err(format!("node {node_id:?} has an inconsistent parent"));
                }
                node.metadata.depth = depth;
                (node.metadata.label.clone(), node.children.clone())
            };

            if let Some(label) = label
                && self.label_map.insert(label.clone(), node_id).is_some()
            {
                return Err(format!("duplicate node label {label:?}"));
            }

            stack.extend(
                children
                    .iter()
                    .rev()
                    .copied()
                    .map(|child_id| (child_id, Some(node_id), depth + 1)),
            );
        }

        if visited.len() != self.map.len() {
            return Err("tree contains unreachable nodes".to_owned());
        }

        Ok(())
    }
}
