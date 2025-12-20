//! Core data models for chat messages.
//!
//! This module provides the universal message representation used across all
//! chat sources. All parsers convert their native format into [`InternalMessage`].
//!
//! # Example
//!
//! ```rust
//! use chatpack::core::models::{InternalMessage, OutputConfig};
//! use chrono::Utc;
//!
//! // Create a simple message
//! let msg = InternalMessage::new("Alice", "Hello, world!");
//!
//! // Create with builder pattern
//! let msg_with_meta = InternalMessage::new("Bob", "Hi there!")
//!     .with_id(12345)
//!     .with_timestamp(Utc::now());
//!
//! // Configure output
//! let config = OutputConfig::new()
//!     .with_timestamps()
//!     .with_replies();
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Universal message representation for all chat sources.
///
/// All parsers convert their native format into this structure.
/// This allows uniform processing regardless of the original chat platform.
///
/// # Fields
///
/// All fields are public for direct access. Use the builder methods
/// for a more ergonomic construction pattern.
///
/// # Serialization
///
/// The struct implements both `Serialize` and `Deserialize`, making it
/// suitable for:
/// - Saving/loading processed messages
/// - Inter-process communication
/// - Integration with other systems (RAG pipelines, databases, etc.)
///
/// Optional fields are skipped during serialization when `None`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalMessage {
    /// Message sender name/username
    pub sender: String,

    /// Message text content
    pub content: String,

    /// Message timestamp (if available from source)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,

    /// Platform-specific message ID (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub id: Option<u64>,

    /// ID of the message this is replying to (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub reply_to: Option<u64>,

    /// Timestamp when message was last edited (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub edited: Option<DateTime<Utc>>,
}

impl InternalMessage {
    /// Creates a new message with only sender and content.
    ///
    /// All metadata fields (timestamp, id, `reply_to`, edited) are set to `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::core::models::InternalMessage;
    ///
    /// let msg = InternalMessage::new("Alice", "Hello!");
    /// assert_eq!(msg.sender, "Alice");
    /// assert_eq!(msg.content, "Hello!");
    /// assert!(msg.timestamp.is_none());
    /// ```
    pub fn new(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            sender: sender.into(),
            content: content.into(),
            timestamp: None,
            id: None,
            reply_to: None,
            edited: None,
        }
    }

    /// Creates a new message with all fields specified.
    ///
    /// Use this when you have all metadata available upfront.
    /// For incremental construction, prefer [`new`](Self::new) with builder methods.
    pub fn with_metadata(
        sender: impl Into<String>,
        content: impl Into<String>,
        timestamp: Option<DateTime<Utc>>,
        id: Option<u64>,
        reply_to: Option<u64>,
        edited: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            sender: sender.into(),
            content: content.into(),
            timestamp,
            id,
            reply_to,
            edited,
        }
    }

    /// Builder method to set the timestamp.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::core::models::InternalMessage;
    /// use chrono::Utc;
    ///
    /// let msg = InternalMessage::new("Alice", "Hello")
    ///     .with_timestamp(Utc::now());
    /// assert!(msg.timestamp.is_some());
    /// ```
    #[must_use]
    pub fn with_timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = Some(ts);
        self
    }

    /// Builder method to set the message ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::core::models::InternalMessage;
    ///
    /// let msg = InternalMessage::new("Alice", "Hello")
    ///     .with_id(12345);
    /// assert_eq!(msg.id, Some(12345));
    /// ```
    #[must_use]
    pub fn with_id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    /// Builder method to set the reply reference.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::core::models::InternalMessage;
    ///
    /// let msg = InternalMessage::new("Bob", "I agree!")
    ///     .with_reply_to(12344);
    /// assert_eq!(msg.reply_to, Some(12344));
    /// ```
    #[must_use]
    pub fn with_reply_to(mut self, reply_id: u64) -> Self {
        self.reply_to = Some(reply_id);
        self
    }

    /// Builder method to set the edited timestamp.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::core::models::InternalMessage;
    /// use chrono::Utc;
    ///
    /// let msg = InternalMessage::new("Alice", "Updated message")
    ///     .with_edited(Utc::now());
    /// assert!(msg.edited.is_some());
    /// ```
    #[must_use]
    pub fn with_edited(mut self, ts: DateTime<Utc>) -> Self {
        self.edited = Some(ts);
        self
    }

    // === Legacy builder methods (for backward compatibility) ===

    /// Sets the timestamp. Alias for [`with_timestamp`](Self::with_timestamp).
    #[must_use]
    #[doc(hidden)]
    pub fn timestamp(self, ts: DateTime<Utc>) -> Self {
        self.with_timestamp(ts)
    }

    /// Sets the message ID. Alias for [`with_id`](Self::with_id).
    #[must_use]
    #[doc(hidden)]
    pub fn id(self, id: u64) -> Self {
        self.with_id(id)
    }

    /// Sets the reply reference. Alias for [`with_reply_to`](Self::with_reply_to).
    #[must_use]
    #[doc(hidden)]
    pub fn reply_to(self, reply_id: u64) -> Self {
        self.with_reply_to(reply_id)
    }

    /// Sets the edited timestamp. Alias for [`with_edited`](Self::with_edited).
    #[must_use]
    #[doc(hidden)]
    pub fn edited(self, ts: DateTime<Utc>) -> Self {
        self.with_edited(ts)
    }

    /// Returns `true` if this message has any metadata (timestamp, id, `reply_to`, or edited).
    pub fn has_metadata(&self) -> bool {
        self.timestamp.is_some()
            || self.id.is_some()
            || self.reply_to.is_some()
            || self.edited.is_some()
    }

    /// Returns `true` if this message's content is empty or whitespace-only.
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }
}

