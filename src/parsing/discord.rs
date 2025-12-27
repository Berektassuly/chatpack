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

    // =========================================================================
    // parse_discord_message tests
    // =========================================================================

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
    fn test_parse_discord_message_with_stickers() {
        let msg = DiscordRawMessage {
            id: "123".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: None,
            content: "Look at this sticker".to_string(),
            author: DiscordAuthor {
                name: "charlie".to_string(),
                nickname: None,
            },
            reference: None,
            attachments: None,
            stickers: Some(vec![DiscordSticker {
                name: "CoolSticker".to_string(),
            }]),
        };

        let result = parse_discord_message(&msg);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert!(parsed.content.contains("[Sticker: CoolSticker]"));
    }

    #[test]
    fn test_parse_discord_message_sticker_only() {
        // Empty content but with sticker should be kept
        let msg = DiscordRawMessage {
            id: "123".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: None,
            content: String::new(),
            author: DiscordAuthor {
                name: "charlie".to_string(),
                nickname: None,
            },
            reference: None,
            attachments: None,
            stickers: Some(vec![DiscordSticker {
                name: "Reaction".to_string(),
            }]),
        };

        let result = parse_discord_message(&msg);
        assert!(result.is_some());
        assert!(result.unwrap().content.contains("[Sticker: Reaction]"));
    }

    #[test]
    fn test_parse_discord_message_attachment_only() {
        // Empty content but with attachment should be kept
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
            attachments: Some(vec![DiscordAttachment {
                file_name: "photo.jpg".to_string(),
            }]),
            stickers: None,
        };

        let result = parse_discord_message(&msg);
        assert!(result.is_some());
        assert!(result.unwrap().content.contains("[Attachment: photo.jpg]"));
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

    #[test]
    fn test_parse_discord_message_with_reply() {
        let msg = DiscordRawMessage {
            id: "456".to_string(),
            timestamp: "2024-01-15T10:31:00+00:00".to_string(),
            timestamp_edited: None,
            content: "This is a reply".to_string(),
            author: DiscordAuthor {
                name: "alice".to_string(),
                nickname: None,
            },
            reference: Some(DiscordReference {
                message_id: Some("123".to_string()),
            }),
            attachments: None,
            stickers: None,
        };

        let result = parse_discord_message(&msg);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert_eq!(parsed.reply_to, Some(123));
    }

    #[test]
    fn test_parse_discord_message_with_edited() {
        let msg = DiscordRawMessage {
            id: "123".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: Some("2024-01-15T10:35:00+00:00".to_string()),
            content: "Edited message".to_string(),
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
        assert!(parsed.edited.is_some());
    }

    // =========================================================================
    // parse_discord_stream_message tests
    // =========================================================================

    #[test]
    fn test_parse_stream_message_basic() {
        let msg = DiscordStreamMessage {
            id: "123456789".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: None,
            content: "Hello from stream".to_string(),
            author: DiscordAuthor {
                name: "alice".to_string(),
                nickname: None,
            },
            reference: None,
        };

        let result = parse_discord_stream_message(&msg);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert_eq!(parsed.sender, "alice");
        assert_eq!(parsed.content, "Hello from stream");
        assert!(parsed.timestamp.is_some());
        assert_eq!(parsed.id, Some(123456789));
    }

    #[test]
    fn test_parse_stream_message_with_nickname() {
        let msg = DiscordStreamMessage {
            id: "123".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: None,
            content: "Hi".to_string(),
            author: DiscordAuthor {
                name: "alice123".to_string(),
                nickname: Some("Alice Display".to_string()),
            },
            reference: None,
        };

        let result = parse_discord_stream_message(&msg);
        assert!(result.is_some());
        assert_eq!(result.unwrap().sender, "Alice Display");
    }

    #[test]
    fn test_parse_stream_message_empty() {
        let msg = DiscordStreamMessage {
            id: "123".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: None,
            content: "   ".to_string(), // Whitespace only
            author: DiscordAuthor {
                name: "bob".to_string(),
                nickname: None,
            },
            reference: None,
        };

        assert!(parse_discord_stream_message(&msg).is_none());
    }

    #[test]
    fn test_parse_stream_message_with_reply() {
        let msg = DiscordStreamMessage {
            id: "456".to_string(),
            timestamp: "2024-01-15T10:31:00+00:00".to_string(),
            timestamp_edited: None,
            content: "Reply to something".to_string(),
            author: DiscordAuthor {
                name: "charlie".to_string(),
                nickname: None,
            },
            reference: Some(DiscordReference {
                message_id: Some("789".to_string()),
            }),
        };

        let result = parse_discord_stream_message(&msg);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert_eq!(parsed.reply_to, Some(789));
    }

    #[test]
    fn test_parse_stream_message_with_edited() {
        let msg = DiscordStreamMessage {
            id: "123".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: Some("2024-01-15T10:40:00+00:00".to_string()),
            content: "Edited stream message".to_string(),
            author: DiscordAuthor {
                name: "alice".to_string(),
                nickname: None,
            },
            reference: None,
        };

        let result = parse_discord_stream_message(&msg);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert!(parsed.edited.is_some());
    }

    #[test]
    fn test_parse_stream_message_invalid_id() {
        let msg = DiscordStreamMessage {
            id: "not_a_number".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: None,
            content: "Has invalid ID".to_string(),
            author: DiscordAuthor {
                name: "alice".to_string(),
                nickname: None,
            },
            reference: None,
        };

        let result = parse_discord_stream_message(&msg);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert!(parsed.id.is_none()); // Should be None since parsing fails
    }

    #[test]
    fn test_parse_stream_message_reference_without_id() {
        let msg = DiscordStreamMessage {
            id: "123".to_string(),
            timestamp: "2024-01-15T10:30:00+00:00".to_string(),
            timestamp_edited: None,
            content: "Has reference but no id".to_string(),
            author: DiscordAuthor {
                name: "alice".to_string(),
                nickname: None,
            },
            reference: Some(DiscordReference { message_id: None }),
        };

        let result = parse_discord_stream_message(&msg);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert!(parsed.reply_to.is_none());
    }

    // =========================================================================
    // Serde deserialization tests
    // =========================================================================

    #[test]
    fn test_discord_export_deserialize() {
        let json = r#"{
            "messages": [
                {
                    "id": "1",
                    "timestamp": "2024-01-15T10:30:00+00:00",
                    "content": "Hello",
                    "author": {"name": "alice"}
                }
            ]
        }"#;

        let export: DiscordExport = serde_json::from_str(json).expect("deserialize");
        assert_eq!(export.messages.len(), 1);
        assert_eq!(export.messages[0].content, "Hello");
    }

    #[test]
    fn test_discord_stream_message_deserialize() {
        let json = r#"{
            "id": "12345",
            "timestamp": "2024-01-15T10:30:00+00:00",
            "content": "Stream test",
            "author": {"name": "bob", "nickname": "Bobby"}
        }"#;

        let msg: DiscordStreamMessage = serde_json::from_str(json).expect("deserialize");
        assert_eq!(msg.id, "12345");
        assert_eq!(msg.content, "Stream test");
        assert_eq!(msg.author.name, "bob");
        assert_eq!(msg.author.nickname, Some("Bobby".to_string()));
    }
}
