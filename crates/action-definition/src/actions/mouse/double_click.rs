use macros::ActionEnum;
use serde::{Deserialize, Serialize};
use types::Point;

use crate::{
    actions::{ActionBranches, MouseButton, ParameterAvailability, action},
    parameters::duration::DurationValue,
    scriptable::Scriptable,
};

/// Double-clicks a mouse button at an optional position.
#[action(icon = MousePointerClick, effect = ChangeState, category = Mouse, timeout = true)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DoubleClick {
    #[parameter(translation = "action-click-position")]
    pub position: Scriptable<Option<Point>>,

    #[parameter(translation = "action-click-button")]
    pub button: Scriptable<MouseButton>,

    #[parameter(translation = "action-click-relative-position")]
    pub relative_position: Scriptable<bool>,

    #[parameter]
    pub delay: Scriptable<Option<DurationValue>>,
}

impl ActionBranches for DoubleClick {}

impl ParameterAvailability for DoubleClick {}
