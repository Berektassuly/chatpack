//! Unified error types for chatpack.
//!
//! This module provides a single [`ChatpackError`] enum that covers all error
//! cases in the library. This design follows the pattern used by popular crates
//! like `reqwest`, `serde_json`, and `csv`.
//!
//! # Error Handling Philosophy
//!
//! - **Library users** get typed errors they can match on
//! - **Application users** get clear, actionable error messages
//! - **Developers** get source error chains for debugging
//!
//! # Example
//!
//! ```rust
//! use chatpack::error::{ChatpackError, Result};
//! use chatpack::parsers::create_parser;
//! use chatpack::cli::Source;
//!
//! fn process_chat(path: &str) -> Result<()> {
//!     let parser = create_parser(Source::Telegram);
//!     let messages = parser.parse(path)?;
//!     
//!     // Handle specific errors
//!     // match result {
//!     //     Err(ChatpackError::Io(e)) => eprintln!("File error: {}", e),
//!     //     Err(ChatpackError::Parse { source, .. }) => eprintln!("Parse error: {}", source),
//!     //     _ => {}
//!     // }
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Error Categories
//!
//! | Category | When it occurs |
//! |----------|----------------|
//! | [`Io`](ChatpackError::Io) | File operations fail |
//! | [`Parse`](ChatpackError::Parse) | JSON/format parsing fails |
//! | [`InvalidFormat`](ChatpackError::InvalidFormat) | File structure doesn't match expected format |
//! | [`InvalidDate`](ChatpackError::InvalidDate) | Date filter has wrong format |
//! | [`Csv`](ChatpackError::Csv) | CSV writing fails |
//! | [`Streaming`](ChatpackError::Streaming) | Streaming parser errors |

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// A specialized [`Result`] type for chatpack operations.
///
/// This type is broadly used across the library for any operation that
/// may produce an error.
///
/// # Example
///
/// ```rust
/// use chatpack::error::Result;
/// use chatpack::core::InternalMessage;
///
/// fn my_function() -> Result<Vec<InternalMessage>> {
///     // ... operations that may fail
///     Ok(vec![])
/// }
/// ```
pub type Result<T> = std::result::Result<T, ChatpackError>;

/// The error type for all chatpack operations.
///
/// This enum represents all possible errors that can occur when using chatpack.
/// Each variant contains context about what went wrong and, where applicable,
/// the underlying source error.
///
/// # Matching on Errors
///
/// ```rust,ignore
/// use chatpack::error::ChatpackError;
///
/// match error {
///     ChatpackError::Io(e) => {
///         eprintln!("IO error: {}", e);
///     }
///     ChatpackError::Parse { format, source, path } => {
///         eprintln!("Failed to parse {} file: {}", format, source);
///         if let Some(p) = path {
///             eprintln!("  File: {}", p.display());
///         }
///     }
///     ChatpackError::InvalidFormat { format, message } => {
///         eprintln!("{} format error: {}", format, message);
///     }
///     _ => eprintln!("Error: {}", error),
/// }
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub enum ChatpackError {
    /// An I/O error occurred.
    ///
    /// This typically happens when:
    /// - The input file doesn't exist
    /// - Permission denied
    /// - Disk is full (when writing output)
    Io(io::Error),

    /// Failed to parse the input file.
    ///
    /// Contains the format being parsed, the underlying parse error,
    /// and optionally the file path.
    Parse {
        /// The format being parsed (e.g., "Telegram JSON", "WhatsApp TXT")
        format: &'static str,
        /// The underlying parse error
        source: ParseErrorKind,
        /// The file path, if available
        path: Option<PathBuf>,
    },

    /// The file format doesn't match the expected structure.
    ///
    /// This occurs when:
    /// - Telegram JSON is missing the "messages" array
    /// - WhatsApp TXT doesn't match any known date format
    /// - Discord export is in an unrecognized format
    InvalidFormat {
        /// The format that was expected
        format: &'static str,
        /// Description of what's wrong
        message: String,
    },

    /// Invalid date format in filter configuration.
    ///
    /// Date filters expect YYYY-MM-DD format.
    InvalidDate {
        /// The invalid date string that was provided
        input: String,
        /// Expected format description
        expected: &'static str,
    },

    /// CSV writing error.
    ///
    /// This can occur when writing output to CSV format.
    Csv(csv::Error),

    /// Streaming parser error.
    ///
    /// Errors specific to streaming parsers for large files.
    Streaming(StreamingErrorKind),

    /// UTF-8 encoding error.
    ///
    /// Occurs when file content is not valid UTF-8.
    Utf8 {
        /// Description of where the error occurred
        context: String,
        /// The underlying UTF-8 error
        source: std::string::FromUtf8Error,
    },

    /// Buffer overflow in streaming parser.
    ///
    /// A single message exceeded the maximum allowed size.
    BufferOverflow {
        /// Maximum allowed size in bytes
        max_size: usize,
        /// Actual size encountered
        actual_size: usize,
    },

    /// Unexpected end of file.
    ///
    /// The file ended before parsing was complete.
    UnexpectedEof {
        /// Context about what was being parsed
        context: String,
    },
}

