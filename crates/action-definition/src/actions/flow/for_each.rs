use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{array::Array, variable::Variable},
    tree::BranchKind,
};

/// Repeats its body once for every item in an array.
#[action(
    icon = CornerDownRight,
    effect = ControlFlow,
    category = Flow,
    looping = true
)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ForEach {
    #[parameter]
    pub array: Array,

    #[parameter]
    pub item_variable: Variable,
}

impl Default for ForEach {
    fn default() -> Self {
        Self {
            array: Default::default(),
            item_variable: Variable::new("item").into(),
        }
    }
}

impl ActionBranches for ForEach {
    fn action_branches(&self) -> Vec<BranchKind> {
        vec![BranchKind::Body]
    }
}

impl ParameterAvailability for ForEach {}
