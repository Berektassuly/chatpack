//! Output configuration for message export.
//!
//! This module provides [`OutputConfig`] for controlling which metadata fields
//! are included when exporting messages to CSV, JSON, or JSONL formats.
//!
//! # Overview
//!
//! By default, only `sender` and `content` are included in output. Use the
//! builder pattern to selectively enable additional fields:
//!
//! | Method | Field | Description |
//! |--------|-------|-------------|
//! | [`with_timestamps`](OutputConfig::with_timestamps) | `timestamp` | When message was sent |
//! | [`with_ids`](OutputConfig::with_ids) | `id` | Platform-specific message ID |
//! | [`with_replies`](OutputConfig::with_replies) | `reply_to` | Parent message reference |
//! | [`with_edited`](OutputConfig::with_edited) | `edited` | Last edit timestamp |
//!
//! # Examples
//!
//! ```
//! use chatpack::core::models::OutputConfig;
//!
//! // Minimal output (sender + content only)
//! let minimal = OutputConfig::new();
//! assert!(!minimal.has_any());
//!
//! // Include timestamps
//! let with_time = OutputConfig::new().with_timestamps();
//!
//! // Include everything
//! let full = OutputConfig::all();
//! assert!(full.include_timestamps);
//! assert!(full.include_ids);
//! ```

use serde::{Deserialize, Serialize};

/// Controls which message fields are included in output.
///
/// Used by [`write_csv`](crate::core::output::write_csv),
/// [`write_json`](crate::core::output::write_json), and
/// [`write_jsonl`](crate::core::output::write_jsonl) to determine
/// which optional fields to include.
///
/// # Default Behavior
///
/// By default, only `sender` and `content` are included. This produces
/// the most compact output, optimal for LLM context windows.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "csv-output")]
/// # fn main() -> chatpack::Result<()> {
/// use chatpack::prelude::*;
///
/// let messages = vec![Message::new("Alice", "Hello!")];
///
/// // Minimal output
/// write_csv(&messages, "minimal.csv", &OutputConfig::new())?;
///
/// // With timestamps and IDs
/// let config = OutputConfig::new()
///     .with_timestamps()
///     .with_ids();
/// write_csv(&messages, "detailed.csv", &config)?;
///
/// // Everything
/// write_csv(&messages, "full.csv", &OutputConfig::all())?;
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "csv-output"))]
/// # fn main() {}
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Include message timestamps in output.
    ///
    /// Timestamps are formatted as RFC 3339 in JSON, ISO 8601 in CSV.
    pub include_timestamps: bool,

    /// Include platform-specific message IDs in output.
    pub include_ids: bool,

    /// Include reply-to references in output.
    ///
    /// Useful for reconstructing conversation threads.
    pub include_replies: bool,

    /// Include edit timestamps in output.
    ///
    /// Shows when messages were last modified.
    pub include_edited: bool,
}

impl OutputConfig {
    /// Creates a new output configuration with all options disabled.
    ///
    /// Only sender and content will be included in the output.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a configuration that includes all available metadata.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::core::models::OutputConfig;
    ///
    /// let config = OutputConfig::all();
    /// assert!(config.include_timestamps);
    /// assert!(config.include_ids);
    /// assert!(config.include_replies);
    /// assert!(config.include_edited);
    /// ```
    pub fn all() -> Self {
        Self {
            include_timestamps: true,
            include_ids: true,
            include_replies: true,
            include_edited: true,
        }
    }

    /// Enable timestamp inclusion in output.
    #[must_use]
    pub fn with_timestamps(mut self) -> Self {
        self.include_timestamps = true;
        self
    }

    /// Enable message ID inclusion in output.
    #[must_use]
    pub fn with_ids(mut self) -> Self {
        self.include_ids = true;
        self
    }

    /// Enable reply reference inclusion in output.
    #[must_use]
    pub fn with_replies(mut self) -> Self {
        self.include_replies = true;
        self
    }

    /// Enable edited timestamp inclusion in output.
    #[must_use]
    pub fn with_edited(mut self) -> Self {
        self.include_edited = true;
        self
    }

    /// Returns `true` if any metadata option is enabled.
    pub fn has_any(&self) -> bool {
        self.include_timestamps || self.include_ids || self.include_replies || self.include_edited
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_config_builder() {
        let config = OutputConfig::new().with_timestamps().with_ids();

        assert!(config.include_timestamps);
        assert!(config.include_ids);
        assert!(!config.include_replies);
        assert!(!config.include_edited);
    }

    #[test]
    fn test_output_config_all() {
        let config = OutputConfig::all();
        assert!(config.include_timestamps);
        assert!(config.include_ids);
        assert!(config.include_replies);
        assert!(config.include_edited);
    }

    #[test]
    fn test_output_config_has_any() {
        assert!(!OutputConfig::new().has_any());
        assert!(OutputConfig::new().with_timestamps().has_any());
    }
}
