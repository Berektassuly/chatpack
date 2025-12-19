//! Core data models for chat messages.

use chrono::{DateTime, Utc};

/// Universal message representation for all chat sources.
/// All parsers convert their native format into this structure.
#[derive(Debug, Clone)]
pub struct InternalMessage {
    /// Message sender name
    pub sender: String,
    /// Message text content
    pub content: String,
    /// Message timestamp (if available)
    pub timestamp: Option<DateTime<Utc>>,
    /// Message ID (if available)
    pub id: Option<u64>,
    /// ID of the message this is replying to (if available)
    pub reply_to: Option<u64>,
}

impl InternalMessage {
    /// Creates a new message with only sender and content.
    /// Other fields are set to None.
    pub fn new(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            sender: sender.into(),
            content: content.into(),
            timestamp: None,
            id: None,
            reply_to: None,
        }
    }

    /// Creates a new message with all fields.
    pub fn with_metadata(
        sender: impl Into<String>,
        content: impl Into<String>,
        timestamp: Option<DateTime<Utc>>,
        id: Option<u64>,
        reply_to: Option<u64>,
    ) -> Self {
        Self {
            sender: sender.into(),
            content: content.into(),
            timestamp,
            id,
            reply_to,
        }
    }

    /// Builder-style method to set timestamp.
    pub fn timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = Some(ts);
        self
    }

    /// Builder-style method to set message ID.
    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    /// Builder-style method to set reply reference.
    pub fn reply_to(mut self, reply_id: u64) -> Self {
        self.reply_to = Some(reply_id);
        self
    }
}

/// Configuration for output format.
/// Controls which metadata fields are included in the output.
#[derive(Debug, Clone, Default)]
pub struct OutputConfig {
    /// Include timestamps in output
    pub include_timestamps: bool,
    /// Include message IDs in output
    pub include_ids: bool,
    /// Include reply references in output
    pub include_replies: bool,
}

impl OutputConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timestamps(mut self) -> Self {
        self.include_timestamps = true;
        self
    }

    pub fn with_ids(mut self) -> Self {
        self.include_ids = true;
        self
    }

    pub fn with_replies(mut self) -> Self {
        self.include_replies = true;
        self
    }
}
