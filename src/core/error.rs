use crate::core::node::NodeId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Node not found")]
    NodeNotFound(NodeId),
}

pub type Result<T> = std::result::Result<T, Error>;
