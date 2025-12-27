//! Telegram JSON export parser.

use std::fs;
use std::path::Path;

use crate::Message;
use crate::config::TelegramConfig;
use crate::error::ChatpackError;
use crate::parser::{Parser, Platform};
use crate::parsing::telegram::{TelegramExport, parse_telegram_message};

#[cfg(feature = "streaming")]
use crate::streaming::{StreamingConfig, StreamingParser, TelegramStreamingParser};

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
///
/// # Example
///
/// ```rust,no_run
/// use chatpack::parsers::TelegramParser;
/// use chatpack::parser::Parser;
///
/// let parser = TelegramParser::new();
/// let messages = parser.parse("telegram_export.json".as_ref())?;
/// # Ok::<(), chatpack::ChatpackError>(())
/// ```
pub struct TelegramParser {
    config: TelegramConfig,
}

impl TelegramParser {
    /// Creates a new parser with default configuration.
    pub fn new() -> Self {
        Self {
            config: TelegramConfig::default(),
        }
    }

    /// Creates a parser with custom configuration.
    pub fn with_config(config: TelegramConfig) -> Self {
        Self { config }
    }

    /// Creates a parser optimized for streaming large files.
    pub fn with_streaming() -> Self {
        Self {
            config: TelegramConfig::streaming(),
        }
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &TelegramConfig {
        &self.config
    }

    /// Parses content from a string (internal implementation).
    #[allow(clippy::unused_self)] // Keep &self for consistency with other parsers and future config use
    fn parse_content(&self, content: &str) -> Result<Vec<Message>, ChatpackError> {
        let export: TelegramExport = serde_json::from_str(content)?;

        // Use shared parsing logic
        let messages = export
            .messages
            .iter()
            .filter_map(parse_telegram_message)
            .collect();

        Ok(messages)
    }
}

impl Default for TelegramParser {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the new unified Parser trait
impl Parser for TelegramParser {
    fn name(&self) -> &'static str {
        "Telegram"
    }

    fn platform(&self) -> Platform {
        Platform::Telegram
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
                .with_max_message_size(self.config.max_message_size)
                .with_skip_invalid(self.config.skip_invalid);

            let streaming_parser = TelegramStreamingParser::with_config(streaming_config);
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
    use crate::parsing::telegram::extract_telegram_text;
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
    fn test_parser_name() {
        let parser = TelegramParser::new();
        assert_eq!(Parser::name(&parser), "Telegram");
    }
}
