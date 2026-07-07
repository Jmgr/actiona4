use std::time::Duration;

use macros::action;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability},
    parameters::duration::DurationValue,
    scriptable::Scriptable,
};

#[action(icon = MousePointer2, effect = ReadState, category = Clipboard, timeout = true)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WaitForClipboardChanged {
    #[parameter]
    pub check_interval: Scriptable<DurationValue>,

    #[parameter(only = Linux)]
    pub selection: Scriptable<bool>,
}

impl Default for WaitForClipboardChanged {
    fn default() -> Self {
        Self {
            check_interval: Scriptable::Static {
                value: Duration::from_millis(200).into(),
            }
            .into(),
            selection: Default::default(),
        }
    }
}

impl ActionBranches for WaitForClipboardChanged {}

impl ParameterAvailability for WaitForClipboardChanged {}
