use macros::ActionEnum;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{Param, label::Label, source_code::SourceCode, variable::Variable},
    scriptable::Scriptable,
    tree::BranchKind,
};

#[action(icon = ShieldX, effect = ReadState, category = Clipboard)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GetClipboardText {
    #[parameter]
    pub result: Variable,

    #[parameter(only = Linux)]
    pub selection: Scriptable<bool>,
}

impl ActionBranches for GetClipboardText {}

impl ParameterAvailability for GetClipboardText {}