/// Kinds of parse errors that can occur.
#[derive(Debug)]
pub enum ParseErrorKind {
    /// JSON parsing error
    Json(serde_json::Error),
    /// Regex/pattern matching error
    Pattern(String),
    /// Generic parsing error
    Other(String),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::Json(e) => write!(f, "{}", e),
            ParseErrorKind::Pattern(s) => write!(f, "{}", s),
            ParseErrorKind::Other(s) => write!(f, "{}", s),
        }
    }
}

impl Error for ParseErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseErrorKind::Json(e) => Some(e),
            _ => None,
        }
    }
}

/// Kinds of streaming errors.
#[derive(Debug)]
pub enum StreamingErrorKind {
    /// IO error during streaming
    Io(io::Error),
    /// JSON parsing error during streaming
    Json(serde_json::Error),
    /// Invalid format encountered
    InvalidFormat(String),
    /// Buffer overflow
    BufferOverflow { max_size: usize, actual_size: usize },
    /// Unexpected EOF
    UnexpectedEof,
}

impl fmt::Display for StreamingErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamingErrorKind::Io(e) => write!(f, "IO error: {}", e),
            StreamingErrorKind::Json(e) => write!(f, "JSON error: {}", e),
            StreamingErrorKind::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            StreamingErrorKind::BufferOverflow {
                max_size,
                actual_size,
            } => write!(
                f,
                "Buffer overflow: {} bytes (max: {})",
                actual_size, max_size
            ),
            StreamingErrorKind::UnexpectedEof => write!(f, "Unexpected end of file"),
        }
    }
}

impl Error for StreamingErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StreamingErrorKind::Io(e) => Some(e),
            StreamingErrorKind::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ChatpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChatpackError::Io(e) => write!(f, "IO error: {}", e),

            ChatpackError::Parse {
                format,
                source,
                path,
            } => {
                write!(f, "Failed to parse {} export", format)?;
                if let Some(p) = path {
                    write!(f, " (file: {})", p.display())?;
                }
                write!(f, ": {}", source)
            }

            ChatpackError::InvalidFormat { format, message } => {
                write!(f, "Invalid {} format: {}", format, message)
            }

            ChatpackError::InvalidDate { input, expected } => {
                write!(
                    f,
                    "Invalid date '{}'. Expected format: {}",
                    input, expected
                )
            }

            ChatpackError::Csv(e) => write!(f, "CSV error: {}", e),

            ChatpackError::Streaming(e) => write!(f, "Streaming error: {}", e),

            ChatpackError::Utf8 { context, source } => {
                write!(f, "UTF-8 encoding error in {}: {}", context, source)
            }

            ChatpackError::BufferOverflow {
                max_size,
                actual_size,
            } => {
                write!(
                    f,
                    "Message too large: {} bytes (maximum: {} bytes)",
                    actual_size, max_size
                )
            }

            ChatpackError::UnexpectedEof { context } => {
                write!(f, "Unexpected end of file while {}", context)
            }
        }
    }
}

impl Error for ChatpackError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ChatpackError::Io(e) => Some(e),
            ChatpackError::Parse { source, .. } => Some(source),
            ChatpackError::Csv(e) => Some(e),
            ChatpackError::Streaming(e) => Some(e),
            ChatpackError::Utf8 { source, .. } => Some(source),
            _ => None,
        }
    }
}

// ============================================================================
// From implementations for ergonomic error conversion
// ============================================================================

impl From<io::Error> for ChatpackError {
    fn from(err: io::Error) -> Self {
        ChatpackError::Io(err)
    }
}

impl From<serde_json::Error> for ChatpackError {
    fn from(err: serde_json::Error) -> Self {
        ChatpackError::Parse {
            format: "JSON",
            source: ParseErrorKind::Json(err),
            path: None,
        }
    }
}

impl From<csv::Error> for ChatpackError {
    fn from(err: csv::Error) -> Self {
        ChatpackError::Csv(err)
    }
}

impl From<std::string::FromUtf8Error> for ChatpackError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ChatpackError::Utf8 {
            context: "output conversion".to_string(),
            source: err,
        }
    }
}

// ============================================================================
// Convenience constructors
// ============================================================================

impl ChatpackError {
    /// Creates a parse error for Telegram format.
    pub fn telegram_parse(source: serde_json::Error, path: Option<PathBuf>) -> Self {
        ChatpackError::Parse {
            format: "Telegram JSON",
            source: ParseErrorKind::Json(source),
            path,
        }
    }

    /// Creates a parse error for WhatsApp format.
    pub fn whatsapp_parse(message: impl Into<String>, path: Option<PathBuf>) -> Self {
        ChatpackError::Parse {
            format: "WhatsApp TXT",
            source: ParseErrorKind::Pattern(message.into()),
            path,
        }
    }