impl Default for InternalMessage {
    fn default() -> Self {
        Self::new("", "")
    }
}

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
    use chrono::TimeZone;

    #[test]
    fn test_message_new() {
        let msg = InternalMessage::new("Alice", "Hello");
        assert_eq!(msg.sender, "Alice");
        assert_eq!(msg.content, "Hello");
        assert!(msg.timestamp.is_none());
        assert!(msg.id.is_none());
    }

    #[test]
    fn test_message_builder() {
        let ts = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
        let msg = InternalMessage::new("Alice", "Hello")
            .with_timestamp(ts)
            .with_id(123)
            .with_reply_to(122)
            .with_edited(ts);

        assert_eq!(msg.timestamp, Some(ts));
        assert_eq!(msg.id, Some(123));
        assert_eq!(msg.reply_to, Some(122));
        assert_eq!(msg.edited, Some(ts));
    }

    #[test]
    fn test_message_has_metadata() {
        let msg1 = InternalMessage::new("Alice", "Hello");
        assert!(!msg1.has_metadata());

        let msg2 = InternalMessage::new("Alice", "Hello").with_id(123);
        assert!(msg2.has_metadata());
    }

    #[test]
    fn test_message_is_empty() {
        assert!(InternalMessage::new("Alice", "").is_empty());
        assert!(InternalMessage::new("Alice", "   ").is_empty());
        assert!(!InternalMessage::new("Alice", "Hello").is_empty());
    }

    #[test]
    fn test_message_serialization() {
        let msg = InternalMessage::new("Alice", "Hello").with_id(123);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Alice"));
        assert!(json.contains("123"));
        // timestamp should be skipped (None)
        assert!(!json.contains("timestamp"));
    }

    #[test]
    fn test_message_deserialization() {
        let json = r#"{"sender":"Bob","content":"Hi","id":456}"#;
        let msg: InternalMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.sender, "Bob");
        assert_eq!(msg.content, "Hi");
        assert_eq!(msg.id, Some(456));
        assert!(msg.timestamp.is_none());
    }

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
