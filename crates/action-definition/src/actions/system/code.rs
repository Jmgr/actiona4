use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{
        Param,
        branches::{self, Branches},
        source_code::SourceCode,
    },
    tree::BranchKind,
};

/// Runs source code and follows the returned action result.
#[action(icon = CodeXml, effect = ExternalSystem, category = System)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Code {
    #[parameter]
    pub source: SourceCode,

    /// User defined branches
    #[parameter]
    branches: Branches,
}

impl Code {
    pub fn new(source: impl Into<SourceCode>) -> Self {
        Self {
            source: Param::new(source.into()),
            branches: Branches::default().into(),
        }
    }

    #[must_use]
    pub fn with_branches(mut self, branches: Vec<String>) -> Self {
        self.branches = branches.into();
        self
    }
}

impl ActionBranches for Code {
    fn action_branches(&self) -> Vec<BranchKind> {
        self.branches
            .iter()
            .map(|branch| BranchKind::Named(branch.clone()))
            .collect()
    }
}

impl ParameterAvailability for Code {}
