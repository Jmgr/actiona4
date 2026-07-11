use macros::ActionEnum;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{Param, label::Label, source_code::SourceCode, value::Value},
    tree::BranchKind,
};

/// Chooses a branch based on a value's truthiness.
#[action(icon = CornerDownRight, effect = ControlFlow, category = Flow)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct If {
    #[parameter]
    pub value: Value,
}

impl ActionBranches for If {
    fn action_branches(&self) -> Vec<BranchKind> {
        vec![BranchKind::True, BranchKind::False]
    }
}

impl ParameterAvailability for If {}
