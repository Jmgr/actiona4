use std::iter;

use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{Param, value::Value},
    tree::BranchKind,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SwitchCase {
    pub name: String,
    pub value: Value,
}

impl SwitchCase {
    pub fn new(name: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Chooses a branch whose case value matches the input value.
#[action(icon = CodeXml, effect = ControlFlow, category = Flow)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Switch {
    #[parameter]
    pub value: Value,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cases: Vec<SwitchCase>,
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
