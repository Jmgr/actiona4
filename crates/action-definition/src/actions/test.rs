use serde::{Deserialize, Serialize};

use crate::{
    actions::{Action, Branching},
    scriptable::Scriptable,
    tree::BranchKind,
};

#[derive(Action, Clone, Debug, Default, Deserialize, Serialize)]
#[action(icon = TestTubeDiagonal)]
pub struct Test {
    #[parameter]
    pub percent: Scriptable<i64>,
}

impl Branching for Test {
    fn branches(&self) -> Vec<BranchKind> {
        vec![BranchKind::True, BranchKind::False]
    }
}
