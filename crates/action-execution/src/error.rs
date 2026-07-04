use action_definition::tree::{self, BranchKind, NodeId};

use crate::resolve_param::ResolveParamError;

#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct RunError {
    pub node_id: Option<NodeId>,

    #[source]
    pub kind: RunErrorKind,
}

impl From<ResolveParamError> for RunError {
    fn from(value: ResolveParamError) -> Self {
        Self::new(value)
    }
}

impl From<eyre::Report> for RunError {
    fn from(value: eyre::Report) -> Self {
        Self::new(value)
    }
}

impl RunError {
    pub fn new(kind: impl Into<RunErrorKind>) -> Self {
        Self {
            node_id: None,
            kind: kind.into(),
        }
    }

    pub fn at_node(mut self, node_id: NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RunErrorKind {
    #[error(transparent)]
    ResolveParam(#[from] ResolveParamError),
    #[error(transparent)]
    Run(#[from] eyre::Report),
    #[error("canceled")]
    Canceled,
    #[error(transparent)]
    Tree(#[from] tree::Error),
    #[error("no node found with label {0}")]
    LabelNotFound(String),
    #[error("expected children of {0:?} to be branches")]
    ExpectedChildBranches(NodeId),
    #[error("no {0} branch found in {1:?}'s children")]
    BranchNotFound(BranchKind, NodeId),
}
