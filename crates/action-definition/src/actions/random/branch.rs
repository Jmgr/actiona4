use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::branches::{self, Branches},
    tree::BranchKind,
};

/// Chooses one branch at random.
#[action(icon = CodeXml, effect = ControlFlow, category = Random)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RandomBranch {
    #[parameter]
    pub branches: Branches,
}

impl ActionBranches for RandomBranch {
    fn action_branches(&self) -> Vec<BranchKind> {
        self.branches
            .iter()
            .cloned()
            .map(BranchKind::Named)
            .collect()
    }
}

impl ParameterAvailability for RandomBranch {}
