//! Chat export parsers for different platforms.
//!
//! This module provides parsers for:
//! - [`TelegramParser`] - Telegram JSON exports
//! - [`WhatsAppParser`] - `WhatsApp` TXT exports (auto-detects locale)
//! - [`InstagramParser`] - Instagram JSON exports (with Mojibake fix)
//!
//! # Using the Factory Function
//!
//! The easiest way to get a parser is via [`create_parser`]:
//!
//! ```rust,no_run
//! use chatpack::parsers::create_parser;
//! use chatpack::cli::Source;
//!
//! let parser = create_parser(Source::Telegram);
//! let messages = parser.parse("chat.json").unwrap();
//! ```
//!
//! # Using Parsers Directly
//!
//! For more control, instantiate parsers directly:
//!
//! ```rust,no_run
//! use chatpack::parsers::{TelegramParser, ChatParser};
//!
//! let parser = TelegramParser::new();
//! let messages = parser.parse("result.json").unwrap();
//! ```
//!
//! # Implementing a Custom Parser
//!
//! To add support for a new chat source, implement [`ChatParser`]:
//!
//! ```rust
//! use chatpack::parsers::ChatParser;
//! use chatpack::core::InternalMessage;
//! use std::error::Error;
//!
//! struct MyCustomParser;
//!
//! impl ChatParser for MyCustomParser {
//!     fn name(&self) -> &'static str {
//!         "MyChat"
//!     }
//!
//!     fn parse(&self, file_path: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>> {
//!         // Parse your format here
//!         Ok(vec![])
//!     }
//! }
//! ```

mod instagram;
mod telegram;
mod whatsapp;

pub use instagram::InstagramParser;
pub use telegram::TelegramParser;
pub use whatsapp::WhatsAppParser;

use std::error::Error;

use crate::cli::Source;
use crate::core::InternalMessage;

/// Trait for parsing chat exports from different messengers.
///
/// Implement this trait to add support for a new chat source.
/// The parser should convert the source-specific format into
/// a vector of [`InternalMessage`] structs.
///
/// # Example Implementation
///
/// ```rust
/// use chatpack::parsers::ChatParser;
/// use chatpack::core::InternalMessage;
/// use std::error::Error;
///
/// struct DiscordParser;
///
/// impl ChatParser for DiscordParser {
///     fn name(&self) -> &'static str {
///         "Discord"
///     }
///
///     fn parse(&self, file_path: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>> {
///         let content = std::fs::read_to_string(file_path)?;
///         // Parse Discord-specific format...
///         Ok(vec![])
///     }
/// }
/// ```
pub trait ChatParser: Send + Sync {
    /// Returns the human-readable name of the chat source.
    ///
    /// Used for logging and error messages.
    fn name(&self) -> &'static str;

    /// Parses a chat export file and returns a list of messages.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the export file
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<InternalMessage>)` - Parsed messages in chronological order
    /// * `Err` - If parsing fails (file not found, invalid format, etc.)
    ///
    /// # Notes
    ///
    /// - Messages should be returned in chronological order (oldest first)
    /// - Empty or whitespace-only messages may be filtered out
    /// - Platform-specific metadata should be preserved where available
    fn parse(&self, file_path: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>>;
}

/// Creates the appropriate parser for the given source.
///
/// This is the recommended way to get a parser when the source
/// is determined at runtime (e.g., from CLI arguments).
///
/// # Example
///
/// ```rust
/// use chatpack::parsers::create_parser;
/// use chatpack::cli::Source;
///
/// let parser = create_parser(Source::Telegram);
/// println!("Using {} parser", parser.name());
/// ```
///
/// # Adding New Sources
///
/// To add a new source:
/// 1. Create `newsource.rs` implementing [`ChatParser`]
/// 2. Add variant to [`Source`] enum in `cli.rs`
/// 3. Add match arm here
pub fn create_parser(source: Source) -> Box<dyn ChatParser> {
    match source {
        Source::Telegram => Box::new(TelegramParser::new()),
        Source::WhatsApp => Box::new(WhatsAppParser::new()),
        Source::Instagram => Box::new(InstagramParser::new()),
    }
}

/// Convenience function to parse a file with auto-detected source.
///
/// Attempts to detect the source from file extension and content.
/// Falls back to Telegram JSON if detection fails.
///
/// # Example
///
/// ```rust,no_run
/// use chatpack::parsers::parse_auto;
///
/// // Automatically detects format
/// let messages = parse_auto("chat.json").unwrap();
/// ```
pub fn parse_auto(file_path: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>> {
    let source = detect_source(file_path)?;
    let parser = create_parser(source);
    parser.parse(file_path)
}

/// Detects the chat source from file extension and content.
fn detect_source(file_path: &str) -> Result<Source, Box<dyn Error>> {
    let path = std::path::Path::new(file_path);

    match path.extension().and_then(|e| e.to_str()) {
        Some("txt") => Ok(Source::WhatsApp),
        Some("json") => {
            // Try to detect from content
            let content = std::fs::read_to_string(file_path)?;
            if content.contains("\"messages\"") && content.contains("\"type\"") {
                // Could be Telegram or Instagram
                if content.contains("\"sender_name\"") {
                    Ok(Source::Instagram)
                } else {
                    Ok(Source::Telegram)
                }
            } else {
                Ok(Source::Telegram) // Default
            }
        }
        _ => Ok(Source::Telegram), // Default
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
    fn test_parser_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TelegramParser>();
        assert_send_sync::<WhatsAppParser>();
        assert_send_sync::<InstagramParser>();
    }
}