    /// Creates a parse error for Instagram format.
    pub fn instagram_parse(source: serde_json::Error, path: Option<PathBuf>) -> Self {
        ChatpackError::Parse {
            format: "Instagram JSON",
            source: ParseErrorKind::Json(source),
            path,
        }
    }

    /// Creates a parse error for Discord format.
    pub fn discord_parse(source: serde_json::Error, path: Option<PathBuf>) -> Self {
        ChatpackError::Parse {
            format: "Discord",
            source: ParseErrorKind::Json(source),
            path,
        }
    }

    /// Creates an invalid format error.
    pub fn invalid_format(format: &'static str, message: impl Into<String>) -> Self {
        ChatpackError::InvalidFormat {
            format,
            message: message.into(),
        }
    }

    /// Creates an invalid date error.
    pub fn invalid_date(input: impl Into<String>) -> Self {
        ChatpackError::InvalidDate {
            input: input.into(),
            expected: "YYYY-MM-DD",
        }
    }

    /// Creates a streaming error from components.
    pub fn streaming(kind: StreamingErrorKind) -> Self {
        ChatpackError::Streaming(kind)
    }

    /// Creates a buffer overflow error.
    pub fn buffer_overflow(max_size: usize, actual_size: usize) -> Self {
        ChatpackError::BufferOverflow {
            max_size,
            actual_size,
        }
    }

    /// Creates an unexpected EOF error.
    pub fn unexpected_eof(context: impl Into<String>) -> Self {
        ChatpackError::UnexpectedEof {
            context: context.into(),
        }
    }

    /// Returns `true` if this is an IO error.
    pub fn is_io(&self) -> bool {
        matches!(self, ChatpackError::Io(_))
    }

    /// Returns `true` if this is a parse error.
    pub fn is_parse(&self) -> bool {
        matches!(self, ChatpackError::Parse { .. })
    }

    /// Returns `true` if this is an invalid format error.
    pub fn is_invalid_format(&self) -> bool {
        matches!(self, ChatpackError::InvalidFormat { .. })
    }

    /// Returns `true` if this is a date-related error.
    pub fn is_invalid_date(&self) -> bool {
        matches!(self, ChatpackError::InvalidDate { .. })
    }
}

// ============================================================================
// Integration with streaming module
// ============================================================================

impl From<crate::streaming::StreamingError> for ChatpackError {
    fn from(err: crate::streaming::StreamingError) -> Self {
        match err {
            crate::streaming::StreamingError::Io(e) => {
                ChatpackError::Streaming(StreamingErrorKind::Io(e))
            }
            crate::streaming::StreamingError::Json(e) => {
                ChatpackError::Streaming(StreamingErrorKind::Json(e))
            }
            crate::streaming::StreamingError::InvalidFormat(s) => {
                ChatpackError::Streaming(StreamingErrorKind::InvalidFormat(s))
            }
            crate::streaming::StreamingError::BufferOverflow {
                max_size,
                actual_size,
            } => ChatpackError::Streaming(StreamingErrorKind::BufferOverflow {
                max_size,
                actual_size,
            }),
            crate::streaming::StreamingError::UnexpectedEof => {
                ChatpackError::Streaming(StreamingErrorKind::UnexpectedEof)
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = ChatpackError::from(io_err);
        let display = err.to_string();
        assert!(display.contains("IO error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_parse_error_with_path() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = ChatpackError::Parse {
            format: "Telegram JSON",
            source: ParseErrorKind::Json(json_err),
            path: Some(PathBuf::from("/path/to/file.json")),
        };
        let display = err.to_string();
        assert!(display.contains("Telegram JSON"));
        assert!(display.contains("/path/to/file.json"));
    }

    #[test]
    fn test_invalid_date_display() {
        let err = ChatpackError::invalid_date("not-a-date");
        let display = err.to_string();
        assert!(display.contains("not-a-date"));
        assert!(display.contains("YYYY-MM-DD"));
    }

    #[test]
    fn test_buffer_overflow_display() {
        let err = ChatpackError::buffer_overflow(1024, 2048);
        let display = err.to_string();
        assert!(display.contains("2048"));
        assert!(display.contains("1024"));
    }

    #[test]
    fn test_error_source_chain() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err = ChatpackError::from(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_is_methods() {
        let io_err = ChatpackError::Io(io::Error::new(io::ErrorKind::NotFound, ""));
        assert!(io_err.is_io());
        assert!(!io_err.is_parse());

        let date_err = ChatpackError::invalid_date("bad");
        assert!(date_err.is_invalid_date());
        assert!(!date_err.is_io());
    }

    #[test]
    fn test_convenience_constructors() {
        let err = ChatpackError::invalid_format("WhatsApp", "could not detect date format");
        assert!(err.is_invalid_format());
        assert!(err.to_string().contains("WhatsApp"));

        let err = ChatpackError::unexpected_eof("reading message array");
        let display = err.to_string();
        assert!(display.contains("reading message array"));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        fn returns_error() -> Result<i32> {
            Err(ChatpackError::invalid_date("bad"))
        }

        assert!(returns_result().is_ok());
        assert!(returns_error().is_err());
    }
}