use std::default;

use serde::{Deserialize, Serialize};

use crate::tree::BranchKind;

/// What to do once the action has finished running.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum PostRun {
    /// Run the next sibling.
    #[default]
    NextSibling,

    /// Run the first child, if any, otherwise run the parent's next sibling.
    /// This is typically used by root and branch actions.
    NextChild,

    /// Run the branch.
    Branch(BranchKind),

    /// Jump to a label.
    GotoLabel(String),

    /// Stop the execution.
    Stop,

    /// Exit the application.
    Exit,

    /// Leave the nearest enclosing loop.
    Break,

    /// Skip to the next iteration of the nearest enclosing loop.
    Continue,
    // TODO: call function
}
