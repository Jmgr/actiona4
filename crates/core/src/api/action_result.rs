pub mod js;

/// Editor/execution control value returned by an action Code script.
///
/// This is intentionally independent from the action tree's internal post-run
/// type; action-execution maps it to its own control-flow model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionBranch {
    Yes,
    No,
    Cancel,
    True,
    False,
    Custom(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionResult {
    NextSibling,
    NextChild,
    Branch(ActionBranch),
    GotoLabel(String),
    Stop,
}
