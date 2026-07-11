use serde::{Deserialize, Serialize};

use crate::actions::{ActionBranches, ActionInstance, ParameterAvailability, action};

#[action(icon = CornerDownRight, effect = ControlFlow, category = Flow, timeout = true)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct And {
    pub inputs: Vec<ActionInstance>,
}

impl ActionBranches for And {}

impl ParameterAvailability for And {}
