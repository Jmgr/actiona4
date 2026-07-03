use super::NodeId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("node {0:?} cannot have children")]
    NodeCannotHaveChildren(NodeId),
    #[error("node {0:?} cannot be removed")]
    NodeCannotBeRemoved(NodeId),
    #[error("cannot find node with ID {0:?}")]
    InvalidNode(NodeId),
    #[error("label {0} already exists")]
    DuplicateLabel(String),
    #[error("drop is not allowed")]
    DropNotAllowed,
    #[error("node {0:?} is not an action")]
    NotAnAction(NodeId),
    #[error("unknown parameter {0:?}")]
    UnknownParameter(String),
    #[error("parameter (de)serialization failed: {0}")]
    ParameterSerialization(String),
}
