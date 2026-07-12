use std::iter;

use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{
        Param,
        labelled_branches::{self, LabelledBranch, LabelledBranches},
        value::Value,
    },
    tree::BranchKind,
};

/// Chooses a branch whose case value matches the input value.
#[action(icon = CodeXml, effect = ControlFlow, category = Flow)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Switch {
    #[parameter]
    pub value: Value,

    #[parameter]
    pub cases: LabelledBranches,
}

impl ActionBranches for Switch {
    fn action_branches(&self) -> Vec<BranchKind> {
        // A Default branch and one branch per case.
        iter::once(BranchKind::Default)
            .chain(
                self.cases
                    .iter()
                    .map(|branch| BranchKind::Named(branch.name.clone())),
            )
            .collect()
    }
}

impl ParameterAvailability for Switch {}
