use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, MouseButton, ParameterAvailability, action},
    scriptable::Scriptable,
    tree::BranchKind,
};

#[action(icon = CornerDownRight, effect = ControlFlow, category = Flow)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ButtonCondition {
    #[parameter(translation = "action-click-button")]
    pub button: Scriptable<MouseButton>,
}

impl ActionBranches for ButtonCondition {
    fn action_branches(&self) -> Vec<BranchKind> {
        vec![BranchKind::Pressed, BranchKind::Released]
    }
}

impl ParameterAvailability for ButtonCondition {}
