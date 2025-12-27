//! Core data models for chat output configuration.
//!
//! This module provides the [`OutputConfig`] type for configuring output generation.
//!
//! # Example
//!
//! ```rust
//! use chatpack::Message;
//! use chatpack::core::models::OutputConfig;
//! use chrono::Utc;
//!
//! // Create a simple message
//! let msg = Message::new("Alice", "Hello, world!");
//!
//! // Create with builder pattern
//! let msg_with_meta = Message::new("Bob", "Hi there!")
//!     .with_id(12345)
//!     .with_timestamp(Utc::now());
//!
//! // Configure output
//! let config = OutputConfig::new()
//!     .with_timestamps()
//!     .with_replies();
//! ```

use serde::{Deserialize, Serialize};

/// Configuration for output format.
///
/// Controls which metadata fields are included in the output when writing
/// to CSV, JSON, or JSONL formats.
///
/// # Example
///
/// ```rust
/// use chatpack::core::models::OutputConfig;
///
/// // Default: only sender and content
/// let minimal = OutputConfig::new();
///
/// // Include all available metadata
/// let full = OutputConfig::new()
///     .with_timestamps()
///     .with_ids()
///     .with_replies()
///     .with_edited();
///
/// // Or use the convenience method
/// let full = OutputConfig::all();
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Include timestamps in output
    pub include_timestamps: bool,

    /// Include message IDs in output
    pub include_ids: bool,

    /// Include reply references in output
    pub include_replies: bool,

    /// Include edited timestamps in output
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
