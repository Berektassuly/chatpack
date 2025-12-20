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
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
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
//! ## Core Concepts
//!
//! ### Parsing
//!
//! All parsers implement the [`ChatParser`] trait and return a `Vec<InternalMessage>`:
//!
//! ```rust,no_run
//! use chatpack::prelude::*;
//!
//! // Using the factory function (recommended)
//! let parser = create_parser(Source::WhatsApp);
//! let messages = parser.parse("whatsapp_chat.txt").unwrap();
//!
//! // Or use parsers directly
//! let parser = TelegramParser;
//! let messages = parser.parse("telegram_export.json").unwrap();
//! ```
//!
//! ### Auto-Detection
//!
//! If you don't know the source format, use [`parse_auto`]:
//!
//! ```rust,no_run
//! use chatpack::parsers::parse_auto;
//!
//! let messages = parse_auto("unknown_chat.json").unwrap();
//! ```
//!
//! ### Message Structure
//!
//! All messages are normalized to [`InternalMessage`]:
//!
//! ```rust
//! use chatpack::prelude::*;
//! use chrono::{TimeZone, Utc};
//!
//! let msg = InternalMessage::new("Alice", "Hello, world!")
//!     .with_id(42)
//!     .with_timestamp(Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap())
//!     .with_reply_to(41);
//!
//! assert_eq!(msg.sender, "Alice");
//! assert_eq!(msg.content, "Hello, world!");
//! assert!(msg.has_metadata());
//! ```
//!
//! ### Filtering
//!
//! Filter messages by sender or date range:
//!
//! ```rust,no_run
//! use chatpack::prelude::*;
//!
//! let parser = create_parser(Source::Telegram);
//! let messages = parser.parse("chat.json").unwrap();
//!
//! // Filter by sender
//! let config = FilterConfig::new()
//!     .with_user("Alice".to_string());
//! let alice_messages = apply_filters(messages.clone(), &config);
//!
//! // Filter by date range
//! let config = FilterConfig::new()
//!     .after_date("2024-01-01").unwrap()
//!     .before_date("2024-12-31").unwrap();
//! let filtered = apply_filters(messages, &config);
//! ```
//!
//! ### Merging Consecutive Messages
//!
//! Combine consecutive messages from the same sender to reduce token count:
//!
//! ```rust
//! use chatpack::prelude::*;
//!
//! let messages = vec![
//!     InternalMessage::new("Alice", "Hello"),
//!     InternalMessage::new("Alice", "How are you?"),
//!     InternalMessage::new("Bob", "Hi!"),
//!     InternalMessage::new("Bob", "I'm good"),
//! ];
//!
//! let merged = merge_consecutive(messages);
//!
//! assert_eq!(merged.len(), 2);
//! assert_eq!(merged[0].content, "Hello\nHow are you?");
//! assert_eq!(merged[1].content, "Hi!\nI'm good");
//! ```
//!
//! ### Output Formats
//!
//! Write messages to JSON, JSONL, or CSV:
//!
//! ```rust,no_run
//! use chatpack::prelude::*;
//!
//! let messages = vec![
//!     InternalMessage::new("Alice", "Hello"),
//!     InternalMessage::new("Bob", "Hi!"),
//! ];
//!
//! // Minimal output (just sender and content)
//! let config = OutputConfig::new();
//! write_json(&messages, "minimal.json", &config).unwrap();
//!
//! // Full metadata
//! let config = OutputConfig::all();
//! write_json(&messages, "full.json", &config).unwrap();
//!
//! // Custom selection
//! let config = OutputConfig::new()
//!     .with_timestamps()
//!     .with_ids();
//! write_jsonl(&messages, "output.jsonl", &config).unwrap();
//!
//! // CSV format
//! write_csv(&messages, "output.csv", &config).unwrap();
//! ```
//!
//! ### Processing Statistics
//!
//! Track compression ratio and message counts:
//!
//! ```rust
//! use chatpack::prelude::*;
//!
//! let original_count = 100;
//! let merged_count = 45;
//!
//! let stats = ProcessingStats::new(original_count, merged_count);
//!
//! println!("Compression: {:.1}%", stats.compression_ratio());
//! println!("Messages saved: {}", stats.messages_saved());
//! // Output:
//! // Compression: 55.0%
//! // Messages saved: 55
//! ```
//!
//! ## Complete Example
//!
//! ```rust,no_run
//! use chatpack::prelude::*;
//!
//! fn process_chat(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Parse the export
//!     let parser = create_parser(Source::Telegram);
//!     let messages = parser.parse(input)?;
//!     let original_count = messages.len();
//!
//!     // 2. Filter (optional)
//!     let config = FilterConfig::new()
//!         .after_date("2024-01-01")?;
//!     let filtered = apply_filters(messages, &config);
//!     let filtered_count = filtered.len();
//!
//!     // 3. Merge consecutive messages
//!     let merged = merge_consecutive(filtered);
//!
//!     // 4. Output
//!     let output_config = OutputConfig::new()
//!         .with_timestamps();
//!     write_json(&merged, output, &output_config)?;
//!
//!     // 5. Statistics
//!     let stats = ProcessingStats::new(original_count, merged.len())
//!         .with_filtered(filtered_count);
//!     println!("{}", stats);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Supported Export Formats
//!
//! ### Telegram
//!
//! Export from Telegram Desktop:
//! 1. Open chat → ⋮ menu → Export chat history
//! 2. Select JSON format
//! 3. Use with `Source::Telegram` or `TelegramParser`
//!
//! ### WhatsApp
//!
//! Export from WhatsApp:
//! 1. Open chat → ⋮ menu → More → Export chat
//! 2. Choose "Without Media"
//! 3. Use with `Source::WhatsApp` or `WhatsAppParser`
//!
//! Supported formats:
//! - US: `1/15/24, 10:30 AM - Alice: Hello`
//! - EU: `15.01.24, 10:30 - Alice: Hello`
//! - Bracketed: `[15.01.24, 10:30:00] Alice: Hello`
//!
//! ### Instagram
//!
//! Export from Instagram:
//! 1. Settings → Privacy → Download your data
//! 2. Select JSON format
//! 3. Find `messages/inbox/*/message_1.json`
//! 4. Use with `Source::Instagram` or `InstagramParser`
//!
//! ## Module Structure
//!
//! - [`core`] — Core types and functionality
//!   - [`core::models`] — [`InternalMessage`], [`OutputConfig`]
//!   - [`core::filter`] — [`FilterConfig`], [`apply_filters`]
//!   - [`core::processor`] — [`merge_consecutive`], [`ProcessingStats`]
//!   - [`core::output`] — [`write_json`], [`write_jsonl`], [`write_csv`]
//! - [`parsers`] — Chat parsers
//!   - [`TelegramParser`], [`WhatsAppParser`], [`InstagramParser`]
//!   - [`create_parser`], [`parse_auto`]
//! - [`cli`] — CLI types ([`Source`], [`OutputFormat`])
//! - [`prelude`] — Convenient re-exports
//!
//! ## Feature Flags
//!
//! This crate has no optional features. All functionality is available by default.
//!
//! ## Serialization
//!
//! All main types implement `Serialize` and `Deserialize` from serde:
//!
//! ```rust
//! use chatpack::prelude::*;
//!
//! let msg = InternalMessage::new("Alice", "Hello");
//! let json = serde_json::to_string(&msg).unwrap();
//! let restored: InternalMessage = serde_json::from_str(&json).unwrap();
//!
//! assert_eq!(msg, restored);
//! ```

pub mod cli;
pub mod core;
pub mod parsers;

/// Convenient re-exports for common usage.
///
/// Import everything you need with a single line:
///
/// ```rust
/// use chatpack::prelude::*;
/// ```
pub mod prelude {
    // Models
    pub use crate::core::models::{InternalMessage, OutputConfig};

    // Filtering
    pub use crate::core::filter::{apply_filters, FilterConfig, FilterError};

    // Processing
    pub use crate::core::processor::{merge_consecutive, ProcessingStats};

    // Output
    pub use crate::core::output::{write_csv, write_json, write_jsonl};

    // Parsers
    pub use crate::parsers::{
        create_parser, ChatParser, InstagramParser, TelegramParser, WhatsAppParser,
    };

    // CLI types
    pub use crate::cli::{OutputFormat, Source};
}