use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ActionInstance, ParameterAvailability, action},
    tree::BranchKind,
};

/// Waits until one input completes, then continues through that input's
/// positional handler branch.
#[action(icon = CornerDownRight, effect = ControlFlow, category = Flow, timeout = true)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Or {
    pub inputs: Vec<ActionInstance>,
}

impl ActionBranches for Or {
    fn action_branches(&self) -> Vec<BranchKind> {
        self.inputs
            .iter()
            .enumerate()
            .map(|(index, _)| BranchKind::Named(index.to_string()))
            .collect()
    }
}

impl ParameterAvailability for Or {}
