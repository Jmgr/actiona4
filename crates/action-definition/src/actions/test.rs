use serde::{Deserialize, Serialize};

use crate::{
    actions::{Branching, action},
    parameters::duration::DurationValue,
    post_run::PostRun,
    scriptable::Scriptable,
    tree::BranchKind,
};

#[action(icon = TestTubeDiagonal)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Test {
    #[parameter]
    pub percent: Scriptable<i64>,

    #[parameter]
    pub duration: Scriptable<DurationValue>,

    #[serde(skip)]
    pub post_run: PostRun,
}

impl Branching for Test {
    fn branches(&self) -> Vec<BranchKind> {
        vec![BranchKind::True, BranchKind::False]
    }
}
