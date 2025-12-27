//! Shared Discord parsing utilities.
//!
//! This module contains types and functions shared between the standard
//! and streaming Discord parsers.

use chrono::DateTime;
use serde::Deserialize;

use crate::Message;

/// Raw Discord message structure for deserialization.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordRawMessage {
    pub id: String,
    pub timestamp: String,
    pub timestamp_edited: Option<String>,
    pub content: String,
    pub author: DiscordAuthor,
    pub reference: Option<DiscordReference>,
    pub attachments: Option<Vec<DiscordAttachment>>,
    pub stickers: Option<Vec<DiscordSticker>>,
}

/// Discord author structure.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordAuthor {
    pub name: String,
    pub nickname: Option<String>,
}

/// Discord message reference (for replies).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordReference {
    pub message_id: Option<String>,
}

/// Discord attachment structure.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordAttachment {
    pub file_name: String,
}

/// Discord sticker structure.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordSticker {
    pub name: String,
}

/// Discord export wrapper.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordExport {
    pub messages: Vec<DiscordRawMessage>,
}

/// Parses a raw Discord message into a `Message`.
///
/// Returns `None` if the message has no content and no attachments/stickers.
pub fn parse_discord_message(msg: &DiscordRawMessage) -> Option<Message> {
    // Skip empty messages without attachments/stickers
    if msg.content.trim().is_empty()
        && msg.attachments.as_ref().is_none_or(|a| a.is_empty())
        && msg.stickers.as_ref().is_none_or(|s| s.is_empty())
    {
        return None;
    }

    // Build content: text + attachment/sticker info
    let mut content = msg.content.clone();

    // Append attachment filenames
    if let Some(attachments) = &msg.attachments {
        for att in attachments {
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(&format!("[Attachment: {}]", att.file_name));
        }
    }

    // Append sticker names
    if let Some(stickers) = &msg.stickers {
        for sticker in stickers {
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(&format!("[Sticker: {}]", sticker.name));
        }
    }

    // Use nickname if available, fallback to username
    let sender = msg
        .author
        .nickname
        .as_ref()
        .unwrap_or(&msg.author.name)
        .clone();

    // Parse timestamp (ISO 8601 / RFC3339)
    let timestamp = DateTime::parse_from_rfc3339(&msg.timestamp)
        .ok()
        .map(|dt| dt.to_utc());

    // Parse edited timestamp
    let edited = msg
        .timestamp_edited
        .as_ref()
        .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
        .map(|dt| dt.to_utc());

    // Parse message ID (Discord snowflake)
    let id = msg.id.parse::<u64>().ok();

    // Parse reply reference
    let reply_to = msg
        .reference
        .as_ref()
        .and_then(|r| r.message_id.as_ref())
        .and_then(|id_str| id_str.parse::<u64>().ok());

    Some(Message::with_metadata(
        sender, content, timestamp, id, reply_to, edited,
    ))
}

/// Lightweight Discord message for streaming (without attachments/stickers).
///
/// Used by JSONL streaming where each line is a complete message.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordStreamMessage {
    pub id: String,
    pub timestamp: String,
    pub timestamp_edited: Option<String>,
    pub content: String,
    pub author: DiscordAuthor,
    pub reference: Option<DiscordReference>,
}

/// Parses a streaming Discord message (simpler, no attachments).
pub fn parse_discord_stream_message(msg: &DiscordStreamMessage) -> Option<Message> {
    if msg.content.trim().is_empty() {
        return None;
    }

    let sender = msg
        .author
        .nickname
        .as_ref()
        .unwrap_or(&msg.author.name)
        .clone();

    let timestamp = DateTime::parse_from_rfc3339(&msg.timestamp)
        .ok()
        .map(|dt| dt.to_utc());

    let edited = msg
        .timestamp_edited
        .as_ref()
        .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
        .map(|dt| dt.to_utc());

    let id = msg.id.parse::<u64>().ok();

    let reply_to = msg
        .reference
        .as_ref()
        .and_then(|r| r.message_id.as_ref())
        .and_then(|id_str| id_str.parse::<u64>().ok());

    Some(Message::with_metadata(
        sender,
        msg.content.clone(),
        timestamp,
        id,
        reply_to,
        edited,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_discord_message_basic() {
        let msg = DiscordRawMessage {
            id: "123456789".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: None,
            content: "Hello world".to_string(),
            author: DiscordAuthor {
                name: "alice".to_string(),
                nickname: None,
            },
            reference: None,
            attachments: None,
            stickers: None,
        };

        let result = parse_discord_message(&msg);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert_eq!(parsed.sender, "alice");
        assert_eq!(parsed.content, "Hello world");
        assert!(parsed.timestamp.is_some());
    }

    #[test]
    fn test_parse_discord_message_with_nickname() {
        let msg = DiscordRawMessage {
            id: "123".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: None,
            content: "Hi".to_string(),
            author: DiscordAuthor {
                name: "alice123".to_string(),
                nickname: Some("Alice".to_string()),
            },
            reference: None,
            attachments: None,
            stickers: None,
        };

        let result = parse_discord_message(&msg);
        assert!(result.is_some());
        assert_eq!(result.unwrap().sender, "Alice");
    }

    #[test]
    fn test_parse_discord_message_with_attachments() {
        let msg = DiscordRawMessage {
            id: "123".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: None,
            content: "Check this".to_string(),
            author: DiscordAuthor {
                name: "bob".to_string(),
                nickname: None,
            },
            reference: None,
            attachments: Some(vec![DiscordAttachment {
                file_name: "image.png".to_string(),
            }]),
            stickers: None,
        };

        let result = parse_discord_message(&msg);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert!(parsed.content.contains("[Attachment: image.png]"));
    }

    #[test]
    fn test_parse_discord_message_empty() {
        let msg = DiscordRawMessage {
            id: "123".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: None,
            content: String::new(),
            author: DiscordAuthor {
                name: "bob".to_string(),
                nickname: None,
            },
            reference: None,
            attachments: None,
            stickers: None,
        };

        assert!(parse_discord_message(&msg).is_none());
    }
}
