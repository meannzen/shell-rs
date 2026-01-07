use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShellError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("{0}")]
    CommandNotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}
