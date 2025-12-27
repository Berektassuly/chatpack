//! Streaming parsers for memory-efficient processing of large chat exports.
//!
//! This module provides streaming alternatives to the standard parsers,
//! designed for files >1GB where loading everything into memory is impractical.
//!
//! # Architecture
//!
//! The streaming API is built around two core traits:
//! - [`StreamingParser`] - produces an iterator of messages
//! - [`MessageIterator`] - the actual iterator implementation
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "telegram")]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use chatpack::streaming::{StreamingParser, TelegramStreamingParser};
//! use chatpack::Message;
//!
//! let parser = TelegramStreamingParser::new();
//!
//! // Process messages one at a time, never loading all into memory
//! for result in parser.stream("large_export.json").unwrap() {
//!     match result {
//!         Ok(message) => println!("{}: {}", message.sender, message.content),
//!         Err(e) => eprintln!("Skipped invalid message: {}", e),
//!     }
//! }
//!
//! // Or collect with error handling
//! let messages: Vec<Message> = parser
//!     .stream("large_export.json")
//!     .unwrap()
//!     .filter_map(Result::ok)
//!     .collect();
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "telegram"))]
//! # fn main() {}
//! ```
//!
//! # Memory Usage
//!
//! | Approach | 1GB File | 10GB File |
//! |----------|----------|-----------|
//! | Standard parser | ~3GB RAM | ~30GB RAM |
//! | Streaming parser | ~50MB RAM | ~50MB RAM |
//!
//! # Supported Formats
//!
//! - Telegram JSON (via [`TelegramStreamingParser`]) - requires `telegram` feature
//! - Discord JSONL/JSON (via [`DiscordStreamingParser`]) - requires `discord` feature
//! - Instagram JSON (via [`InstagramStreamingParser`]) - requires `instagram` feature
//! - WhatsApp TXT (via [`WhatsAppStreamingParser`]) - requires `whatsapp` feature

#[cfg(feature = "discord")]
mod discord;
mod error;
#[cfg(feature = "instagram")]
mod instagram;
#[cfg(feature = "telegram")]
mod telegram;
mod traits;
#[cfg(feature = "whatsapp")]
mod whatsapp;

#[cfg(feature = "discord")]
pub use discord::DiscordStreamingParser;
pub use error::{StreamingError, StreamingResult};
#[cfg(feature = "instagram")]
pub use instagram::InstagramStreamingParser;
#[cfg(feature = "telegram")]
pub use telegram::TelegramStreamingParser;
pub use traits::{MessageIterator, StreamingConfig, StreamingParser};
#[cfg(feature = "whatsapp")]
pub use whatsapp::WhatsAppStreamingParser;

use crate::parser::Platform;

/// Creates a streaming parser for the specified platform.
///
/// All platforms with the corresponding feature enabled support streaming parsing.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "telegram")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use chatpack::streaming::create_streaming_parser;
/// use chatpack::parser::Platform;
///
/// let parser = create_streaming_parser(Platform::Telegram);
/// for msg in parser.stream("large_file.json")? {
///     // Process each message
/// }
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "telegram"))]
/// # fn main() {}
/// ```
///
/// # Panics
///
/// Panics if the corresponding parser feature is not enabled.
pub fn create_streaming_parser(platform: Platform) -> Box<dyn StreamingParser> {
    match platform {
        #[cfg(feature = "telegram")]
        Platform::Telegram => Box::new(TelegramStreamingParser::new()),
        #[cfg(feature = "discord")]
        Platform::Discord => Box::new(DiscordStreamingParser::new()),
        #[cfg(feature = "instagram")]
        Platform::Instagram => Box::new(InstagramStreamingParser::new()),
        #[cfg(feature = "whatsapp")]
        Platform::WhatsApp => Box::new(WhatsAppStreamingParser::new()),
        // Fallback for when features are disabled
        #[allow(unreachable_patterns)]
        _ => panic!(
            "Streaming parser for {:?} is not enabled. Enable the corresponding feature.",
            platform
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "telegram")]
    #[test]
    fn test_create_streaming_parser_telegram() {
        let parser = create_streaming_parser(Platform::Telegram);
        assert_eq!(parser.name(), "Telegram (Streaming)");
    }

    #[cfg(feature = "discord")]
    #[test]
    fn test_create_streaming_parser_discord() {
        let parser = create_streaming_parser(Platform::Discord);
        assert_eq!(parser.name(), "Discord (Streaming)");
    }

    #[cfg(feature = "instagram")]
    #[test]
    fn test_create_streaming_parser_instagram() {
        let parser = create_streaming_parser(Platform::Instagram);
        assert_eq!(parser.name(), "Instagram (Streaming)");
    }

    #[cfg(feature = "whatsapp")]
    #[test]
    fn test_create_streaming_parser_whatsapp() {
        let parser = create_streaming_parser(Platform::WhatsApp);
        assert_eq!(parser.name(), "WhatsApp (Streaming)");
    }
}
