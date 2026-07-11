use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    scriptable::Scriptable,
    tree::BranchKind,
};

/// Repeats its body a fixed number of times.
#[action(
    icon = CornerDownRight,
    effect = ControlFlow,
    category = Flow,
    looping = true
)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Loop {
    #[parameter]
    pub max_counter: Scriptable<u32>,
}

impl ActionBranches for Loop {
    fn action_branches(&self) -> Vec<BranchKind> {
        vec![BranchKind::Body]
    }
}

impl ParameterAvailability for Loop {}
