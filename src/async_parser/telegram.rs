//! Async Telegram parser.

use std::path::Path;

use async_trait::async_trait;

use crate::Message;
use crate::config::TelegramConfig;
use crate::error::ChatpackError;
use crate::parsing::telegram::{TelegramExport, parse_telegram_message};

use super::{AsyncParser, read_file_async};

/// Async parser for Telegram JSON exports.
///
/// # Example
///
/// ```rust,no_run
/// use chatpack::async_parser::{AsyncParser, AsyncTelegramParser};
///
/// # async fn example() -> Result<(), chatpack::ChatpackError> {
/// let parser = AsyncTelegramParser::new();
/// let messages = parser.parse("telegram_export.json").await?;
///
/// for msg in messages {
///     println!("{}: {}", msg.sender, msg.content);
/// }
/// # Ok(())
/// # }
/// ```
pub struct AsyncTelegramParser {
    #[allow(dead_code)]
    config: TelegramConfig,
}

impl AsyncTelegramParser {
    /// Creates a new async parser with default configuration.
    pub fn new() -> Self {
        Self {
            config: TelegramConfig::default(),
        }
    }

    /// Creates a parser with custom configuration.
    pub fn with_config(config: TelegramConfig) -> Self {
        Self { config }
    }
}

impl Default for AsyncTelegramParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AsyncParser for AsyncTelegramParser {
    fn name(&self) -> &'static str {
        "Telegram (Async)"
    }

    async fn parse(&self, path: impl AsRef<Path> + Send) -> Result<Vec<Message>, ChatpackError> {
        let content = read_file_async(path).await?;
        self.parse_str(&content)
    }

    fn parse_str(&self, content: &str) -> Result<Vec<Message>, ChatpackError> {
        let export: TelegramExport = serde_json::from_str(content)?;

        let messages = export
            .messages
            .iter()
            .filter_map(parse_telegram_message)
            .collect();

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_parser_name() {
        let parser = AsyncTelegramParser::new();
        assert_eq!(parser.name(), "Telegram (Async)");
    }

    #[test]
    fn test_parse_str() {
        let json = r#"{
            "messages": [
                {
                    "id": 1,
                    "type": "message",
                    "date_unixtime": "1705314600",
                    "from": "Alice",
                    "text": "Hello!"
                }
            ]
        }"#;

        let parser = AsyncTelegramParser::new();
        let messages = parser.parse_str(json).unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].sender, "Alice");
        assert_eq!(messages[0].content, "Hello!");
    }
}
