use macros::action;
use serde::{Deserialize, Serialize};
use types::Point;

use crate::{
    actions::{ActionBranches, ParameterAvailability},
    scriptable::Scriptable,
};

/// Sets the cursor position immediately.
#[action(icon = MousePointer2, effect = ChangeState, category = Mouse)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SetCursorPosition {
    #[parameter]
    pub position: Scriptable<Point>,

    #[parameter(translation = "action-click-relative-position")]
    pub relative_position: Scriptable<bool>,
}

impl ActionBranches for SetCursorPosition {}

impl ParameterAvailability for SetCursorPosition {}
