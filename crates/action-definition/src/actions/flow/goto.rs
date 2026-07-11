use macros::ActionEnum;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{Param, label::Label, source_code::SourceCode},
    scriptable::Scriptable,
    tree::BranchKind,
};

/// Continues execution at a labeled action.
#[action(icon = CornerDownRight, effect = ControlFlow, category = Flow)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Goto {
    #[parameter]
    pub target: Scriptable<Label>,
}

impl ActionBranches for Goto {}

impl ParameterAvailability for Goto {}
