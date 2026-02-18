use crate::core::node::{NodeId, NodeKey};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Node not found")]
    NodeNotFound(NodeId),
    #[error("Node not found by key")]
    NodeNotFoundByKey(NodeKey),
    #[error("The tree has no root node")]
    NoRootNode,
    #[error("TinySkia Renderer error: {0}")]
    TinySkiaRendererError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
