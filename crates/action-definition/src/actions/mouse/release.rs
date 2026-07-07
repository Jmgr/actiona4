use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, MouseButton, ParameterAvailability, action},
    scriptable::Scriptable,
};

#[action(icon = MousePointerClick, effect = ChangeState, category = Mouse)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Release {
    #[parameter]
    pub button: Scriptable<Option<MouseButton>>,
}

impl ActionBranches for Release {}

impl ParameterAvailability for Release {}
