use macros::ActionEnum;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::duration::DurationValue,
    post_run::PostRun,
    scriptable::Scriptable,
    tree::BranchKind,
};

#[derive(ActionEnum, Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WaitUnit {
    Milliseconds,
    #[default]
    Seconds,
    Minutes,
    Hours,
    Days,
}

#[action(
    icon = TestTubeDiagonal,
    effect = ControlFlow,
    category = Flow,
    timeout = true,
    waitable = true
)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Wait {
    #[parameter(min = Some(0.0))]
    pub duration: Scriptable<f64>,

    #[parameter]
    pub unit: Scriptable<WaitUnit>,
}

impl Default for Wait {
    fn default() -> Self {
        Self {
            duration: Scriptable::Static { value: 500.0 }.into(),
            unit: Scriptable::Static {
                value: WaitUnit::Milliseconds,
            }
            .into(),
        }
    }
}

impl ActionBranches for Wait {}

impl ParameterAvailability for Wait {}
