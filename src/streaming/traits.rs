//! Core traits for streaming parsers.

use crate::error::ChatpackError;
use crate::Message;

use super::StreamingResult;

/// Iterator over messages from a streaming parser.
///
/// This trait is object-safe and allows for dynamic dispatch,
/// enabling runtime selection of streaming parsers.
pub trait MessageIterator: Iterator<Item = StreamingResult<Message>> + Send {
    /// Returns approximate progress as a percentage (0.0 - 100.0).
    ///
    /// Returns `None` if progress cannot be determined.
    fn progress(&self) -> Option<f64> {
        None
    }

    /// Returns the number of bytes processed so far.
    fn bytes_processed(&self) -> u64;

    /// Returns the total file size in bytes, if known.
    fn total_bytes(&self) -> Option<u64> {
        None
    }
}

/// A parser that can stream messages from large files.
///
/// Unlike [`ChatParser`], which loads everything into memory,
/// `StreamingParser` produces an iterator that yields messages one at a time.
///
/// # Implementation Notes
///
/// Implementors should:
/// - Use buffered I/O with reasonable buffer sizes (64KB - 1MB)
/// - Handle malformed records gracefully (skip and continue)
/// - Provide progress reporting when possible
///
/// [`ChatParser`]: crate::parsers::ChatParser
pub trait StreamingParser: Send + Sync {
    /// Returns the name of this parser.
    fn name(&self) -> &'static str;

    /// Opens a file and returns an iterator over messages.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or has invalid format.
    fn stream(&self, file_path: &str) -> Result<Box<dyn MessageIterator>, ChatpackError>;

    /// Returns the recommended buffer size for this parser.
    fn recommended_buffer_size(&self) -> usize {
        64 * 1024 // 64KB default
    }

    /// Returns true if this parser supports progress reporting.
    fn supports_progress(&self) -> bool {
        true
    }
}

/// Configuration for streaming parsers.
#[derive(Debug, Clone, Copy)]
pub struct StreamingConfig {
    /// Buffer size for reading (default: 64KB)
    pub buffer_size: usize,

    /// Maximum size of a single message in bytes (default: 10MB)
    pub max_message_size: usize,

    /// Whether to skip invalid messages or return errors (default: skip)
    pub skip_invalid: bool,

    /// Report progress every N messages (default: 10000)
    pub progress_interval: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64 * 1024,             // 64KB
            max_message_size: 10 * 1024 * 1024, // 10MB
            skip_invalid: true,
            progress_interval: 10_000,
        }
    }
}

impl StreamingConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the buffer size.
    #[must_use]
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Sets the maximum message size.
    #[must_use]
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Sets whether to skip invalid messages.
    #[must_use]
    pub fn with_skip_invalid(mut self, skip: bool) -> Self {
        self.skip_invalid = skip;
        self
    }

    /// Sets the progress reporting interval.
    #[must_use]
    pub fn with_progress_interval(mut self, interval: usize) -> Self {
        self.progress_interval = interval;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.buffer_size, 64 * 1024);
        assert_eq!(config.max_message_size, 10 * 1024 * 1024);
        assert!(config.skip_invalid);
    }

    #[test]
    fn test_streaming_config_builder() {
        let config = StreamingConfig::new()
            .with_buffer_size(128 * 1024)
            .with_max_message_size(1024)
            .with_skip_invalid(false);

        assert_eq!(config.buffer_size, 128 * 1024);
        assert_eq!(config.max_message_size, 1024);
        assert!(!config.skip_invalid);
    }

    #[test]
    fn test_streaming_config_copy() {
        let config = StreamingConfig::new();
        let copied = config; // Copy
        assert_eq!(config.buffer_size, copied.buffer_size);
    }
}
