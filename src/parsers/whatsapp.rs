//! `WhatsApp` TXT export parser.
//!
//! `WhatsApp` exports vary by locale. This parser auto-detects the format
//! by analyzing the first 20 lines of the file.
//!
//! Supported formats:
//! - US: `[1/15/24, 10:30:45 AM] Sender: Message`
//! - EU: `[15.01.24, 10:30:45] Sender: Message`
//! - EU2: `15/01/2024, 10:30 - Sender: Message`
//! - RU: `15.01.2024, 10:30 - Sender: Message`

use std::fs;
use std::path::Path;

use regex::Regex;

use crate::Message;
use crate::config::WhatsAppConfig;
use crate::error::ChatpackError;
use crate::parser::{Parser, Platform};
use crate::parsing::whatsapp::{
    detect_whatsapp_format, is_whatsapp_system_message, parse_whatsapp_timestamp,
};

#[cfg(feature = "streaming")]
use crate::streaming::{StreamingConfig, StreamingParser, WhatsAppStreamingParser};

/// Parser for WhatsApp TXT exports.
///
/// # Example
///
/// ```rust,no_run
/// use chatpack::parsers::WhatsAppParser;
/// use chatpack::parser::Parser;
///
/// let parser = WhatsAppParser::new();
/// let messages = parser.parse("whatsapp_chat.txt".as_ref())?;
/// # Ok::<(), chatpack::ChatpackError>(())
/// ```
pub struct WhatsAppParser {
    config: WhatsAppConfig,
}

impl WhatsAppParser {
    /// Creates a new parser with default configuration.
    pub fn new() -> Self {
        Self {
            config: WhatsAppConfig::default(),
        }
    }

    /// Creates a parser with custom configuration.
    pub fn with_config(config: WhatsAppConfig) -> Self {
        Self { config }
    }

    /// Creates a parser optimized for streaming large files.
    pub fn with_streaming() -> Self {
        Self {
            config: WhatsAppConfig::streaming(),
        }
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &WhatsAppConfig {
        &self.config
    }
}

impl Default for WhatsAppParser {
    fn default() -> Self {
        Self::new()
    }
}

impl WhatsAppParser {
    /// Parses content from a string (internal implementation).
    fn parse_content(&self, content: &str) -> Result<Vec<Message>, ChatpackError> {
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Ok(vec![]);
        }

        // Step 1: Auto-detect format from first 20 lines
        let sample_size = std::cmp::min(20, lines.len());
        let format = detect_whatsapp_format(&lines[..sample_size]).ok_or_else(|| {
            ChatpackError::invalid_format(
                "WhatsApp",
                "Could not detect WhatsApp export format. \
                 Make sure the file is a valid WhatsApp chat export.",
            )
        })?;

        // Step 2: Compile regex for detected format
        let regex = Regex::new(format.pattern())
            .map_err(|e| ChatpackError::invalid_format("WhatsApp", e.to_string()))?;

        // Step 3: Parse all lines
        let mut messages: Vec<Message> = Vec::new();

        for line in &lines {
            if line.trim().is_empty() {
                continue;
            }

            if let Some(caps) = regex.captures(line) {
                // New message starts
                let date_str = caps.get(1).map_or("", |m| m.as_str());
                let time_str = caps.get(2).map_or("", |m| m.as_str());
                let sender = caps.get(3).map_or("", |m| m.as_str().trim());
                let msg_content = caps.get(4).map_or("", |m| m.as_str());

                // Skip system messages (if configured)
                if self.config.skip_system_messages
                    && is_whatsapp_system_message(sender, msg_content)
                {
                    continue;
                }

                let timestamp = parse_whatsapp_timestamp(date_str, time_str, format);

                let msg = Message::with_metadata(
                    sender,
                    msg_content,
                    timestamp,
                    None, // WhatsApp doesn't have message IDs in export
                    None, // No reply references in text export
                    None, // No edit timestamps
                );

                messages.push(msg);
            } else {
                // Continuation of previous message (multiline)
                if let Some(last_msg) = messages.last_mut() {
                    last_msg.content.push('\n');
                    last_msg.content.push_str(line);
                }
                // If no previous message, skip orphan line
            }
        }

        Ok(messages)
    }
}

