use std::default;

use serde::{Deserialize, Serialize};

use crate::tree::BranchKind;

/// What to do once the action has finished running.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum PostRun {
    /// Run the next sibling.
    #[default]
    NextSibling,

    /// Run the first child, if any, otherwise run the parent's next sibling.
    /// This is typically used by root and branch actions.
    NextChild, // TODO: do we need this

    /// Run the branch.
    Branch(BranchKind),

    /// Jump to a label.
    GotoLabel(String),

    /// Stop the execution.
    Stop,
    // TODO: call function
    // TODO: loop
}
