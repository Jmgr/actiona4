use macros::ActionEnum;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{Param, label::Label, source_code::SourceCode},
    scriptable::Scriptable,
    tree::BranchKind,
};

/// Clears the system clipboard or Linux selection clipboard.
#[action(icon = ShieldX, effect = ChangeState, category = Clipboard)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ClearClipboard {
    #[parameter(only = Linux)]
    pub selection: Scriptable<bool>,
}

impl ActionBranches for ClearClipboard {}

impl ParameterAvailability for ClearClipboard {}
