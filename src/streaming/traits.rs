//! Core traits for streaming parsers.
//!
//! This module defines the trait hierarchy for memory-efficient streaming:
//! - [`MessageIterator`] - Iterator with progress tracking
//! - [`StreamingParser`] - Parser that produces iterators
//! - [`StreamingConfig`] - Configuration options

use crate::Message;
use crate::error::ChatpackError;

use super::StreamingResult;

/// Iterator over messages from a streaming parser with progress tracking.
///
/// Extends the standard [`Iterator`] trait with methods for monitoring
/// parsing progress, useful for progress bars and logging.
///
/// # Object Safety
///
/// This trait is object-safe, enabling dynamic dispatch via `Box<dyn MessageIterator>`.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "telegram")]
/// # fn main() -> chatpack::Result<()> {
/// use chatpack::streaming::{StreamingParser, TelegramStreamingParser, MessageIterator};
///
/// let parser = TelegramStreamingParser::new();
/// let mut iter = parser.stream("export.json")?;
///
/// while let Some(result) = iter.next() {
///     let msg = result?;
///
///     // Check progress periodically
///     if let Some(pct) = iter.progress() {
///         eprintln!("\r{:.1}%", pct);
///     }
/// }
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "telegram"))]
/// # fn main() {}
/// ```
pub trait MessageIterator: Iterator<Item = StreamingResult<Message>> + Send {
    /// Returns approximate progress as a percentage (0.0 to 100.0).
    ///
    /// Returns `None` if progress cannot be determined (e.g., unknown file size).
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

/// A parser that streams messages from files without loading everything into memory.
///
/// Unlike standard parsers that load the entire file, `StreamingParser` produces
/// an iterator that yields messages one at a time, enabling processing of
/// arbitrarily large files with constant memory usage.
///
/// # Implementation Guidelines
///
/// Implementors should:
/// - Use buffered I/O (64KB - 1MB buffers)
/// - Skip malformed records gracefully
/// - Track bytes processed for progress reporting
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "telegram")]
/// # fn main() -> chatpack::Result<()> {
/// use chatpack::streaming::{StreamingParser, TelegramStreamingParser};
///
/// let parser = TelegramStreamingParser::new();
/// let messages: Vec<_> = parser
///     .stream("export.json")?
///     .filter_map(Result::ok)
///     .collect();
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "telegram"))]
/// # fn main() {}
/// ```
pub trait StreamingParser: Send + Sync {
    /// Returns the human-readable name of this parser.
    fn name(&self) -> &'static str;

    /// Opens a file and returns an iterator over messages.
    ///
    /// # Errors
    ///
    /// Returns [`ChatpackError::Io`] if the file cannot be opened.
    fn stream(&self, file_path: &str) -> Result<Box<dyn MessageIterator>, ChatpackError>;

    /// Returns the recommended buffer size for this parser.
    ///
    /// Default: 64KB
    fn recommended_buffer_size(&self) -> usize {
        64 * 1024 // 64KB default
    }

    /// Returns `true` if this parser supports progress reporting.
    fn supports_progress(&self) -> bool {
        true
    }
}

/// Configuration options for streaming parsers.
///
/// Controls buffer sizes, error handling, and progress reporting behavior.
///
/// # Examples
///
/// ```
/// use chatpack::streaming::StreamingConfig;
///
/// let config = StreamingConfig::new()
///     .with_buffer_size(128 * 1024)  // 128KB buffer
///     .with_skip_invalid(false);      // Return errors instead of skipping
/// ```
#[derive(Debug, Clone, Copy)]
pub struct StreamingConfig {
    /// Buffer size for file reading.
    ///
    /// Default: 64KB. Larger buffers improve throughput but use more memory.
    pub buffer_size: usize,

    /// Maximum size of a single message in bytes.
    ///
    /// Default: 10MB. Messages exceeding this are skipped or error.
    pub max_message_size: usize,

    /// Whether to skip invalid messages or return errors.
    ///
    /// Default: `true` (skip). Set to `false` for strict validation.
    pub skip_invalid: bool,

    /// Report progress every N messages.
    ///
    /// Default: 10,000. Lower values provide more frequent updates.
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

    // =========================================================================
    // StreamingConfig tests
    // =========================================================================

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.buffer_size, 64 * 1024);
        assert_eq!(config.max_message_size, 10 * 1024 * 1024);
        assert!(config.skip_invalid);
        assert_eq!(config.progress_interval, 10_000);
    }

    #[test]
    fn test_streaming_config_new() {
        let config = StreamingConfig::new();
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
    fn test_streaming_config_with_progress_interval() {
        let config = StreamingConfig::new().with_progress_interval(5000);
        assert_eq!(config.progress_interval, 5000);
    }

    #[test]
    fn test_streaming_config_builder_chain() {
        let config = StreamingConfig::new()
            .with_buffer_size(256 * 1024)
            .with_max_message_size(20 * 1024 * 1024)
            .with_skip_invalid(false)
            .with_progress_interval(1000);

        assert_eq!(config.buffer_size, 256 * 1024);
        assert_eq!(config.max_message_size, 20 * 1024 * 1024);
        assert!(!config.skip_invalid);
        assert_eq!(config.progress_interval, 1000);
    }

    #[test]
    fn test_streaming_config_copy() {
        let config = StreamingConfig::new();
        let copied = config; // Copy
        assert_eq!(config.buffer_size, copied.buffer_size);
        assert_eq!(config.max_message_size, copied.max_message_size);
        assert_eq!(config.skip_invalid, copied.skip_invalid);
    }

    #[test]
    fn test_streaming_config_clone() {
        let config = StreamingConfig::new().with_buffer_size(512 * 1024);
        let cloned = config;
        assert_eq!(config.buffer_size, cloned.buffer_size);
    }

    #[test]
    fn test_streaming_config_debug() {
        let config = StreamingConfig::new();
        let debug = format!("{:?}", config);
        assert!(debug.contains("StreamingConfig"));
        assert!(debug.contains("buffer_size"));
    }
}
