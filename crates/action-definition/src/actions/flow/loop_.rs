use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    tree::BranchKind,
};

/// Repeats its body until it is stopped, exited, or broken out of.
#[action(
    icon = CornerDownRight,
    effect = ControlFlow,
    category = Flow,
    looping = true
)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::empty_structs_with_brackets)]
pub struct Loop {}

impl ActionBranches for Loop {
    fn action_branches(&self) -> Vec<BranchKind> {
        vec![BranchKind::Body]
    }
}

impl ParameterAvailability for Loop {}
