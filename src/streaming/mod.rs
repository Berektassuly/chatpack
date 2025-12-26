//! Streaming parsers for memory-efficient processing of large chat exports.
//!
//! This module provides streaming alternatives to the standard parsers,
//! designed for files >1GB where loading everything into memory is impractical.
//!
//! # Architecture
//!
//! The streaming API is built around two core traits:
//! - [`StreamingParser`] — produces an iterator of messages
//! - [`MessageIterator`] — the actual iterator implementation
//!
//! # Example
//!
//! ```rust,no_run
//! use chatpack::streaming::{StreamingParser, TelegramStreamingParser};
//! use chatpack::core::InternalMessage;
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
//! let messages: Vec<InternalMessage> = parser
//!     .stream("large_export.json")
//!     .unwrap()
//!     .filter_map(Result::ok)
//!     .collect();
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
//! - Telegram JSON (via [`TelegramStreamingParser`])
//! - Discord JSONL (via [`DiscordStreamingParser`])
//!
//! WhatsApp and Instagram use line-based/small JSON formats that don't
//! typically require streaming.

mod discord;
mod error;
mod telegram;
mod traits;

pub use discord::DiscordStreamingParser;
pub use error::{StreamingError, StreamingResult};
pub use telegram::TelegramStreamingParser;
pub use traits::{MessageIterator, StreamingConfig, StreamingParser};

use crate::cli::Source;

/// Creates a streaming parser for the specified source.
///
/// Returns `None` for sources that don't support streaming
/// (WhatsApp and Instagram typically don't need it).
///
/// # Example
///
/// ```rust,no_run
/// use chatpack::streaming::create_streaming_parser;
/// use chatpack::cli::Source;
///
/// if let Some(parser) = create_streaming_parser(Source::Telegram) {
///     for msg in parser.stream("large_file.json").unwrap() {
///         // Process each message
///     }
/// }
/// ```
pub fn create_streaming_parser(source: Source) -> Option<Box<dyn StreamingParser>> {
    match source {
        Source::Telegram => Some(Box::new(TelegramStreamingParser::new())),
        Source::Discord => Some(Box::new(DiscordStreamingParser::new())),
        // WhatsApp and Instagram don't typically need streaming
        Source::WhatsApp | Source::Instagram => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_streaming_parser_telegram() {
        let parser = create_streaming_parser(Source::Telegram);
        assert!(parser.is_some());
        assert_eq!(parser.unwrap().name(), "Telegram (Streaming)");
    }

    #[test]
    fn test_create_streaming_parser_discord() {
        let parser = create_streaming_parser(Source::Discord);
        assert!(parser.is_some());
    }

    #[test]
    fn test_create_streaming_parser_unsupported() {
        assert!(create_streaming_parser(Source::WhatsApp).is_none());
        assert!(create_streaming_parser(Source::Instagram).is_none());
    }
}
