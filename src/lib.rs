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
//! ## Quick Start (New Unified API)
//!
//! The new [`parser`] module provides a unified API with streaming support:
//!
//! ```rust,no_run
//! use chatpack::parser::{Parser, Platform, create_parser};
//! use chatpack::prelude::*;
//!
//! fn main() -> Result<()> {
//!     // Parse a Telegram export
//!     let parser = create_parser(Platform::Telegram);
//!     let messages = parser.parse("telegram_export.json".as_ref())?;
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
//! use chatpack::parser::{Parser, Platform, create_streaming_parser};
//!
//! let parser = create_streaming_parser(Platform::Telegram);
//!
//! // Process messages one at a time
//! for result in parser.stream("huge_export.json".as_ref())? {
//!     if let Ok(msg) = result {
//!         println!("{}: {}", msg.sender, msg.content);
//!     }
//! }
//! # Ok::<(), chatpack::ChatpackError>(())
//! ```
//!
//! ## Module Structure
//!
//! - [`parser`] — **New unified parser API** (recommended)
//!   - [`Parser`](parser::Parser) — Unified parser trait with streaming
//!   - [`Platform`](parser::Platform) — Supported platforms enum
//!   - [`create_parser`](parser::create_parser), [`create_streaming_parser`](parser::create_streaming_parser)
//! - [`config`] — Parser configuration types
//!   - [`TelegramConfig`](config::TelegramConfig), [`WhatsAppConfig`](config::WhatsAppConfig), etc.
//! - [`core`] — Core types and functionality
//!   - [`core::models`] — [`InternalMessage`] (deprecated, use [`Message`]), [`OutputConfig`]
//!   - [`core::filter`] — [`FilterConfig`], [`apply_filters`]
//!   - [`core::processor`] — [`merge_consecutive`], [`ProcessingStats`]
//!   - [`core::output`] — [`write_json`], [`write_jsonl`], [`write_csv`]
//! - [`parsers`] — Chat parsers (legacy + new API)
//!   - [`TelegramParser`], [`WhatsAppParser`], [`InstagramParser`], [`DiscordParser`]
//! - [`streaming`] — Streaming parsers for large files
//!   - [`TelegramStreamingParser`], [`DiscordStreamingParser`]
//! - [`cli`] — CLI types ([`Source`], [`OutputFormat`])
//! - [`error`] — Unified error types ([`ChatpackError`], [`Result`])
//! - [`prelude`] — Convenient re-exports

pub mod cli;
pub mod config;
pub mod core;
pub mod error;
pub mod message;
pub mod parser;
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
#[allow(deprecated)]
pub mod prelude {
    // Core message type
    pub use crate::Message;

    // Error types
    pub use crate::error::{ChatpackError, Result};

    // New unified parser API (recommended)
    pub use crate::parser::{Parser, Platform};

    // Platform configs
    pub use crate::config::{DiscordConfig, InstagramConfig, TelegramConfig, WhatsAppConfig};

    // Models (with backward-compatible alias)
    pub use crate::core::models::{InternalMessage, OutputConfig};

    // Filtering
    pub use crate::core::filter::{FilterConfig, apply_filters};

    // Processing
    pub use crate::core::processor::{ProcessingStats, merge_consecutive};

    // Output (file writers and string converters)
    pub use crate::core::output::{to_csv, to_json, to_jsonl, write_csv, write_json, write_jsonl};

    // Parsers (implement both Parser and legacy ChatParser traits)
    pub use crate::parsers::{DiscordParser, InstagramParser, TelegramParser, WhatsAppParser};

    // Legacy parser trait (deprecated)
    pub use crate::parsers::{ChatParser, create_parser};

    // CLI types
    pub use crate::cli::{OutputFormat, Source};
}
