use serde::{Deserialize, Serialize};
use types::Point;

use crate::{
    actions::{Action, Branching},
    scriptable::Scriptable,
};

#[derive(Action, Clone, Debug, Default, Deserialize, Serialize)]
#[action(icon = MousePointerClick)]
pub struct Click {
    #[parameter]
    pub position: Scriptable<Point>,
}

impl Branching for Click {}
