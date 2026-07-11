use macros::action;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, scroll::Axis},
    scriptable::Scriptable,
};

/// Waits for mouse wheel input.
#[action(
    icon = MousePointer2,
    effect = ReadState,
    category = Mouse,
    timeout = true,
    waitable = true
)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WaitForScroll {
    #[parameter]
    pub axis: Scriptable<Option<Axis>>,
}

impl ActionBranches for WaitForScroll {}

impl ParameterAvailability for WaitForScroll {}
