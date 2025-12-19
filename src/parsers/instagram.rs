//! Instagram JSON export parser.
//!
//! Handles Meta's JSON exports with Mojibake encoding fix.
//!
//! Instagram exports messages as JSON (from "Download Your Data" feature).
//! The main quirk is that Meta exports UTF-8 text encoded as ISO-8859-1,
//! causing Cyrillic and other non-ASCII text to appear as garbage (Mojibake).

use std::error::Error;
use std::fs;

use chrono::{TimeZone, Utc};
use serde::Deserialize;

use super::ChatParser;
use crate::core::InternalMessage;

#[derive(Debug, Deserialize)]
struct InstagramExport {
    messages: Vec<InstagramMessage>,
}

#[derive(Debug, Deserialize)]
struct InstagramMessage {
    sender_name: String,
    timestamp_ms: i64,
    content: Option<String>,
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
fn fix_encoding(s: &str) -> String {
    let bytes: Vec<u8> = s.chars().map(|c| c as u8).collect();
    String::from_utf8(bytes).unwrap_or_else(|_| s.to_string())
}

/// Parser for Instagram JSON exports.
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
        let content = fs::read_to_string(file_path)?;

        let export: InstagramExport = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse Instagram JSON: {}", e))?;

        let mut messages: Vec<InternalMessage> = export
            .messages
            .into_iter()
            .filter_map(|msg| {
                // Skip messages without content (shares, reactions without text, etc.)
                let content = msg.content?;
                if content.is_empty() {
                    return None;
                }

                let timestamp = Utc.timestamp_millis_opt(msg.timestamp_ms).single()?;

                Some(InternalMessage {
                    id: None,
                    timestamp: Some(timestamp),
                    sender: fix_encoding(&msg.sender_name),
                    reply_to: None,
                    edited: None,
                    content: fix_encoding(&content),
                })
            })
            .collect();

        // Instagram stores messages newest-first, reverse for chronological order
        messages.reverse();

        Ok(messages)
    }
}
