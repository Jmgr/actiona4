use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{Param, duration::DurationValue},
    scriptable::Scriptable,
};

/// Waits until a condition becomes true.
#[action(
    icon = TestTubeDiagonal,
    effect = ControlFlow,
    category = Flow,
    timeout = true,
    waitable = true
)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WaitUntil {
    #[parameter]
    pub condition: Scriptable<bool>,

    #[parameter]
    pub poll_interval: Scriptable<DurationValue>,
}

impl Default for WaitUntil {
    fn default() -> Self {
        Self {
            condition: Param::default(),
            poll_interval: Scriptable::new_static(Duration::from_millis(200)).into(),
        }
    }
}

impl ActionBranches for WaitUntil {}

impl ParameterAvailability for WaitUntil {}
