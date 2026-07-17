use action_definition::tree::{self, BranchKind, NodeId};
use actiona_core::{api::js::DeepEqualError, scripting::ScriptError};

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

impl From<ScriptError> for RunError {
    fn from(value: ScriptError) -> Self {
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

    #[must_use]
    pub const fn at_node(mut self, node_id: NodeId) -> Self {
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
    #[error("{action} has no inputs")]
    EmptyWaitInputs { action: &'static str },
    #[error("{action} cannot be used as an And/Or input")]
    NonWaitableInput { action: &'static str },
    #[error("{action} must be inside a loop")]
    LoopControlOutsideLoop { action: &'static str },
    #[error(transparent)]
    Tree(#[from] tree::Error),
    #[error("no node found with label {0}")]
    LabelNotFound(String),
    #[error("invalid wait unit")]
    InvalidWaitUnit { value: String },
    #[error("invalid wait duration")]
    InvalidWaitDuration,
    #[error("random number minimum {minimum} must be less than maximum {maximum}")]
    InvalidRandomNumberRange { minimum: f64, maximum: f64 },
    #[error("random integer minimum {minimum} must not exceed maximum {maximum}")]
    InvalidRandomIntegerRange { minimum: i64, maximum: i64 },
    #[error("random string characters must not be empty when length is greater than zero")]
    EmptyRandomCharacters,
    #[error("random item array must not be empty")]
    EmptyRandomItem,
    #[error("failed to resolve switch branch `{branch}` value: {source}")]
    SwitchBranchValueResolveFailed {
        branch: String,
        #[source]
        source: ScriptError,
    },
    #[error("failed to compare switch value with branch `{branch}`: {source}")]
    SwitchBranchCompareFailed {
        branch: String,
        #[source]
        source: DeepEqualError,
    },
    #[error("expected children of {0:?} to be branches")]
    ExpectedChildBranches(NodeId),
    #[error("no {0} branch found in {1:?}'s children")]
    BranchNotFound(BranchKind, NodeId),
    #[error(transparent)]
    Script(#[from] ScriptError),
}
