use macros::action;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability},
    parameters::variable::Variable,
};

#[action(icon = MousePointer2, effect = ReadState, category = Mouse)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GetCursorPosition {
    #[parameter]
    pub result: Variable,
}

impl ActionBranches for GetCursorPosition {}

impl ParameterAvailability for GetCursorPosition {}
