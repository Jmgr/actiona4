use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::duration::DurationValue,
    post_run::PostRun,
    scriptable::Scriptable,
    tree::BranchKind,
};

/// Provides configurable behavior for action-definition and runner tests.
#[action(icon = TestTubeDiagonal, effect = ControlFlow, category = Flow)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Test {
    #[parameter]
    pub percent: Scriptable<i64>,

    #[parameter]
    pub duration: Scriptable<DurationValue>,

    #[serde(skip)]
    pub post_run: PostRun,
}

impl Default for Test {
    fn default() -> Self {
        Self {
            percent: Scriptable::Static { value: 50 }.into(),
            duration: Default::default(),
            post_run: Default::default(),
        }
    }
}

impl ActionBranches for Test {
    fn action_branches(&self) -> Vec<BranchKind> {
        vec![BranchKind::True, BranchKind::False]
    }
}

impl ParameterAvailability for Test {}
