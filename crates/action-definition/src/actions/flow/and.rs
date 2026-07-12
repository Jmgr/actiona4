use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::action_list::ActionList,
};

/// Waits until every input action has completed.
#[action(icon = CornerDownRight, effect = ControlFlow, category = Flow, timeout = true)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct And {
    #[parameter]
    pub inputs: ActionList,
}

impl ActionBranches for And {}

impl ParameterAvailability for And {}
