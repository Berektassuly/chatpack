//! Telegram JSON export parser.

use std::error::Error;
use std::fs;

use chrono::DateTime;
use serde::Deserialize;
use serde_json::Value;

use super::ChatParser;
use crate::core::InternalMessage;

/// Parser for Telegram JSON exports.
///
/// Telegram exports chats as JSON with the following structure:
/// ```json
/// {
///   "name": "Chat Name",
///   "messages": [
///     {
///       "id": 12345,
///       "type": "message",
///       "date_unixtime": "1234567890",
///       "from": "Sender Name",
///       "text": "Hello" | ["Hello", {"type": "link", "text": "url"}],
///       "reply_to_message_id": 12344,
///       "edited_unixtime": "1234567899"
///     }
///   ]
/// }
/// ```
pub struct TelegramParser;

impl TelegramParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TelegramParser {
    fn default() -> Self {
        Self::new()
    }
}

// Internal structures for deserializing Telegram JSON

#[derive(Debug, Deserialize)]
struct TelegramExport {
    messages: Vec<TelegramMessage>,
}

#[derive(Debug, Deserialize)]
struct TelegramMessage {
    /// Message ID
    id: Option<u64>,
    /// Message type (we only care about "message")
    #[serde(rename = "type")]
    msg_type: String,
    /// Unix timestamp as string
    date_unixtime: Option<String>,
    /// Sender name
    from: Option<String>,
    /// Message text (can be string or array)
    text: Option<Value>,
    /// Reply reference
    reply_to_message_id: Option<u64>,
    /// Edit timestamp as string (if message was edited)
    edited_unixtime: Option<String>,
}

impl ChatParser for TelegramParser {
    fn name(&self) -> &'static str {
        "Telegram"
    }

    fn parse(&self, file_path: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>> {
        let content = fs::read_to_string(file_path)?;
        self.parse_str(&content)
    }

    fn parse_str(&self, content: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>> {
        let export: TelegramExport = serde_json::from_str(content)?;

        let messages = export
            .messages
            .iter()
            .filter(|msg| msg.msg_type == "message")
            .filter_map(|msg| {
                let sender = msg.from.as_ref()?;
                let text_value = msg.text.as_ref()?;
                let msg_content = extract_text(text_value);

                if msg_content.trim().is_empty() {
                    return None;
                }

                // Parse timestamp
                let timestamp = msg.date_unixtime.as_ref().and_then(|ts_str| {
                    ts_str
                        .parse::<i64>()
                        .ok()
                        .and_then(|ts| DateTime::from_timestamp(ts, 0))
                });

                // Parse edited timestamp
                let edited = msg.edited_unixtime.as_ref().and_then(|ts_str| {
                    ts_str
                        .parse::<i64>()
                        .ok()
                        .and_then(|ts| DateTime::from_timestamp(ts, 0))
                });

                Some(InternalMessage::with_metadata(
                    sender,
                    msg_content,
                    timestamp,
                    msg.id,
                    msg.reply_to_message_id,
                    edited,
                ))
            })
            .collect();

        Ok(messages)
    }
}

/// Extracts text content from Telegram's `text` field.
///
/// The field can be:
/// - A simple string: `"Hello"`
/// - An array with strings and objects: `["Text", {"type": "link", "text": "url"}]`
fn extract_text(text_value: &Value) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_text_string() {
        let value = json!("Hello world");
        assert_eq!(extract_text(&value), "Hello world");
    }

    #[test]
    fn test_extract_text_array_with_link() {
        let value = json!([
            "Check this: ",
            {"type": "link", "text": "https://example.com"},
            " cool!"
        ]);
        assert_eq!(
            extract_text(&value),
            "Check this: https://example.com cool!"
        );
    }

    #[test]
    fn test_extract_text_empty() {
        let value = json!(null);
        assert_eq!(extract_text(&value), "");
    }

    #[test]
    fn test_parser_name() {
        let parser = TelegramParser::new();
        assert_eq!(parser.name(), "Telegram");
    }
}
