use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Windows API error: {0}")]
    WindowsApiError(#[from] windows_result::Error),
    #[error("Unknown windows error: {0}")]
    UnknownWindowsError(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
