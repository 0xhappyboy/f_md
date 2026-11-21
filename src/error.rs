//! Error types for the Markdown document engine.
#[derive(Debug)]
pub enum FMdError {
    /// I/O failure when reading or writing files.
    Io(std::io::Error),
    /// The Markdown content is malformed or cannot be processed.
    Parse(String),
    /// The requested operation is not valid for the current document state.
    Invalid(String),
}
impl std::fmt::Display for FMdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FMdError::Io(e) => write!(f, "io error: {}", e),
            FMdError::Parse(e) => write!(f, "parse error: {}", e),
            FMdError::Invalid(e) => write!(f, "invalid operation: {}", e),
        }
    }
}
impl std::error::Error for FMdError {}
impl From<std::io::Error> for FMdError {
    fn from(e: std::io::Error) -> Self {
        FMdError::Io(e)
    }
}
/// Convenience alias for Results returned by this crate.
pub type FMdResult<T> = Result<T, FMdError>;
