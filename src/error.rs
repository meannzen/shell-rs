#[derive(Debug)]
pub enum ShellError {
    Io(std::io::Error),
    ParseError(String),
    CommandNotFound(String),
    PermissionDenied(String),
    InternalError(String),
}

impl std::error::Error for ShellError {}

impl From<std::io::Error> for ShellError {
    fn from(value: std::io::Error) -> Self {
        ShellError::Io(value)
    }
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
