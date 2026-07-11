use macros::ActionEnum;
use serde::{Deserialize, Serialize};
use types::Point;

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::duration::DurationValue,
    scriptable::Scriptable,
};

#[derive(ActionEnum, Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MouseButton {
    #[default]
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

/// Clicks a mouse button at an optional position.
#[action(icon = MousePointerClick, effect = ChangeState, category = Mouse, timeout = true)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Click {
    #[parameter(translation = "action-click-position")]
    pub position: Scriptable<Option<Point>>,

    #[parameter(translation = "action-click-button")]
    pub button: Scriptable<MouseButton>,

    #[parameter(translation = "action-click-relative-position")]
    pub relative_position: Scriptable<bool>,

    #[parameter(translation = "action-click-amount", min = Some(0), max = Some(i32::MAX as i64))]
    pub amount: Scriptable<Option<i64>>,

    #[parameter]
    pub interval: Scriptable<Option<DurationValue>>,

    #[parameter]
    pub duration: Scriptable<Option<DurationValue>>,
}

impl ActionBranches for Click {}

impl ParameterAvailability for Click {}
