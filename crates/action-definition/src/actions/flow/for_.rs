use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{Param, variable::Variable},
    scriptable::Scriptable,
    tree::BranchKind,
};

/// Repeats its body for every zero-based index below a count.
#[action(
    icon = CornerDownRight,
    effect = ControlFlow,
    category = Flow,
    looping = true
)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct For {
    #[parameter]
    pub count: Scriptable<u32>,

    #[parameter]
    pub index_variable: Variable,
}

impl Default for For {
    fn default() -> Self {
        Self {
            count: Param::default(),
            index_variable: Variable::new("i").into(),
        }
    }
}

impl ActionBranches for For {
    fn action_branches(&self) -> Vec<BranchKind> {
        vec![BranchKind::Body]
    }
}

impl ParameterAvailability for For {}
