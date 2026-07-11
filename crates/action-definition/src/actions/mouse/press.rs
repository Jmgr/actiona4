use macros::ActionEnum;
use serde::{Deserialize, Serialize};
use types::Point;

use crate::{
    actions::{ActionBranches, MouseButton, ParameterAvailability, action},
    parameters::duration::DurationValue,
    scriptable::Scriptable,
};

/// Moves to an optional position and presses a mouse button.
#[action(icon = MousePointerClick, effect = ChangeState, category = Mouse)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Press {
    #[parameter]
    pub position: Scriptable<Option<Point>>,

    #[parameter]
    pub button: Scriptable<MouseButton>,

    #[parameter(translation = "action-click-relative-position")]
    pub relative_position: Scriptable<bool>,
}

impl ActionBranches for Press {}

impl ParameterAvailability for Press {}
