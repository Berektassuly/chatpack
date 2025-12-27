//! Platform-specific chat export parsers.
//!
//! This module provides parser implementations for each supported messaging platform.
//! All parsers implement the [`Parser`] trait, providing a consistent interface.
//!
//! # Available Parsers
//!
//! | Parser | Feature | Export Format | Special Handling |
//! |--------|---------|---------------|------------------|
//! | [`TelegramParser`] | `telegram` | JSON | Service messages, forwards |
//! | [`WhatsAppParser`] | `whatsapp` | TXT | Auto-detects 4 date formats |
//! | [`InstagramParser`] | `instagram` | JSON | Fixes Mojibake encoding |
//! | [`DiscordParser`] | `discord` | JSON/TXT/CSV | Attachments, stickers |
//!
//! # Examples
//!
//! ## Direct Parser Usage
//!
//! ```no_run
//! # #[cfg(feature = "telegram")]
//! # fn main() -> chatpack::Result<()> {
//! use chatpack::parser::Parser;
//! use chatpack::parsers::TelegramParser;
//!
//! let parser = TelegramParser::new();
//! let messages = parser.parse("result.json".as_ref())?;
//!
//! println!("Parsed {} messages", messages.len());
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "telegram"))]
//! # fn main() {}
//! ```
//!
//! ## Dynamic Parser Selection
//!
//! ```no_run
//! # #[cfg(feature = "telegram")]
//! # fn main() -> chatpack::Result<()> {
//! use chatpack::parser::{Parser, Platform, create_parser};
//!
//! let parser = create_parser(Platform::Telegram);
//! let messages = parser.parse("result.json".as_ref())?;
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "telegram"))]
//! # fn main() {}
//! ```
//!
//! ## Streaming Large Files
//!
//! ```no_run
//! # #[cfg(all(feature = "telegram", feature = "streaming"))]
//! # fn main() -> chatpack::Result<()> {
//! use chatpack::parser::{Parser, Platform, create_streaming_parser};
//!
//! let parser = create_streaming_parser(Platform::Telegram);
//! for result in parser.stream("large_export.json".as_ref())? {
//!     let msg = result?;
//!     // Process one message at a time
//! }
//! # Ok(())
//! # }
//! # #[cfg(not(all(feature = "telegram", feature = "streaming")))]
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

// Re-export the unified Parser trait and Platform
pub use crate::parser::{Parser, Platform};
