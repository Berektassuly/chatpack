use std::error::Error;

use crate::core::InternalMessage;
use super::ChatParser;

/// Parser for Instagram JSON exports.
///
/// Instagram exports messages as JSON (from "Download Your Data" feature).
/// The structure typically looks like:
/// ```json
/// {
///   "participants": [{"name": "User1"}, {"name": "User2"}],
///   "messages": [
///     {
///       "sender_name": "User1",
///       "timestamp_ms": 1234567890000,
///       "content": "Hello!",
///       "type": "Generic"
///     }
///   ]
/// }
/// ```
///
/// # TODO
/// - Handle different message types (Generic, Share, etc.)
/// - Handle reactions
/// - Handle media (photos, videos, audio)
/// - Decode UTF-8 encoding issues (Instagram exports with escaped unicode)
pub struct InstagramParser;

impl InstagramParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InstagramParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatParser for InstagramParser {
    fn name(&self) -> &'static str {
        "Instagram"
    }

    fn parse(&self, file_path: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>> {
        // TODO: Implement Instagram parsing
        //
        // Steps:
        // 1. Read and parse JSON file
        // 2. Extract messages array
        // 3. Filter by type == "Generic" (skip shares, reactions, etc.)
        // 4. Fix UTF-8 encoding (Instagram escapes unicode weirdly)
        // 5. Convert to InternalMessage
        //
        // Note: Messages are typically in reverse chronological order,
        // so you may need to reverse the array.
        //
        // Serde structs suggestion:
        // #[derive(Deserialize)]
        // struct InstagramExport {
        //     messages: Vec<InstagramMessage>,
        // }
        //
        // #[derive(Deserialize)]
        // struct InstagramMessage {
        //     sender_name: String,
        //     content: Option<String>,
        //     #[serde(rename = "type")]
        //     msg_type: Option<String>,
        // }

        Err(format!(
            "Instagram parser not yet implemented. File: {}",
            file_path
        ).into())
    }
}

/// Fixes Instagram's broken UTF-8 encoding.
/// Instagram exports text as latin1-decoded UTF-8, causing mojibake.
///
/// # Example
/// Input:  "Привет" exported as "ÐŸÑ€Ð¸Ð²ÐµÑ‚"
/// Output: "Привет"
#[allow(dead_code)]
fn fix_instagram_encoding(text: &str) -> String {
    // Instagram encodes UTF-8 bytes as latin1 characters
    // To fix: interpret each char as a byte and decode as UTF-8
    let bytes: Vec<u8> = text.chars().map(|c| c as u8).collect();
    String::from_utf8(bytes).unwrap_or_else(|_| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_name() {
        let parser = InstagramParser::new();
        assert_eq!(parser.name(), "Instagram");
    }

    #[test]
    fn test_fix_encoding() {
        // This test will work when you have actual mojibake examples
        let fixed = fix_instagram_encoding("Hello");
        assert_eq!(fixed, "Hello");
    }
}
