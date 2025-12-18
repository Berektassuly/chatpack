mod telegram;
mod whatsapp;
mod instagram;

pub use telegram::TelegramParser;
pub use whatsapp::WhatsAppParser;
pub use instagram::InstagramParser;

use std::error::Error;

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

/// Supported chat sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatSource {
    Telegram,
    WhatsApp,
    Instagram,
}

impl ChatSource {
    /// Parse from command-line argument string
    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg.to_lowercase().as_str() {
            "telegram" | "tg" => Some(Self::Telegram),
            "whatsapp" | "wa" => Some(Self::WhatsApp),
            "instagram" | "ig" => Some(Self::Instagram),
            _ => None,
        }
    }

    /// List all available sources for help message
    pub fn available() -> &'static [&'static str] {
        &["telegram (tg)", "whatsapp (wa)", "instagram (ig)"]
    }
}

/// Factory function: creates the appropriate parser for the given source.
///
/// To add a new source:
/// 1. Create `newsource.rs` implementing `ChatParser`
/// 2. Add variant to `ChatSource` enum
/// 3. Add match arm here
pub fn create_parser(source: ChatSource) -> Box<dyn ChatParser> {
    match source {
        ChatSource::Telegram => Box::new(TelegramParser::new()),
        ChatSource::WhatsApp => Box::new(WhatsAppParser::new()),
        ChatSource::Instagram => Box::new(InstagramParser::new()),
    }
}
