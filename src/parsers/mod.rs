//! Chat export parsers for various platforms.
//!
//! This module provides parsers for chat exports from different messaging platforms.
//! Each parser implements both the new [`Parser`] trait and the legacy [`ChatParser`] trait.
//!
//! # Available Parsers
//!
//! - [`TelegramParser`] - Parses Telegram JSON exports (requires `telegram` feature)
//! - [`WhatsAppParser`] - Parses WhatsApp TXT exports (requires `whatsapp` feature)
//! - [`InstagramParser`] - Parses Instagram JSON exports (requires `instagram` feature)
//! - [`DiscordParser`] - Parses Discord JSON/TXT/CSV exports (requires `discord` feature)
//!
//! # New Unified API (Recommended)
//!
//! The new [`Parser`] trait provides a unified API with streaming support:
//!
//! ```rust,no_run
//! # #[cfg(feature = "telegram")]
//! # fn main() -> chatpack::Result<()> {
//! use chatpack::parser::{Parser, Platform, create_parser};
//!
//! let parser = create_parser(Platform::Telegram);
//! let messages = parser.parse("telegram_export.json".as_ref())?;
//!
//! // Or stream for large files
//! # #[cfg(feature = "streaming")]
//! let parser = chatpack::parser::create_streaming_parser(Platform::Telegram);
//! # #[cfg(feature = "streaming")]
//! for result in parser.stream("large_export.json".as_ref())? {
//!     // Process each message
//! }
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "telegram"))]
//! # fn main() {}
//! ```
//!
//! # Legacy API
//!
//! The [`ChatParser`] trait is still supported for backward compatibility:
//!
//! ```rust
//! # #[cfg(all(feature = "cli", feature = "telegram"))]
//! # fn main() {
//! use chatpack::cli::Source;
//! use chatpack::parsers::create_parser;
//!
//! let parser = create_parser(Source::Telegram);
//! // let messages = parser.parse("telegram_export.json")?;
//! # }
//! # #[cfg(not(all(feature = "cli", feature = "telegram")))]
//! # fn main() {}
//! ```

#[cfg(feature = "discord")]
mod discord;
#[cfg(feature = "instagram")]
mod instagram;
#[cfg(feature = "telegram")]
mod telegram;
#[cfg(feature = "whatsapp")]
mod whatsapp;

#[cfg(feature = "discord")]
pub use discord::DiscordParser;
#[cfg(feature = "instagram")]
pub use instagram::InstagramParser;
#[cfg(feature = "telegram")]
pub use telegram::TelegramParser;
#[cfg(feature = "whatsapp")]
pub use whatsapp::WhatsAppParser;

// Re-export the new unified Parser trait and Platform
pub use crate::parser::{Parser, Platform};

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
/// # #[cfg(all(feature = "cli", feature = "telegram"))]
/// # fn main() {
/// use chatpack::cli::Source;
/// use chatpack::parsers::create_parser;
///
/// let parser = create_parser(Source::Telegram);
/// assert_eq!(parser.name(), "Telegram");
/// # }
/// # #[cfg(not(all(feature = "cli", feature = "telegram")))]
/// # fn main() {}
/// ```
#[cfg(feature = "cli")]
#[allow(deprecated)]
pub fn create_parser(source: crate::cli::Source) -> Box<dyn ChatParser> {
    use crate::cli::Source;

    match source {
        #[cfg(feature = "telegram")]
        Source::Telegram => Box::new(TelegramParser::new()),
        #[cfg(feature = "whatsapp")]
        Source::WhatsApp => Box::new(WhatsAppParser::new()),
        #[cfg(feature = "instagram")]
        Source::Instagram => Box::new(InstagramParser::new()),
        #[cfg(feature = "discord")]
        Source::Discord => Box::new(DiscordParser::new()),
        // Fallback for when features are disabled
        #[allow(unreachable_patterns)]
        _ => panic!("Parser for {:?} is not enabled. Enable the corresponding feature.", source),
    }
}

#[cfg(test)]
#[allow(deprecated)] // Tests for deprecated ChatParser API (backward compatibility)
mod tests {
    use super::*;

    #[cfg(all(feature = "cli", feature = "telegram"))]
    #[test]
    fn test_create_parser_telegram() {
        use crate::cli::Source;
        let parser = create_parser(Source::Telegram);
        assert_eq!(parser.name(), "Telegram");
    }

    #[cfg(all(feature = "cli", feature = "whatsapp"))]
    #[test]
    fn test_create_parser_whatsapp() {
        use crate::cli::Source;
        let parser = create_parser(Source::WhatsApp);
        assert_eq!(parser.name(), "WhatsApp");
    }

    #[cfg(all(feature = "cli", feature = "instagram"))]
    #[test]
    fn test_create_parser_instagram() {
        use crate::cli::Source;
        let parser = create_parser(Source::Instagram);
        assert_eq!(parser.name(), "Instagram");
    }

    #[cfg(all(feature = "cli", feature = "discord"))]
    #[test]
    fn test_create_parser_discord() {
        use crate::cli::Source;
        let parser = create_parser(Source::Discord);
        assert_eq!(parser.name(), "Discord");
    }

    #[cfg(all(
        feature = "cli",
        feature = "telegram",
        feature = "whatsapp",
        feature = "instagram",
        feature = "discord"
    ))]
    #[test]
    fn test_all_parsers_implement_trait() {
        use crate::cli::Source;

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
