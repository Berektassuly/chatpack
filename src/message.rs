//! Core message type for chatpack.
//!
//! This module provides the [`Message`] type, the universal representation
//! for chat messages from all supported platforms.
//!
//! # Example
//!
//! ```rust
//! use chatpack::Message;
//! use chrono::Utc;
//!
//! // Create a simple message
//! let msg = Message::new("Alice", "Hello, world!");
//!
//! // Create with builder pattern
//! let msg_with_meta = Message::new("Bob", "Hi there!")
//!     .with_id(12345)
//!     .with_timestamp(Utc::now());
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A chat message with optional metadata.
///
/// This is the universal message representation used across all chat sources.
/// All parsers convert their native format into this structure, enabling
/// uniform processing regardless of the original chat platform.
///
/// # Fields
///
/// - `sender` and `content` are always present
/// - `timestamp`, `id`, `reply_to`, and `edited` are optional metadata
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
///
/// # Example
///
/// ```rust
/// use chatpack::Message;
/// use chrono::Utc;
///
/// let msg = Message::new("Alice", "Hello!")
///     .with_timestamp(Utc::now())
///     .with_id(12345);
///
/// assert_eq!(msg.sender(), "Alice");
/// assert_eq!(msg.content(), "Hello!");
/// assert!(msg.id().is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
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

impl Message {
    /// Creates a new message with only sender and content.
    ///
    /// All metadata fields (timestamp, id, `reply_to`, edited) are set to `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::Message;
    ///
    /// let msg = Message::new("Alice", "Hello!");
    /// assert_eq!(msg.sender(), "Alice");
    /// assert_eq!(msg.content(), "Hello!");
    /// assert!(msg.timestamp().is_none());
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

    // =========================================================================
    // Builder methods
    // =========================================================================

    /// Builder method to set the timestamp.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::Message;
    /// use chrono::Utc;
    ///
    /// let msg = Message::new("Alice", "Hello")
    ///     .with_timestamp(Utc::now());
    /// assert!(msg.timestamp().is_some());
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
    /// use chatpack::Message;
    ///
    /// let msg = Message::new("Alice", "Hello")
    ///     .with_id(12345);
    /// assert_eq!(msg.id(), Some(12345));
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
    /// use chatpack::Message;
    ///
    /// let msg = Message::new("Bob", "I agree!")
    ///     .with_reply_to(12344);
    /// assert_eq!(msg.reply_to(), Some(12344));
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
    /// use chatpack::Message;
    /// use chrono::Utc;
    ///
    /// let msg = Message::new("Alice", "Updated message")
    ///     .with_edited(Utc::now());
    /// assert!(msg.edited().is_some());
    /// ```
    #[must_use]
    pub fn with_edited(mut self, ts: DateTime<Utc>) -> Self {
        self.edited = Some(ts);
        self
    }

    // =========================================================================
    // Accessor methods
    // =========================================================================

    /// Returns the sender name.
    pub fn sender(&self) -> &str {
        &self.sender
    }

    /// Returns the message content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the timestamp, if available.
    pub fn timestamp(&self) -> Option<DateTime<Utc>> {
        self.timestamp
    }

    /// Returns the message ID, if available.
    pub fn id(&self) -> Option<u64> {
        self.id
    }

    /// Returns the reply-to ID, if available.
    pub fn reply_to(&self) -> Option<u64> {
        self.reply_to
    }

    /// Returns the edited timestamp, if available.
    pub fn edited(&self) -> Option<DateTime<Utc>> {
        self.edited
    }

    // =========================================================================
    // Utility methods
    // =========================================================================

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

impl Default for Message {
    fn default() -> Self {
        Self::new("", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_message_new() {
        let msg = Message::new("Alice", "Hello");
        assert_eq!(msg.sender(), "Alice");
        assert_eq!(msg.content(), "Hello");
        assert!(msg.timestamp().is_none());
        assert!(msg.id().is_none());
    }

    #[test]
    fn test_message_builder() {
        let ts = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
        let msg = Message::new("Alice", "Hello")
            .with_timestamp(ts)
            .with_id(123)
            .with_reply_to(122)
            .with_edited(ts);

        assert_eq!(msg.timestamp(), Some(ts));
        assert_eq!(msg.id(), Some(123));
        assert_eq!(msg.reply_to(), Some(122));
        assert_eq!(msg.edited(), Some(ts));
    }

    #[test]
    fn test_message_has_metadata() {
        let msg1 = Message::new("Alice", "Hello");
        assert!(!msg1.has_metadata());

        let msg2 = Message::new("Alice", "Hello").with_id(123);
        assert!(msg2.has_metadata());
    }

    #[test]
    fn test_message_is_empty() {
        assert!(Message::new("Alice", "").is_empty());
        assert!(Message::new("Alice", "   ").is_empty());
        assert!(!Message::new("Alice", "Hello").is_empty());
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::new("Alice", "Hello").with_id(123);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Alice"));
        assert!(json.contains("123"));
        // timestamp should be skipped (None)
        assert!(!json.contains("timestamp"));
    }

    #[test]
    fn test_message_deserialization() {
        let json = r#"{"sender":"Bob","content":"Hi","id":456}"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert_eq!(msg.sender(), "Bob");
        assert_eq!(msg.content(), "Hi");
        assert_eq!(msg.id(), Some(456));
        assert!(msg.timestamp().is_none());
    }

    #[test]
    fn test_message_accessors() {
        let ts = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
        let msg = Message::new("Alice", "Hello")
            .with_timestamp(ts)
            .with_id(123)
            .with_reply_to(122)
            .with_edited(ts);

        assert_eq!(msg.sender(), "Alice");
        assert_eq!(msg.content(), "Hello");
        assert_eq!(msg.timestamp(), Some(ts));
        assert_eq!(msg.id(), Some(123));
        assert_eq!(msg.reply_to(), Some(122));
        assert_eq!(msg.edited(), Some(ts));
    }
}
