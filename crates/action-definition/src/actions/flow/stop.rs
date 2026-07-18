use macros::ActionEnum;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{Param, label::Label, source_code::SourceCode},
    scriptable::Scriptable,
    tree::BranchKind,
};

/// Stops the current action tree execution.
#[action(icon = ShieldX, effect = ControlFlow, category = Flow)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::empty_structs_with_brackets)]
pub struct Stop {}

impl ActionBranches for Stop {}

impl ParameterAvailability for Stop {}
