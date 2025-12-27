//! Instagram JSON export parser.
//!
//! Handles Meta's JSON exports with Mojibake encoding fix.
//!
//! Instagram exports messages as JSON (from "Download Your Data" feature).
//! The main quirk is that Meta exports UTF-8 text encoded as ISO-8859-1,
//! causing Cyrillic and other non-ASCII text to appear as garbage (Mojibake).

use std::fs;
use std::path::Path;

use crate::Message;
use crate::config::InstagramConfig;
use crate::error::ChatpackError;
use crate::parser::{Parser, Platform};
use crate::parsing::instagram::{InstagramExport, parse_instagram_message};

#[cfg(feature = "streaming")]
use crate::streaming::{InstagramStreamingParser, StreamingConfig, StreamingParser};

/// Parser for Instagram JSON exports.
///
/// # Example
///
/// ```rust,no_run
/// use chatpack::parsers::InstagramParser;
/// use chatpack::parser::Parser;
///
/// let parser = InstagramParser::new();
/// let messages = parser.parse("instagram_messages.json".as_ref())?;
/// # Ok::<(), chatpack::ChatpackError>(())
/// ```
pub struct InstagramParser {
    config: InstagramConfig,
}

impl InstagramParser {
    /// Creates a new parser with default configuration.
    pub fn new() -> Self {
        Self {
            config: InstagramConfig::default(),
        }
    }

    /// Creates a parser with custom configuration.
    pub fn with_config(config: InstagramConfig) -> Self {
        Self { config }
    }

    /// Creates a parser optimized for streaming large files.
    pub fn with_streaming() -> Self {
        Self {
            config: InstagramConfig::streaming(),
        }
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &InstagramConfig {
        &self.config
    }

    /// Parses content from a string (internal implementation).
    fn parse_content(&self, content: &str) -> Result<Vec<Message>, ChatpackError> {
        let export: InstagramExport = serde_json::from_str(content)?;

        let fix = self.config.fix_encoding;
        let mut messages: Vec<Message> = export
            .messages
            .iter()
            .filter_map(|msg| parse_instagram_message(msg, fix))
            .collect();

        // Instagram stores messages newest-first, reverse for chronological order
        messages.reverse();

        Ok(messages)
    }
}

impl Default for InstagramParser {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the new unified Parser trait
impl Parser for InstagramParser {
    fn name(&self) -> &'static str {
        "Instagram"
    }

    fn platform(&self) -> Platform {
        Platform::Instagram
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

            let streaming_parser = InstagramStreamingParser::with_config(streaming_config);
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
