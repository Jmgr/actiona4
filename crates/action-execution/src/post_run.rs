use action_definition::tree::BranchKind;

/// What to do once the action has finished running.
#[derive(Clone, Debug, PartialEq)]
pub enum PostRun {
    /// Run the next sibling.
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
