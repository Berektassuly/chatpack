//! Shared parsing utilities for all platforms.
//!
//! This module contains common types and functions used by both
//! standard (in-memory) and streaming parsers to avoid code duplication.

#[cfg(feature = "telegram")]
pub mod telegram;

#[cfg(feature = "instagram")]
pub mod instagram;

#[cfg(feature = "whatsapp")]
pub mod whatsapp;

#[cfg(feature = "discord")]
pub mod discord;

// Re-export commonly used items
#[cfg(feature = "telegram")]
pub use telegram::{TelegramRawMessage, extract_telegram_text, parse_telegram_message};

#[cfg(feature = "instagram")]
pub use instagram::{
    InstagramRawMessage, fix_mojibake_encoding, parse_instagram_message,
    parse_instagram_message_owned,
};

#[cfg(feature = "whatsapp")]
pub use whatsapp::{
    DateFormat as WhatsAppDateFormat, detect_whatsapp_format, is_whatsapp_system_message,
    parse_whatsapp_timestamp,
};

#[cfg(feature = "discord")]
pub use discord::{DiscordRawMessage, parse_discord_message};
