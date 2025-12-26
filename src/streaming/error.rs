//! Error types for streaming parsers.

use std::error::Error;
use std::fmt;
use std::io;

/// Result type for streaming operations.
pub type StreamingResult<T> = Result<T, StreamingError>;

/// Errors that can occur during streaming parsing.
#[derive(Debug)]
pub enum StreamingError {
    /// IO error while reading file
    Io(io::Error),

    /// JSON parsing error
    Json(serde_json::Error),

    /// Invalid file format (missing expected structure)
    InvalidFormat(String),

    /// Unexpected end of file
    UnexpectedEof,

    /// Buffer overflow (message too large)
    BufferOverflow { max_size: usize, actual_size: usize },
}

impl fmt::Display for StreamingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamingError::Io(e) => write!(f, "IO error: {e}"),
            StreamingError::Json(e) => write!(f, "JSON error: {e}"),
            StreamingError::InvalidFormat(msg) => write!(f, "Invalid format: {msg}"),
            StreamingError::UnexpectedEof => write!(f, "Unexpected end of file"),
            StreamingError::BufferOverflow {
                max_size,
                actual_size,
            } => {
                write!(
                    f,
                    "Message too large: {actual_size} bytes (max: {max_size})"
                )
            }
        }
    }
}

impl Error for StreamingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StreamingError::Io(e) => Some(e),
            StreamingError::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for StreamingError {
    fn from(err: io::Error) -> Self {
        StreamingError::Io(err)
    }
}

impl From<serde_json::Error> for StreamingError {
    fn from(err: serde_json::Error) -> Self {
        StreamingError::Json(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = StreamingError::InvalidFormat("missing messages array".into());
        assert!(err.to_string().contains("Invalid format"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let streaming_err: StreamingError = io_err.into();
        assert!(matches!(streaming_err, StreamingError::Io(_)));
    }

    #[test]
    fn test_buffer_overflow_display() {
        let err = StreamingError::BufferOverflow {
            max_size: 1024,
            actual_size: 2048,
        };
        let msg = err.to_string();
        assert!(msg.contains("2048"));
        assert!(msg.contains("1024"));
    }
}
