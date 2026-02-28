use crate::core::node::NodeId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Node not found")]
    NodeNotFound(NodeId),
    #[error("Renderer error: {0}")] // TODO: Make this universal
    D2DRendererError(windows_result::Error),
    #[error("Renderer error: {0}")] // TODO: Make this universal
    GenericRendererError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