// Implement the new unified Parser trait
impl Parser for WhatsAppParser {
    fn name(&self) -> &'static str {
        "WhatsApp"
    }

    fn platform(&self) -> Platform {
        Platform::WhatsApp
    }

    fn parse(&self, path: &Path) -> Result<Vec<Message>, ChatpackError> {
        let content = fs::read_to_string(path)?;
        self.parse_content(&content)
    }

    fn parse_str(&self, content: &str) -> Result<Vec<Message>, ChatpackError> {
        self.parse_content(content)
    }

    #[cfg(feature = "streaming")]
    fn stream(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = Result<Message, ChatpackError>> + Send>, ChatpackError>
    {
        if self.config.streaming {
            // Use native streaming parser
            let streaming_config = StreamingConfig::new()
                .with_buffer_size(self.config.buffer_size)
                .with_skip_invalid(self.config.skip_invalid);

            let streaming_parser = WhatsAppStreamingParser::with_config(streaming_config);
            let iterator =
                StreamingParser::stream(&streaming_parser, path.to_str().unwrap_or_default())?;

            Ok(Box::new(
                iterator.map(|result| result.map_err(ChatpackError::from)),
            ))
        } else {
            // Fallback: load everything into memory
            let messages = Parser::parse(self, path)?;
            Ok(Box::new(messages.into_iter().map(Ok)))
        }
    }

    #[cfg(feature = "streaming")]
    fn supports_streaming(&self) -> bool {
        self.config.streaming
    }

    #[cfg(feature = "streaming")]
    fn recommended_buffer_size(&self) -> usize {
        self.config.buffer_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::whatsapp::DateFormat;

    #[test]
    fn test_parser_name() {
        let parser = WhatsAppParser::new();
        assert_eq!(Parser::name(&parser), "WhatsApp");
    }

    #[test]
    fn test_detect_format_us() {
        let lines = vec![
            "[1/15/24, 10:30:45 AM] Alice: Hello",
            "[1/15/24, 10:31:00 AM] Bob: Hi there",
        ];
        assert_eq!(detect_whatsapp_format(&lines), Some(DateFormat::US));
    }

    #[test]
    fn test_detect_format_eu_dot_bracketed() {
        let lines = vec![
            "[15.01.24, 10:30:45] Alice: Hello",
            "[15.01.24, 10:31:00] Bob: Hi there",
        ];
        assert_eq!(
            detect_whatsapp_format(&lines),
            Some(DateFormat::EuDotBracketed)
        );
    }

    #[test]
    fn test_detect_format_eu_dot_no_bracket() {
        let lines = vec![
            "26.10.2025, 20:40 - Alice: Hello",
            "26.10.2025, 20:41 - Bob: Hi there",
        ];
        assert_eq!(
            detect_whatsapp_format(&lines),
            Some(DateFormat::EuDotNoBracket)
        );
    }

    #[test]
    fn test_detect_format_eu_slash() {
        let lines = vec![
            "15/01/2024, 10:30 - Alice: Hello",
            "15/01/2024, 10:31 - Bob: Hi there",
        ];
        assert_eq!(detect_whatsapp_format(&lines), Some(DateFormat::EuSlash));
    }

    #[test]
    fn test_is_system_message_english() {
        assert!(is_whatsapp_system_message(
            "Alice",
            "Messages and calls are end-to-end encrypted"
        ));
        assert!(is_whatsapp_system_message(
            "Bob",
            "added Charlie to the group"
        ));
        assert!(is_whatsapp_system_message("Alice", "left"));
        assert!(!is_whatsapp_system_message("Alice", "Hello everyone!"));
        assert!(!is_whatsapp_system_message("Bob", "<Media omitted>"));
    }

    #[test]
    fn test_is_system_message_russian() {
        assert!(is_whatsapp_system_message(
            "Система",
            "Сообщения и звонки защищены сквозным шифрованием"
        ));
        assert!(is_whatsapp_system_message("Bob", "Подробнее"));
        assert!(!is_whatsapp_system_message("Муха", "Добрый вечер"));
        assert!(!is_whatsapp_system_message("Bob", "<Без медиафайлов>"));
    }

    #[test]
    fn test_parse_timestamp_us() {
        let ts = parse_whatsapp_timestamp("1/15/24", "10:30:45 AM", DateFormat::US);
        assert!(ts.is_some());
    }

    #[test]
    fn test_parse_timestamp_eu_dot() {
        let ts = parse_whatsapp_timestamp("15.01.24", "10:30:45", DateFormat::EuDotBracketed);
        assert!(ts.is_some());

        let ts2 = parse_whatsapp_timestamp("26.10.2025", "20:40", DateFormat::EuDotNoBracket);
        assert!(ts2.is_some());
    }

    #[test]
    fn test_media_not_filtered() {
        // <Media omitted> should NOT be treated as system message
        assert!(!is_whatsapp_system_message("Alice", "<Media omitted>"));
        assert!(!is_whatsapp_system_message("Bob", "image omitted"));
        assert!(!is_whatsapp_system_message("Муха", "<Без медиафайлов>"));
    }

    #[test]
    fn test_empty_sender_is_system() {
        assert!(is_whatsapp_system_message("", "Some message"));
        assert!(is_whatsapp_system_message("   ", "Some message"));
    }
}
