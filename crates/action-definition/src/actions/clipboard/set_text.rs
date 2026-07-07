use macros::ActionEnum;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{Param, label::Label, source_code::SourceCode},
    scriptable::Scriptable,
    tree::BranchKind,
};

#[action(icon = ShieldX, effect = ChangeState, category = Clipboard)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SetClipboardText {
    #[parameter]
    pub text: Scriptable<String>,

    #[parameter(only = Linux)]
    pub selection: Scriptable<bool>,
}

impl ActionBranches for SetClipboardText {}

impl ParameterAvailability for SetClipboardText {}
