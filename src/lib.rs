//! # Chatpack
//!
//! A Rust library for parsing and converting chat exports from popular messaging
//! platforms into LLM-friendly formats.
//!
//! ## Overview
//!
//! Chatpack provides a unified API for working with chat exports from:
//! - **Telegram** — JSON exports from Telegram Desktop
//! - **WhatsApp** — Text exports (both iOS and Android formats)
//! - **Instagram** — JSON exports from Instagram data download
//! - **Discord** — JSON/TXT/CSV exports from DiscordChatExporter
//!
//! The library handles the complexity of different export formats and provides
//! tools for filtering, merging, and outputting messages in formats optimized
//! for use with Large Language Models.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use chatpack::prelude::*;
//!
//! fn main() -> Result<()> {
//!     // Parse a Telegram export
//!     let parser = create_parser(Source::Telegram);
//!     let messages = parser.parse("telegram_export.json")?;
//!
//!     // Merge consecutive messages from the same sender
//!     let merged = merge_consecutive(messages);
//!
//!     // Write to JSON
//!     write_json(&merged, "output.json", &OutputConfig::new())?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Streaming for Large Files
//!
//! For files larger than 1GB, use the streaming API to avoid memory issues:
//!
//! ```rust,no_run
//! use chatpack::streaming::{StreamingParser, TelegramStreamingParser};
//!
//! let parser = TelegramStreamingParser::new();
//!
//! // Process messages one at a time
//! for result in parser.stream("huge_export.json")? {
//!     if let Ok(msg) = result {
//!         println!("{}: {}", msg.sender, msg.content);
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Module Structure
//!
//! - [`core`] — Core types and functionality
//!   - [`core::models`] — [`InternalMessage`] (deprecated, use [`Message`]), [`OutputConfig`]
//!   - [`core::filter`] — [`FilterConfig`], [`apply_filters`]
//!   - [`core::processor`] — [`merge_consecutive`], [`ProcessingStats`]
//!   - [`core::output`] — [`write_json`], [`write_jsonl`], [`write_csv`]
//! - [`parsers`] — Chat parsers
//!   - [`TelegramParser`], [`WhatsAppParser`], [`InstagramParser`], [`DiscordParser`]
//!   - [`create_parser`]
//! - [`streaming`] — Streaming parsers for large files
//!   - [`TelegramStreamingParser`], [`DiscordStreamingParser`]
//! - [`cli`] — CLI types ([`Source`], [`OutputFormat`])
//! - [`error`] — Unified error types ([`ChatpackError`], [`Result`])
//! - [`prelude`] — Convenient re-exports

pub mod cli;
pub mod core;
pub mod error;
pub mod message;
pub mod parsers;
pub mod streaming;

// Re-export the main types at the crate root for convenience
pub use error::{ChatpackError, Result};
pub use message::Message;

/// Convenient re-exports for common usage.
///
/// Import everything you need with a single line:
///
/// ```rust
/// use chatpack::prelude::*;
/// ```
pub mod prelude {
    // Core message type
    pub use crate::Message;

    // Error types
    pub use crate::error::{ChatpackError, Result};

    // Models (with backward-compatible alias)
    pub use crate::core::models::{InternalMessage, OutputConfig};

    // Filtering
    pub use crate::core::filter::{FilterConfig, apply_filters};

    // Processing
    pub use crate::core::processor::{ProcessingStats, merge_consecutive};

    // Output (file writers and string converters)
    pub use crate::core::output::{to_csv, to_json, to_jsonl, write_csv, write_json, write_jsonl};

    // Parsers
    pub use crate::parsers::{
        ChatParser, DiscordParser, InstagramParser, TelegramParser, WhatsAppParser, create_parser,
    };

    // CLI types
    pub use crate::cli::{OutputFormat, Source};
}
