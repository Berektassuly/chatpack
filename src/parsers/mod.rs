//! Chat export parsers for various platforms.
//!
//! This module provides parsers for chat exports from different messaging platforms.
//! Each parser implements the [`Parser`] trait from [`crate::parser`].
//!
//! # Available Parsers
//!
//! - [`TelegramParser`] - Parses Telegram JSON exports (requires `telegram` feature)
//! - [`WhatsAppParser`] - Parses WhatsApp TXT exports (requires `whatsapp` feature)
//! - [`InstagramParser`] - Parses Instagram JSON exports (requires `instagram` feature)
//! - [`DiscordParser`] - Parses Discord JSON/TXT/CSV exports (requires `discord` feature)
//!
//! # Usage
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
