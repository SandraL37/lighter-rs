use crate::core::node::NodeId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Node not found")]
    NodeNotFound(NodeId),
    #[error("The tree has no root node")]
    NoRootNode,
    #[error("TinySkia Renderer error: {0}")]
    TinySkiaRendererError(String),
    #[error("Piet Renderer error: {0}")]
    PietRendererError(piet_common::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
