use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    scriptable::Scriptable,
    tree::BranchKind,
};

/// Repeats its body while a condition remains true.
#[action(
    icon = CornerDownRight,
    effect = ControlFlow,
    category = Flow,
    looping = true
)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct While {
    #[parameter]
    pub condition: Scriptable<bool>,
}

impl ActionBranches for While {
    fn action_branches(&self) -> Vec<BranchKind> {
        vec![BranchKind::Body]
    }
}

impl ParameterAvailability for While {}
