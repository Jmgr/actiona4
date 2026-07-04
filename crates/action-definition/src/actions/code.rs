use serde::{Deserialize, Serialize};

use crate::{
    actions::{Branching, action},
    parameters::{Param, source_code::SourceCode},
    tree::BranchKind,
};

#[action(icon = CodeXml)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Code {
    #[parameter]
    pub source: SourceCode,

    /// User defined branches
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    branches: Vec<String>,
}

impl Code {
    pub fn new(source: impl Into<SourceCode>) -> Self {
        Self {
            source: Param::new(source.into()),
            branches: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_branches(mut self, branches: Vec<String>) -> Self {
        self.branches = branches;
        self
    }
}

impl Branching for Code {
    fn branches(&self) -> Vec<BranchKind> {
        self.branches
            .iter()
            .map(|branch| BranchKind::Named(branch.clone()))
            .collect()
    }
}
