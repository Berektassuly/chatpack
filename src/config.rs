//! Configuration types for parsers and output.
//!
//! This module provides clean configuration structs for library usage,
//! without any CLI framework dependencies.
//!
//! # Parser Configurations
//!
//! Each platform has its own configuration struct:
//!
//! - [`TelegramConfig`] - Telegram JSON export settings
//! - [`WhatsAppConfig`] - WhatsApp TXT export settings
//! - [`InstagramConfig`] - Instagram JSON export settings
//! - [`DiscordConfig`] - Discord multi-format export settings
//!
//! # Example
//!
//! ```rust
//! use chatpack::config::TelegramConfig;
//! use chatpack::parsers::TelegramParser;
//!
//! let config = TelegramConfig::new()
//!     .with_streaming(true)
//!     .with_buffer_size(128 * 1024);
//!
//! let parser = TelegramParser::with_config(config);
//! ```

use serde::{Deserialize, Serialize};

/// Configuration for Telegram export parsing.
///
/// Telegram exports are JSON files with a `messages` array. This config
/// controls how the parser handles large files and invalid data.
///
/// # Example
///
/// ```rust
/// use chatpack::config::TelegramConfig;
///
/// let config = TelegramConfig::new()
///     .with_streaming(true)
///     .with_skip_invalid(true);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    /// Enable streaming mode for large files (default: false)
    pub streaming: bool,

    /// Buffer size for streaming (default: 64KB)
    pub buffer_size: usize,

    /// Maximum message size in bytes (default: 10MB)
    pub max_message_size: usize,

    /// Skip invalid messages instead of returning errors (default: true)
    pub skip_invalid: bool,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            streaming: false,
            buffer_size: 64 * 1024,             // 64KB
            max_message_size: 10 * 1024 * 1024, // 10MB
            skip_invalid: true,
        }
    }
}

impl TelegramConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a streaming-optimized configuration.
    pub fn streaming() -> Self {
        Self {
            streaming: true,
            buffer_size: 256 * 1024, // 256KB for streaming
            ..Self::default()
        }
    }

    /// Enables or disables streaming mode.
    #[must_use]
    pub fn with_streaming(mut self, enabled: bool) -> Self {
        self.streaming = enabled;
        self
    }

    /// Sets the buffer size for streaming.
    #[must_use]
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Sets the maximum message size.
    #[must_use]
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Sets whether to skip invalid messages.
    #[must_use]
    pub fn with_skip_invalid(mut self, skip: bool) -> Self {
        self.skip_invalid = skip;
        self
    }
}

/// Configuration for WhatsApp export parsing.
///
/// WhatsApp exports are TXT files with various locale-specific date formats.
/// The parser auto-detects the format by analyzing the first 20 lines.
///
/// # Example
///
/// ```rust
/// use chatpack::config::WhatsAppConfig;
///
/// let config = WhatsAppConfig::new()
///     .with_skip_system_messages(true);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    /// Enable streaming mode for large files (default: false)
    pub streaming: bool,

    /// Buffer size for streaming (default: 64KB)
    pub buffer_size: usize,

    /// Skip system messages (user added/removed, etc.) (default: true)
    pub skip_system_messages: bool,

    /// Skip invalid messages instead of returning errors (default: true)
    pub skip_invalid: bool,
}

impl Default for WhatsAppConfig {
    fn default() -> Self {
        Self {
            streaming: false,
            buffer_size: 64 * 1024, // 64KB
            skip_system_messages: true,
            skip_invalid: true,
        }
    }
}

impl WhatsAppConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a streaming-optimized configuration.
    pub fn streaming() -> Self {
        Self {
            streaming: true,
            buffer_size: 256 * 1024, // 256KB for streaming
            ..Self::default()
        }
    }

    /// Enables or disables streaming mode.
    #[must_use]
    pub fn with_streaming(mut self, enabled: bool) -> Self {
        self.streaming = enabled;
        self
    }

    /// Sets the buffer size for streaming.
    #[must_use]
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Sets whether to skip system messages.
    #[must_use]
    pub fn with_skip_system_messages(mut self, skip: bool) -> Self {
        self.skip_system_messages = skip;
        self
    }

    /// Sets whether to skip invalid messages.
    #[must_use]
    pub fn with_skip_invalid(mut self, skip: bool) -> Self {
        self.skip_invalid = skip;
        self
    }
}

/// Configuration for Instagram export parsing.
///
/// Instagram exports are JSON files with Mojibake encoding issues.
/// The parser automatically fixes the encoding.
///
/// # Example
///
/// ```rust
/// use chatpack::config::InstagramConfig;
///
/// let config = InstagramConfig::new()
///     .with_fix_encoding(true);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstagramConfig {
    /// Enable streaming mode for large files (default: false)
    pub streaming: bool,

    /// Buffer size for streaming (default: 64KB)
    pub buffer_size: usize,

    /// Maximum message size in bytes (default: 10MB)
    pub max_message_size: usize,

    /// Fix Meta's broken UTF-8 encoding (Mojibake) (default: true)
    pub fix_encoding: bool,

    /// Skip invalid messages instead of returning errors (default: true)
    pub skip_invalid: bool,
}

