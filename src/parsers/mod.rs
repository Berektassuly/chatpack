//! Chat export parsers for different platforms.

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
/// a vector of `InternalMessage` structs.
pub trait ChatParser {
    /// Returns the name of the chat source (e.g., "Telegram", "WhatsApp")
    fn name(&self) -> &'static str;

    /// Parses a chat export file and returns a list of messages.
    ///
    /// # Arguments
    /// * `file_path` - Path to the export file
    ///
    /// # Returns
    /// * `Ok(Vec<InternalMessage>)` - Parsed messages
    /// * `Err` - If parsing fails
    fn parse(&self, file_path: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>>;
}

/// Factory function: creates the appropriate parser for the given source.
///
/// To add a new source:
/// 1. Create `newsource.rs` implementing `ChatParser`
/// 2. Add variant to `Source` enum in cli.rs
/// 3. Add match arm here
pub fn create_parser(source: Source) -> Box<dyn ChatParser> {
    match source {
        Source::Telegram => Box::new(TelegramParser::new()),
        Source::WhatsApp => Box::new(WhatsAppParser::new()),
        Source::Instagram => Box::new(InstagramParser::new()),
    }
}
