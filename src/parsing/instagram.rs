//! Shared Instagram parsing utilities.
//!
//! This module contains types and functions shared between the standard
//! and streaming Instagram parsers.

use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;

use crate::Message;

/// Raw Instagram message structure for deserialization.
#[derive(Debug, Deserialize)]
pub struct InstagramRawMessage {
    pub sender_name: String,
    pub timestamp_ms: i64,
    pub content: Option<String>,
    pub share: Option<InstagramShare>,
    #[serde(default)]
    pub photos: Option<Vec<InstagramMedia>>,
    #[serde(default)]
    pub videos: Option<Vec<InstagramMedia>>,
    #[serde(default)]
    pub audio_files: Option<Vec<InstagramMedia>>,
}

/// Instagram share structure.
#[derive(Debug, Deserialize)]
pub struct InstagramShare {
    pub share_text: Option<String>,
    pub link: Option<String>,
}

/// Instagram media (photo/video/audio) structure.
#[derive(Debug, Deserialize)]
pub struct InstagramMedia {
    pub uri: Option<String>,
}

/// Instagram export wrapper.
#[derive(Debug, Deserialize)]
pub struct InstagramExport {
    pub messages: Vec<InstagramRawMessage>,
}

/// Fix Meta's broken encoding (Mojibake).
///
/// Meta exports UTF-8 text encoded as if it were ISO-8859-1.
/// Each UTF-8 byte is stored as a separate Unicode codepoint.
/// Example: "Привет" becomes "ÐŸÑ€Ð¸Ð²ÐµÑ‚"
///
/// This function reverses that process by:
/// 1. Taking each char as its byte value
/// 2. Reconstructing the original UTF-8 string
///
/// # Example
///
/// ```ignore
/// use chatpack::parsing::instagram::fix_mojibake_encoding;
///
/// // ASCII passes through unchanged
/// assert_eq!(fix_mojibake_encoding("Hello"), "Hello");
///
/// // Mojibake gets fixed
/// // (actual mojibake text would be converted back to proper UTF-8)
/// ```
pub fn fix_mojibake_encoding(s: &str) -> String {
    let bytes: Vec<u8> = s.chars().map(|c| c as u8).collect();
    String::from_utf8(bytes).unwrap_or_else(|_| s.to_string())
}

/// Parses a millisecond timestamp to DateTime.
pub fn parse_ms_timestamp(timestamp_ms: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_millis_opt(timestamp_ms).single()
}

/// Parses a raw Instagram message into a `Message`.
///
/// Returns `None` if the message has no content.
///
/// If `fix_encoding` is true, applies Mojibake fix to sender and content.
pub fn parse_instagram_message(msg: &InstagramRawMessage, fix_encoding: bool) -> Option<Message> {
    // Get content from various possible locations
    let content = msg
        .content
        .clone()
        .or_else(|| msg.share.as_ref().and_then(|s| s.share_text.clone()));

    // Apply encoding fix if needed
    let content = content.map(|c| {
        if fix_encoding {
            fix_mojibake_encoding(&c)
        } else {
            c
        }
    });

    // Skip messages without content
    let content = match content {
        Some(c) if !c.trim().is_empty() => c,
        _ => return None,
    };

    let timestamp = parse_ms_timestamp(msg.timestamp_ms);

    let sender = if fix_encoding {
        fix_mojibake_encoding(&msg.sender_name)
    } else {
        msg.sender_name.clone()
    };

    Some(Message::with_metadata(
        sender, content, timestamp, None, // Instagram doesn't have message IDs in export
        None, // No reply references
        None, // No edit timestamps
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_encoding_ascii() {
        assert_eq!(fix_mojibake_encoding("Hello"), "Hello");
        assert_eq!(fix_mojibake_encoding("Test 123"), "Test 123");
    }

    #[test]
    fn test_parse_ms_timestamp() {
        let ts = parse_ms_timestamp(1705315800000);
        assert!(ts.is_some());
    }

    #[test]
    fn test_parse_instagram_message_basic() {
        let msg = InstagramRawMessage {
            sender_name: "user_one".to_string(),
            timestamp_ms: 1705315800000,
            content: Some("Hello!".to_string()),
            share: None,
            photos: None,
            videos: None,
            audio_files: None,
        };

        let result = parse_instagram_message(&msg, false);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert_eq!(parsed.sender, "user_one");
        assert_eq!(parsed.content, "Hello!");
    }

    #[test]
    fn test_parse_instagram_message_with_share() {
        let msg = InstagramRawMessage {
            sender_name: "user".to_string(),
            timestamp_ms: 1705315800000,
            content: None,
            share: Some(InstagramShare {
                share_text: Some("Check this out!".to_string()),
                link: Some("https://example.com".to_string()),
            }),
            photos: None,
            videos: None,
            audio_files: None,
        };

        let result = parse_instagram_message(&msg, false);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert_eq!(parsed.content, "Check this out!");
    }

    #[test]
    fn test_parse_instagram_message_empty() {
        let msg = InstagramRawMessage {
            sender_name: "user".to_string(),
            timestamp_ms: 1705315800000,
            content: None,
            share: None,
            photos: None,
            videos: None,
            audio_files: None,
        };

        assert!(parse_instagram_message(&msg, false).is_none());
    }
}