impl Default for InstagramConfig {
    fn default() -> Self {
        Self {
            streaming: false,
            buffer_size: 64 * 1024,             // 64KB
            max_message_size: 10 * 1024 * 1024, // 10MB
            fix_encoding: true,
            skip_invalid: true,
        }
    }
}

impl InstagramConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a streaming-optimized configuration.
    pub fn streaming() -> Self {
        Self {
            streaming: true,
            buffer_size: 256 * 1024, // 256KB for streaming
            ..Self::default()
        }
    }

    /// Enables or disables streaming mode.
    #[must_use]
    pub fn with_streaming(mut self, enabled: bool) -> Self {
        self.streaming = enabled;
        self
    }

    /// Sets the buffer size for streaming.
    #[must_use]
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Sets the maximum message size.
    #[must_use]
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Enables or disables encoding fix.
    #[must_use]
    pub fn with_fix_encoding(mut self, fix: bool) -> Self {
        self.fix_encoding = fix;
        self
    }

    /// Sets whether to skip invalid messages.
    #[must_use]
    pub fn with_skip_invalid(mut self, skip: bool) -> Self {
        self.skip_invalid = skip;
        self
    }
}

/// Configuration for Discord export parsing.
///
/// Discord exports can be in JSON, TXT, or CSV format (from DiscordChatExporter).
/// The parser auto-detects the format from file extension or content.
///
/// # Example
///
/// ```rust
/// use chatpack::config::DiscordConfig;
///
/// let config = DiscordConfig::new()
///     .with_prefer_nickname(true);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    /// Enable streaming mode for large files (default: false)
    pub streaming: bool,

    /// Buffer size for streaming (default: 64KB)
    pub buffer_size: usize,

    /// Maximum message size in bytes (default: 10MB)
    pub max_message_size: usize,

    /// Prefer nickname over username when available (default: true)
    pub prefer_nickname: bool,

    /// Include attachment/sticker information (default: true)
    pub include_attachments: bool,

    /// Skip invalid messages instead of returning errors (default: true)
    pub skip_invalid: bool,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            streaming: false,
            buffer_size: 64 * 1024,             // 64KB
            max_message_size: 10 * 1024 * 1024, // 10MB
            prefer_nickname: true,
            include_attachments: true,
            skip_invalid: true,
        }
    }
}

impl DiscordConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a streaming-optimized configuration.
    pub fn streaming() -> Self {
        Self {
            streaming: true,
            buffer_size: 256 * 1024, // 256KB for streaming
            ..Self::default()
        }
    }

    /// Enables or disables streaming mode.
    #[must_use]
    pub fn with_streaming(mut self, enabled: bool) -> Self {
        self.streaming = enabled;
        self
    }

    /// Sets the buffer size for streaming.
    #[must_use]
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Sets the maximum message size.
    #[must_use]
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Sets whether to prefer nickname over username.
    #[must_use]
    pub fn with_prefer_nickname(mut self, prefer: bool) -> Self {
        self.prefer_nickname = prefer;
        self
    }

    /// Sets whether to include attachments in message content.
    #[must_use]
    pub fn with_include_attachments(mut self, include: bool) -> Self {
        self.include_attachments = include;
        self
    }

    /// Sets whether to skip invalid messages.
    #[must_use]
    pub fn with_skip_invalid(mut self, skip: bool) -> Self {
        self.skip_invalid = skip;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_config_default() {
        let config = TelegramConfig::default();
        assert!(!config.streaming);
        assert_eq!(config.buffer_size, 64 * 1024);
        assert!(config.skip_invalid);
    }

    #[test]
    fn test_telegram_config_builder() {
        let config = TelegramConfig::new()
            .with_streaming(true)
            .with_buffer_size(128 * 1024);

        assert!(config.streaming);
        assert_eq!(config.buffer_size, 128 * 1024);
    }

    #[test]
    fn test_telegram_config_streaming() {
        let config = TelegramConfig::streaming();
        assert!(config.streaming);
        assert_eq!(config.buffer_size, 256 * 1024);
    }

    #[test]
    fn test_whatsapp_config_default() {
        let config = WhatsAppConfig::default();
        assert!(!config.streaming);
        assert!(config.skip_system_messages);
    }

    #[test]
    fn test_instagram_config_default() {
        let config = InstagramConfig::default();
        assert!(!config.streaming);
        assert!(config.fix_encoding);
    }

    #[test]
    fn test_discord_config_default() {
        let config = DiscordConfig::default();
        assert!(!config.streaming);
        assert!(config.prefer_nickname);
        assert!(config.include_attachments);
    }
}
