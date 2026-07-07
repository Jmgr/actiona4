use macros::{ActionEnum, action};
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, MouseButton, ParameterAvailability},
    scriptable::Scriptable,
};

#[derive(ActionEnum, Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ButtonDirection {
    #[default]
    Press,
    Release,
}

#[action(icon = MousePointer2, effect = ReadState, category = Mouse, timeout = true)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WaitForButton {
    #[parameter]
    pub button: Scriptable<Option<MouseButton>>,

    #[parameter]
    pub direction: Scriptable<Option<ButtonDirection>>,
}

impl ActionBranches for WaitForButton {}

impl ParameterAvailability for WaitForButton {}
