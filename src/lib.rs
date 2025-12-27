//! # Chatpack
//!
//! A Rust library for parsing and converting chat exports from popular messaging
//! platforms into LLM-friendly formats.
//!
//! ## Overview
//!
//! Chatpack provides a unified API for working with chat exports from:
//! - **Telegram** - JSON exports from Telegram Desktop
//! - **WhatsApp** - Text exports (both iOS and Android formats)
//! - **Instagram** - JSON exports from Instagram data download
//! - **Discord** - JSON/TXT/CSV exports from DiscordChatExporter
//!
//! The library handles the complexity of different export formats and provides
//! tools for filtering, merging, and outputting messages in formats optimized
//! for use with Large Language Models.
//!
//! ## Feature Flags
//!
//! Chatpack uses feature flags to minimize dependencies:
//!
//! | Feature | Description | Dependencies |
//! |---------|-------------|--------------|
//! | `telegram` | Telegram parser | `serde_json` |
//! | `whatsapp` | WhatsApp parser | `regex` |
//! | `instagram` | Instagram parser | `serde_json` |
//! | `discord` | Discord parser | `serde_json`, `regex`, `csv` |
//! | `csv-output` | CSV output writer | `csv` |
//! | `json-output` | JSON/JSONL output writers | `serde_json` |
//! | `streaming` | Streaming parsers for large files | (none) |
//! | `full` | Everything (default) | all above |
//!
//! ## Quick Start
//!
//! The [`parser`] module provides a unified API with streaming support:
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "telegram", feature = "json-output"))]
//! # fn main() -> chatpack::Result<()> {
//! use chatpack::parser::{Parser, Platform, create_parser};
//! use chatpack::prelude::*;
//!
//! // Parse a Telegram export
//! let parser = create_parser(Platform::Telegram);
//! let messages = parser.parse("telegram_export.json".as_ref())?;
//!
//! // Merge consecutive messages from the same sender
//! let merged = merge_consecutive(messages);
//!
//! // Write to JSON
//! write_json(&merged, "output.json", &OutputConfig::new())?;
//!
//! # Ok(())
//! # }
//! # #[cfg(not(all(feature = "telegram", feature = "json-output")))]
//! # fn main() {}
//! ```
//!
//! ## Streaming for Large Files
//!
//! For files larger than 1GB, use the streaming API to avoid memory issues:
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "telegram", feature = "streaming"))]
//! # fn main() -> chatpack::Result<()> {
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
//! # Ok(())
//! # }
//! # #[cfg(not(all(feature = "telegram", feature = "streaming")))]
//! # fn main() {}
//! ```
//!
//! ## Module Structure
//!
//! - [`parser`] - **Unified parser API** (recommended)
//!   - [`Parser`](parser::Parser) - Unified parser trait with streaming
//!   - [`Platform`](parser::Platform) - Supported platforms enum
//!   - [`create_parser`](parser::create_parser), [`create_streaming_parser`](parser::create_streaming_parser)
//! - [`config`] - Parser configuration types
//!   - [`TelegramConfig`](config::TelegramConfig), [`WhatsAppConfig`](config::WhatsAppConfig), etc.
//! - [`core`] - Core types and functionality
//!   - [`core::models`] - [`InternalMessage`], [`OutputConfig`]
//!   - [`core::filter`] - [`FilterConfig`], [`apply_filters`]
//!   - [`core::processor`] - [`merge_consecutive`], [`ProcessingStats`]
//!   - [`core::output`] - [`write_json`], [`write_jsonl`], [`write_csv`]
//! - [`parsers`] - Platform-specific parser implementations
//!   - [`TelegramParser`], [`WhatsAppParser`], [`InstagramParser`], [`DiscordParser`]
//! - [`streaming`] - Streaming parsers for large files (requires `streaming` feature)
//!   - [`TelegramStreamingParser`], [`DiscordStreamingParser`]
//! - [`format`] - Output format types
//!   - [`OutputFormat`](format::OutputFormat), [`write_to_format`](format::write_to_format)
//! - [`progress`] - Progress reporting for long-running operations
//!   - [`Progress`](progress::Progress), [`ProgressCallback`](progress::ProgressCallback)
//! - [`error`] - Unified error types ([`ChatpackError`], [`Result`])
//! - [`prelude`] - Convenient re-exports

// Core modules (always available)
pub mod config;
pub mod core;
pub mod error;
pub mod format;
pub mod message;
pub mod progress;

// Parser modules - require at least one parser feature
#[cfg(any(
    feature = "telegram",
    feature = "whatsapp",
    feature = "instagram",
    feature = "discord"
))]
pub mod parser;

#[cfg(any(
    feature = "telegram",
    feature = "whatsapp",
    feature = "instagram",
    feature = "discord"
))]
pub mod parsers;

// Streaming module (requires streaming feature and at least one parser)
#[cfg(all(
    feature = "streaming",
    any(
        feature = "telegram",
        feature = "whatsapp",
        feature = "instagram",
        feature = "discord"
    )
))]
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

    // New unified parser API (recommended)
    #[cfg(any(
        feature = "telegram",
        feature = "whatsapp",
        feature = "instagram",
        feature = "discord"
    ))]
    pub use crate::parser::{Parser, Platform, create_parser, create_streaming_parser};

    // Platform configs
    pub use crate::config::{DiscordConfig, InstagramConfig, TelegramConfig, WhatsAppConfig};

    // Models
    pub use crate::core::models::{InternalMessage, OutputConfig};

    // Filtering
    pub use crate::core::filter::{FilterConfig, apply_filters};

    // Processing
    pub use crate::core::processor::{ProcessingStats, merge_consecutive};

    // Output format
    pub use crate::format::OutputFormat;
    #[cfg(any(feature = "csv-output", feature = "json-output"))]
    pub use crate::format::{to_format_string, write_to_format};

    // Output (file writers and string converters)
    #[cfg(feature = "csv-output")]
    pub use crate::core::output::{to_csv, write_csv};

    #[cfg(feature = "json-output")]
    pub use crate::core::output::{to_json, to_jsonl, write_json, write_jsonl};

    // Progress reporting
    pub use crate::progress::{Progress, ProgressCallback, no_progress};

    // Parsers (implement Parser trait)
    #[cfg(feature = "telegram")]
    pub use crate::parsers::TelegramParser;

    #[cfg(feature = "whatsapp")]
    pub use crate::parsers::WhatsAppParser;

    #[cfg(feature = "instagram")]
    pub use crate::parsers::InstagramParser;

    #[cfg(feature = "discord")]
    pub use crate::parsers::DiscordParser;
}
