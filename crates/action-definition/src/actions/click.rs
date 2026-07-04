use macros::ActionEnum;
use serde::{Deserialize, Serialize};
use types::Point;

use crate::{
    actions::{Branching, action},
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

#[action(icon = MousePointerClick)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Click {
    #[parameter]
    pub position: Scriptable<Option<Point>>,

    #[parameter]
    pub button: Scriptable<MouseButton>,

    #[parameter]
    pub relative_position: Scriptable<bool>,

    #[parameter(min = Some(0), max = Some(i32::MAX as i64))]
    pub amount: Scriptable<Option<i64>>,

    #[parameter]
    pub interval: Scriptable<Option<DurationValue>>,

    #[parameter]
    pub duration: Scriptable<Option<DurationValue>>,
}

impl Branching for Click {}
