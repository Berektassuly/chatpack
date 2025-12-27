//! Memory-efficient streaming parsers for large chat exports.
//!
//! This module provides streaming alternatives to standard parsers, designed
//! for files that are too large to fit in memory (>500MB).
//!
//! # When to Use Streaming
//!
//! | File Size | Recommendation |
//! |-----------|----------------|
//! | < 100MB | Standard parser (faster) |
//! | 100-500MB | Either works |
//! | > 500MB | Streaming parser (required) |
//!
//! # Memory Comparison
//!
//! | Approach | 1GB File | 10GB File |
//! |----------|----------|-----------|
//! | Standard parser | ~3GB RAM | ~30GB RAM |
//! | Streaming parser | ~50MB RAM | ~50MB RAM |
//!
//! # Core Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`StreamingParser`] | Trait for parsers that produce message iterators |
//! | [`MessageIterator`] | Iterator over messages with progress tracking |
//! | [`StreamingConfig`] | Configuration for buffer sizes and behavior |
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! # #[cfg(feature = "telegram")]
//! # fn main() -> chatpack::Result<()> {
//! use chatpack::streaming::{StreamingParser, TelegramStreamingParser};
//!
//! let parser = TelegramStreamingParser::new();
//!
//! for result in parser.stream("large_export.json")? {
//!     let msg = result?;
//!     println!("{}: {}", msg.sender, msg.content);
//! }
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "telegram"))]
//! # fn main() {}
//! ```
//!
//! ## With Progress Tracking
//!
//! ```no_run
//! # #[cfg(feature = "telegram")]
//! # fn main() -> chatpack::Result<()> {
//! use chatpack::streaming::{StreamingParser, TelegramStreamingParser};
//!
//! let parser = TelegramStreamingParser::new();
//! let mut iter = parser.stream("large_export.json")?;
//! let mut count = 0;
//!
//! while let Some(result) = iter.next() {
//!     let _msg = result?;
//!     count += 1;
//!
//!     if count % 100_000 == 0 {
//!         if let Some(progress) = iter.progress() {
//!             println!("{:.1}% complete", progress);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "telegram"))]
//! # fn main() {}
//! ```
//!
//! # Available Parsers
//!
//! | Parser | Feature | Format |
//! |--------|---------|--------|
//! | [`TelegramStreamingParser`] | `telegram` | JSON |
//! | [`WhatsAppStreamingParser`] | `whatsapp` | TXT |
//! | [`InstagramStreamingParser`] | `instagram` | JSON |
//! | [`DiscordStreamingParser`] | `discord` | JSON/JSONL/CSV |

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
