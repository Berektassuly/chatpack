//! Chat export parsers for various platforms.
//!
//! This module provides parsers for chat exports from different messaging platforms.
//! Each parser implements the [`ChatParser`] trait.
//!
//! # Available Parsers
//!
//! - [`TelegramParser`] - Parses Telegram JSON exports
//! - [`WhatsAppParser`] - Parses WhatsApp TXT exports
//! - [`InstagramParser`] - Parses Instagram JSON exports
//! - [`DiscordParser`] - Parses Discord JSON/TXT/CSV exports
//!
//! # Example
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

use crate::cli::Source;
use crate::core::InternalMessage;
use std::error::Error;

/// Trait for parsing chat exports from different platforms.
///
/// Each platform-specific parser must implement this trait.
pub trait ChatParser: Send + Sync {
    /// Returns the name of the parser (e.g., "Telegram", "WhatsApp").
    fn name(&self) -> &'static str;

    /// Parses a chat export file and returns a vector of internal messages.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the export file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    fn parse(&self, file_path: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>>;
}

/// Creates a parser for the specified source.
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
            // Just verify we can call the trait method
            let _ = parser.name();
        }
    }
}
