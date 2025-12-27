//! Shared Telegram parsing utilities.
//!
//! This module contains types and functions shared between the standard
//! and streaming Telegram parsers.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;

use crate::Message;

/// Raw Telegram message structure for deserialization.
///
/// Used by both standard and streaming parsers.
#[derive(Debug, Deserialize)]
pub struct TelegramRawMessage {
    /// Message ID
    pub id: Option<u64>,
    /// Message type (we only care about "message")
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Unix timestamp as string
    pub date_unixtime: Option<String>,
    /// Sender name
    pub from: Option<String>,
    /// Message text (can be string or array)
    pub text: Option<Value>,
    /// Reply reference
    pub reply_to_message_id: Option<u64>,
    /// Edit timestamp as string (if message was edited)
    pub edited_unixtime: Option<String>,
}

/// Telegram export wrapper.
#[derive(Debug, Deserialize)]
pub struct TelegramExport {
    pub messages: Vec<TelegramRawMessage>,
}

/// Extracts text content from Telegram's complex `text` field.
///
/// The `text` field in Telegram exports can be:
/// - A simple string: `"Hello"`
/// - An array with strings and objects: `["Text", {"type": "link", "text": "url"}]`
///
/// This function handles both cases and returns a single string.
///
/// # Example
///
/// ```ignore
/// use serde_json::json;
/// use chatpack::parsing::telegram::extract_telegram_text;
///
/// let simple = json!("Hello world");
/// assert_eq!(extract_telegram_text(&simple), "Hello world");
///
/// let complex = json!([
///     "Check this: ",
///     {"type": "link", "text": "https://example.com"}
/// ]);
/// assert_eq!(extract_telegram_text(&complex), "Check this: https://example.com");
/// ```
pub fn extract_telegram_text(text_value: &Value) -> String {
    match text_value {
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr
            .iter()
            .filter_map(|item| match item {
                Value::String(s) => Some(s.clone()),
                Value::Object(obj) => obj
                    .get("text")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string),
                _ => None,
            })
            .collect::<String>(),
        _ => String::new(),
    }
}

/// Parses a Unix timestamp string to DateTime.
///
/// Telegram stores timestamps as strings like "1234567890".
pub fn parse_unix_timestamp(ts_str: &str) -> Option<DateTime<Utc>> {
    ts_str
        .parse::<i64>()
        .ok()
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
}

/// Parses a raw Telegram message into a `Message`.
///
/// Returns `None` if:
/// - The message type is not "message"
/// - The sender is missing
/// - The content is empty
///
/// This is the core parsing logic shared between standard and streaming parsers.
pub fn parse_telegram_message(msg: &TelegramRawMessage) -> Option<Message> {
    // Skip non-message types
    if msg.msg_type != "message" {
        return None;
    }

    let sender = msg.from.as_ref()?;
    let text_value = msg.text.as_ref()?;
    let content = extract_telegram_text(text_value);

    if content.trim().is_empty() {
        return None;
    }

    let timestamp = msg
        .date_unixtime
        .as_ref()
        .and_then(|ts| parse_unix_timestamp(ts));
    let edited = msg
        .edited_unixtime
        .as_ref()
        .and_then(|ts| parse_unix_timestamp(ts));

    Some(Message::with_metadata(
        sender,
        content,
        timestamp,
        msg.id,
        msg.reply_to_message_id,
        edited,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_text_string() {
        let value = json!("Hello world");
        assert_eq!(extract_telegram_text(&value), "Hello world");
    }

    #[test]
    fn test_extract_text_array_with_link() {
        let value = json!([
            "Check this: ",
            {"type": "link", "text": "https://example.com"},
            " cool!"
        ]);
        assert_eq!(
            extract_telegram_text(&value),
            "Check this: https://example.com cool!"
        );
    }

    #[test]
    fn test_extract_text_empty() {
        let value = json!(null);
        assert_eq!(extract_telegram_text(&value), "");
    }

    #[test]
    fn test_parse_unix_timestamp() {
        let ts = parse_unix_timestamp("1705314600");
        assert!(ts.is_some());
        assert_eq!(ts.unwrap().timestamp(), 1705314600);
    }

    #[test]
    fn test_parse_unix_timestamp_invalid() {
        assert!(parse_unix_timestamp("not-a-number").is_none());
        assert!(parse_unix_timestamp("").is_none());
    }

    #[test]
    fn test_parse_telegram_message_basic() {
        let msg = TelegramRawMessage {
            id: Some(123),
            msg_type: "message".to_string(),
            date_unixtime: Some("1705314600".to_string()),
            from: Some("Alice".to_string()),
            text: Some(json!("Hello!")),
            reply_to_message_id: None,
            edited_unixtime: None,
        };

        let result = parse_telegram_message(&msg);
        assert!(result.is_some());

        let parsed = result.unwrap();
        assert_eq!(parsed.sender, "Alice");
        assert_eq!(parsed.content, "Hello!");
        assert!(parsed.timestamp.is_some());
    }

    #[test]
    fn test_parse_telegram_message_skip_service() {
        let msg = TelegramRawMessage {
            id: Some(123),
            msg_type: "service".to_string(),
            date_unixtime: Some("1705314600".to_string()),
            from: Some("Alice".to_string()),
            text: Some(json!("pinned a message")),
            reply_to_message_id: None,
            edited_unixtime: None,
        };

        assert!(parse_telegram_message(&msg).is_none());
    }

    #[test]
    fn test_parse_telegram_message_skip_empty() {
        let msg = TelegramRawMessage {
            id: Some(123),
            msg_type: "message".to_string(),
            date_unixtime: Some("1705314600".to_string()),
            from: Some("Alice".to_string()),
            text: Some(json!("   ")),
            reply_to_message_id: None,
            edited_unixtime: None,
        };

        assert!(parse_telegram_message(&msg).is_none());
    }
}
