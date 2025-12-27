//! Chat export parsers for various platforms.
//!
//! This module provides parsers for chat exports from different messaging platforms.
//! Each parser implements both the new [`Parser`] trait and the legacy [`ChatParser`] trait.
//!
//! # Available Parsers
//!
//! - [`TelegramParser`] - Parses Telegram JSON exports
//! - [`WhatsAppParser`] - Parses WhatsApp TXT exports
//! - [`InstagramParser`] - Parses Instagram JSON exports
//! - [`DiscordParser`] - Parses Discord JSON/TXT/CSV exports
//!
//! # New Unified API (Recommended)
//!
//! The new [`Parser`] trait provides a unified API with streaming support:
//!
//! ```rust,no_run
//! use chatpack::parser::{Parser, Platform, create_parser};
//!
//! let parser = create_parser(Platform::Telegram);
//! let messages = parser.parse("telegram_export.json".as_ref())?;
//!
//! // Or stream for large files
//! let parser = chatpack::parser::create_streaming_parser(Platform::Telegram);
//! for result in parser.stream("large_export.json".as_ref())? {
//!     // Process each message
//! }
//! # Ok::<(), chatpack::ChatpackError>(())
//! ```
//!
//! # Legacy API
//!
//! The [`ChatParser`] trait is still supported for backward compatibility:
//!
//! ```rust
//! use chatpack::cli::Source;
//! use chatpack::parsers::create_parser;
//!
//! let parser = create_parser(Source::Telegram);
//! // let messages = parser.parse("telegram_export.json")?;
//! ```

mod discord;
mod instagram;
mod telegram;
mod whatsapp;

pub use discord::DiscordParser;
pub use instagram::InstagramParser;
pub use telegram::TelegramParser;
pub use whatsapp::WhatsAppParser;

// Re-export the new unified Parser trait and Platform
pub use crate::parser::{Parser, Platform};

use crate::cli::Source;
use crate::error::ChatpackError;
use crate::Message;

/// Legacy trait for parsing chat exports from different platforms.
///
/// **Deprecated:** Use the new [`Parser`] trait instead, which provides
/// a unified API with streaming support.
///
/// Each platform-specific parser implements this trait for backward compatibility.
#[deprecated(since = "0.5.0", note = "Use the `Parser` trait from `chatpack::parser` instead")]
pub trait ChatParser: Send + Sync {
    /// Returns the name of the parser (e.g., "Telegram", "WhatsApp").
    fn name(&self) -> &'static str;

    /// Parses a chat export file and returns a vector of messages.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the export file
    ///
    /// # Errors
    ///
    /// Returns a [`ChatpackError`] if the file cannot be read or parsed.
    fn parse(&self, file_path: &str) -> Result<Vec<Message>, ChatpackError>;

    /// Parses chat content from a string.
    ///
    /// This is useful for WASM environments where file system access
    /// is not available.
    ///
    /// # Arguments
    ///
    /// * `content` - The raw content of the chat export
    ///
    /// # Errors
    ///
    /// Returns a [`ChatpackError`] if the content cannot be parsed.
    fn parse_str(&self, content: &str) -> Result<Vec<Message>, ChatpackError>;
}

/// Creates a parser for the specified source (legacy API).
///
/// **Note:** For new code, prefer using [`chatpack::parser::create_parser`] which
/// provides the unified [`Parser`] trait with streaming support.
///
/// # Example
///
/// ```rust
/// use chatpack::cli::Source;
/// use chatpack::parsers::create_parser;
///
/// let parser = create_parser(Source::Telegram);
/// assert_eq!(parser.name(), "Telegram");
/// ```
#[allow(deprecated)]
pub fn create_parser(source: Source) -> Box<dyn ChatParser> {
    match source {
        Source::Telegram => Box::new(TelegramParser::new()),
        Source::WhatsApp => Box::new(WhatsAppParser::new()),
        Source::Instagram => Box::new(InstagramParser::new()),
        Source::Discord => Box::new(DiscordParser::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_parser_telegram() {
        let parser = create_parser(Source::Telegram);
        assert_eq!(parser.name(), "Telegram");
    }

    #[test]
    fn test_create_parser_whatsapp() {
        let parser = create_parser(Source::WhatsApp);
        assert_eq!(parser.name(), "WhatsApp");
    }

    #[test]
    fn test_create_parser_instagram() {
        let parser = create_parser(Source::Instagram);
        assert_eq!(parser.name(), "Instagram");
    }

    #[test]
    fn test_create_parser_discord() {
        let parser = create_parser(Source::Discord);
        assert_eq!(parser.name(), "Discord");
    }

    #[test]
    fn test_all_parsers_implement_trait() {
        let sources = [
            Source::Telegram,
            Source::WhatsApp,
            Source::Instagram,
            Source::Discord,
        ];

        for source in sources {
            let parser = create_parser(source);
            let _ = parser.name();
        }
    }
}
