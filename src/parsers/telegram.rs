use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;
use serde_json::Value;

use crate::core::InternalMessage;
use super::ChatParser;

/// Parser for Telegram JSON exports.
///
/// Telegram exports chats as JSON with the following structure:
/// ```json
/// {
///   "name": "Chat Name",
///   "messages": [
///     {
///       "type": "message",
///       "from": "Sender Name",
///       "text": "Hello" | ["Hello", {"type": "link", "text": "url"}]
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
    #[serde(rename = "type")]
    msg_type: String,
    from: Option<String>,
    text: Option<Value>,
}

impl ChatParser for TelegramParser {
    fn name(&self) -> &'static str {
        "Telegram"
    }

    fn parse(&self, file_path: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let export: TelegramExport = serde_json::from_reader(reader)?;

        let messages = export
            .messages
            .iter()
            .filter(|msg| msg.msg_type == "message")
            .filter_map(|msg| {
                let sender = msg.from.as_ref()?;
                let text_value = msg.text.as_ref()?;
                let content = extract_text(text_value);

                if content.trim().is_empty() {
                    return None;
                }

                Some(InternalMessage::new(sender, content))
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
                    .map(|s| s.to_string()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(""),
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
        assert_eq!(extract_text(&value), "Check this: https://example.com cool!");
    }

    #[test]
    fn test_extract_text_empty() {
        let value = json!(null);
        assert_eq!(extract_text(&value), "");
    }
}
