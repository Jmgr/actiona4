use serde::{Deserialize, Serialize};

use crate::{
    actions::{Action, Branching},
    parameters::source_code::SourceCode,
    tree::BranchKind,
};

#[derive(Action, Clone, Debug, Default, Deserialize, Serialize)]
#[action(icon = CodeXml)]
pub struct Code {
    #[parameter]
    pub source: SourceCode,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    branches: Vec<String>,
}

impl Branching for Code {
    fn branches(&self) -> Vec<BranchKind> {
        self.branches
            .iter()
            .map(|branch| BranchKind::Named(branch.clone()))
            .collect()
    }
}
