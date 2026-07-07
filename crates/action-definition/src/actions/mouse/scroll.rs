use macros::ActionEnum;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    scriptable::Scriptable,
};

#[derive(ActionEnum, Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Axis {
    Horizontal,
    #[default]
    Vertical,
}

#[action(icon = MousePointerClick, effect = ChangeState, category = Mouse)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Scroll {
    #[parameter]
    pub amount: Scriptable<i64>,

    #[parameter]
    pub axis: Scriptable<Axis>,
}

impl ActionBranches for Scroll {}

impl ParameterAvailability for Scroll {}
