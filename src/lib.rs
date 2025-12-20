//! # chatpack
//!
//! Compress chat exports from Telegram, WhatsApp, and Instagram
//! into token-efficient formats for LLMs.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use chatpack::prelude::*;
//!
//! // Parse a Telegram export
//! let parser = create_parser(Source::Telegram);
//! let messages = parser.parse("chat.json").unwrap();
//!
//! // Merge consecutive messages from same sender
//! let merged = merge_consecutive(messages);
//!
//! // Filter by date or sender
//! let config = FilterConfig::new()
//!     .after_date("2024-01-01").unwrap()
//!     .with_user("Alice".to_string());
//! let filtered = apply_filters(merged, &config);
//! ```
//!
//! ## Using Individual Parsers
//!
//! ```rust,no_run
//! use chatpack::parsers::{TelegramParser, WhatsAppParser, InstagramParser, ChatParser};
//!
//! let telegram = TelegramParser::new();
//! let messages = telegram.parse("result.json").unwrap();
//!
//! let whatsapp = WhatsAppParser::new();
//! let messages = whatsapp.parse("chat.txt").unwrap();
//! ```
//!
//! ## Output Formats
//!
//! ```rust,no_run
//! use chatpack::prelude::*;
//!
//! let messages = vec![InternalMessage::new("Alice", "Hello!")];
//! let config = OutputConfig::new().with_timestamps();
//!
//! // Write to different formats
//! write_csv(&messages, "output.csv", &config).unwrap();
//! write_json(&messages, "output.json", &config).unwrap();
//! write_jsonl(&messages, "output.jsonl", &config).unwrap();
//! ```

pub mod cli;
pub mod core;
pub mod parsers;

/// Convenient re-exports for common usage.
///
/// Import everything you need with a single line:
/// ```rust
/// use chatpack::prelude::*;
/// ```
pub mod prelude {
    // === Core Models ===
    pub use crate::core::models::{InternalMessage, OutputConfig};

    // === Filtering ===
    pub use crate::core::filter::{FilterConfig, FilterError, apply_filters};

    // === Processing ===
    pub use crate::core::processor::{ProcessingStats, merge_consecutive};

    // === Output Writers ===
    pub use crate::core::output::{write_csv, write_json, write_jsonl};

    // === Parsers ===
    pub use crate::parsers::{
        ChatParser, InstagramParser, TelegramParser, WhatsAppParser, create_parser,
    };

    // === CLI Types ===
    pub use crate::cli::{OutputFormat, Source};
}

// === Top-level re-exports for ergonomic access ===

// Core types at crate root
pub use core::filter::{FilterConfig, FilterError};
pub use core::models::{InternalMessage, OutputConfig};
pub use core::processor::ProcessingStats;

// Parser trait and factory
pub use parsers::{ChatParser, create_parser};

// CLI types
pub use cli::{OutputFormat, Source};
