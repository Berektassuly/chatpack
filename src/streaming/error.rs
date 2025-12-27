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

    /// JSON parsing error (only available with JSON-using parsers)
    #[cfg(any(feature = "telegram", feature = "instagram", feature = "discord"))]
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
            #[cfg(any(feature = "telegram", feature = "instagram", feature = "discord"))]
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
            #[cfg(any(feature = "telegram", feature = "instagram", feature = "discord"))]
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

#[cfg(any(feature = "telegram", feature = "instagram", feature = "discord"))]
impl From<serde_json::Error> for StreamingError {
    fn from(err: serde_json::Error) -> Self {
        StreamingError::Json(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Display tests
    // =========================================================================

    #[test]
    fn test_error_display_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = StreamingError::Io(io_err);
        let display = err.to_string();
        assert!(display.contains("IO error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_error_display_invalid_format() {
        let err = StreamingError::InvalidFormat("missing messages array".into());
        assert!(err.to_string().contains("Invalid format"));
        assert!(err.to_string().contains("missing messages array"));
    }

    #[test]
    fn test_error_display_unexpected_eof() {
        let err = StreamingError::UnexpectedEof;
        assert!(err.to_string().contains("Unexpected end of file"));
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
        assert!(msg.contains("too large"));
    }

    #[cfg(any(feature = "telegram", feature = "instagram", feature = "discord"))]
    #[test]
    fn test_error_display_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = StreamingError::Json(json_err);
        assert!(err.to_string().contains("JSON error"));
    }

    // =========================================================================
    // From conversions tests
    // =========================================================================

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let streaming_err: StreamingError = io_err.into();
        assert!(matches!(streaming_err, StreamingError::Io(_)));
    }

    #[cfg(any(feature = "telegram", feature = "instagram", feature = "discord"))]
    #[test]
    fn test_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let streaming_err: StreamingError = json_err.into();
        assert!(matches!(streaming_err, StreamingError::Json(_)));
    }

    // =========================================================================
    // Error source tests
    // =========================================================================

    #[test]
    fn test_error_source_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = StreamingError::Io(io_err);
        assert!(err.source().is_some());
    }

    #[cfg(any(feature = "telegram", feature = "instagram", feature = "discord"))]
    #[test]
    fn test_error_source_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = StreamingError::Json(json_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_error_source_none() {
        let err = StreamingError::InvalidFormat("test".into());
        assert!(err.source().is_none());

        let err = StreamingError::UnexpectedEof;
        assert!(err.source().is_none());

        let err = StreamingError::BufferOverflow {
            max_size: 1024,
            actual_size: 2048,
        };
        assert!(err.source().is_none());
    }

    // =========================================================================
    // Debug tests
    // =========================================================================

    #[test]
    fn test_error_debug() {
        let err = StreamingError::InvalidFormat("test".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("InvalidFormat"));
        assert!(debug.contains("test"));
    }
}
