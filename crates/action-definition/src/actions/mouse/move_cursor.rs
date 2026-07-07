use macros::{ActionEnum, action};
use serde::{Deserialize, Serialize};
use types::Point;

use crate::{
    actions::{ActionBranches, ParameterAvailability},
    parameters::duration::DurationValue,
    scriptable::Scriptable,
};

#[derive(ActionEnum, Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tween {
    BackIn,
    BackInOut,
    BackOut,
    BounceIn,
    BounceInOut,
    BounceOut,
    CircIn,
    CircInOut,
    CircOut,
    CubicIn,
    CubicInOut,
    CubicOut,
    ElasticIn,
    ElasticInOut,
    ElasticOut,
    ExpoIn,
    ExpoInOut,
    ExpoOut,
    Linear,
    QuadIn,
    QuadInOut,
    QuadOut,
    QuartIn,
    QuartInOut,
    QuartOut,
    QuintIn,
    QuintInOut,
    QuintOut,
    SineIn,
    SineInOut,
    #[default]
    SineOut,
}

#[action(icon = MousePointer2, effect = ChangeState, category = Mouse, timeout = true)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MoveCursor {
    #[parameter]
    pub position: Scriptable<Point>,

    #[parameter]
    pub speed: Scriptable<Option<f64>>,

    #[parameter]
    pub tween: Scriptable<Option<Tween>>,

    #[parameter]
    pub perlin_scale: Scriptable<Option<f64>>,

    #[parameter]
    pub perlin_amplitude: Scriptable<Option<f64>>,

    #[parameter]
    pub target_randomness: Scriptable<Option<f64>>,

    #[parameter]
    pub interval: Scriptable<Option<DurationValue>>,
}

impl ActionBranches for MoveCursor {}

impl ParameterAvailability for MoveCursor {}
