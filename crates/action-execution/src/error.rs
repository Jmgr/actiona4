use action_definition::tree::{self, BranchKind, NodeId};

use crate::resolve::ResolveError;

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("resolving failed")]
    Resolve(#[from] ResolveError),
    #[error("canceled")]
    Canceled,
    #[error(transparent)]
    Tree(#[from] tree::Error), // TODO
    #[error("no node found with label {0}")]
    LabelNotFound(String),
    #[error("expected children of {0:?} to be branches")]
    ExpectedChildBranches(NodeId),
    #[error("no {0} branch found in {1:?}'s children")]
    BranchNotFound(BranchKind, NodeId),
    #[error(transparent)]
    Execution(#[from] eyre::Report),
}
