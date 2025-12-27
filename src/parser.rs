//! Unified parser trait for chat exports.
//!
//! This module provides a single entry point for parsing chat exports, with support
//! for both in-memory and streaming modes.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "telegram")]
//! # fn main() -> chatpack::Result<()> {
//! use chatpack::parser::{Parser, Platform};
//! use chatpack::parsers::TelegramParser;
//! use std::path::Path;
//!
//! let parser = TelegramParser::new();
//!
//! // Parse entire file into memory
//! let messages = parser.parse(Path::new("chat_export.json"))?;
//!
//! // Or stream for large files
//! for result in parser.stream(Path::new("large_export.json"))? {
//!     if let Ok(msg) = result {
//!         println!("{}: {}", msg.sender, msg.content);
//!     }
//! }
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "telegram"))]
//! # fn main() {}
//! ```
//!
//! # Platform Selection
//!
//! Use [`Platform`] enum to dynamically select parsers:
//!
//! ```rust
//! # #[cfg(feature = "telegram")]
//! # fn main() {
//! use chatpack::parser::{Platform, create_parser};
//!
//! let parser = create_parser(Platform::Telegram);
//! // parser.parse("file.json")?;
//! # }
//! # #[cfg(not(feature = "telegram"))]
//! # fn main() {}
//! ```

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::ChatpackError;
use crate::Message;

#[cfg(feature = "streaming")]
use crate::streaming::MessageIterator;

/// Supported messaging platforms.
///
/// This enum identifies the source platform for chat exports, enabling
/// dynamic parser selection without CLI dependencies.
///
/// # Example
///
/// ```rust
/// use chatpack::parser::Platform;
/// use std::str::FromStr;
///
/// let platform = Platform::from_str("telegram").unwrap();
/// assert_eq!(platform, Platform::Telegram);
///
/// // Aliases are supported
/// let platform = Platform::from_str("tg").unwrap();
/// assert_eq!(platform, Platform::Telegram);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Platform {
    /// Telegram JSON exports from Telegram Desktop
    #[serde(alias = "tg")]
    Telegram,

    /// WhatsApp TXT exports (iOS and Android)
    #[serde(alias = "wa")]
    WhatsApp,

    /// Instagram JSON exports from data download
    #[serde(alias = "ig")]
    Instagram,

    /// Discord exports from DiscordChatExporter (JSON/TXT/CSV)
    #[serde(alias = "dc")]
    Discord,
}

impl Platform {
    /// Returns the default file extension for exports from this platform.
    pub fn default_extension(&self) -> &'static str {
        match self {
            Platform::WhatsApp => "txt",
            Platform::Telegram | Platform::Instagram | Platform::Discord => "json",
        }
    }

    /// Returns all platform names including aliases.
    pub fn all_names() -> &'static [&'static str] {
        &[
            "telegram", "tg", "whatsapp", "wa", "instagram", "ig", "discord", "dc",
        ]
    }

    /// Returns all available platforms.
    pub fn all() -> &'static [Platform] {
        &[
            Platform::Telegram,
            Platform::WhatsApp,
            Platform::Instagram,
            Platform::Discord,
        ]
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Telegram => write!(f, "Telegram"),
            Platform::WhatsApp => write!(f, "WhatsApp"),
            Platform::Instagram => write!(f, "Instagram"),
            Platform::Discord => write!(f, "Discord"),
        }
    }
}

impl std::str::FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "telegram" | "tg" => Ok(Platform::Telegram),
            "whatsapp" | "wa" => Ok(Platform::WhatsApp),
            "instagram" | "ig" => Ok(Platform::Instagram),
            "discord" | "dc" => Ok(Platform::Discord),
            _ => Err(format!(
                "Unknown platform: '{}'. Expected one of: {}",
                s,
                Platform::all_names().join(", ")
            )),
        }
    }
}

// Conversion from CLI Source to Platform (only with cli feature)
#[cfg(feature = "cli")]
impl From<crate::cli::Source> for Platform {
    fn from(source: crate::cli::Source) -> Self {
        match source {
            crate::cli::Source::Telegram => Platform::Telegram,
            crate::cli::Source::WhatsApp => Platform::WhatsApp,
            crate::cli::Source::Instagram => Platform::Instagram,
            crate::cli::Source::Discord => Platform::Discord,
        }
    }
}

/// Iterator adapter that wraps StreamingError into ChatpackError.
#[cfg(feature = "streaming")]
pub struct ParseIterator {
    inner: Box<dyn MessageIterator>,
}

#[cfg(feature = "streaming")]
impl ParseIterator {
    /// Creates a new parse iterator from a message iterator.
    pub fn new(inner: Box<dyn MessageIterator>) -> Self {
        Self { inner }
    }

    /// Returns the progress as a percentage (0.0 - 100.0).
    pub fn progress(&self) -> Option<f64> {
        self.inner.progress()
    }

    /// Returns the number of bytes processed so far.
    pub fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }

    /// Returns the total file size in bytes, if known.
    pub fn total_bytes(&self) -> Option<u64> {
        self.inner.total_bytes()
    }
}

#[cfg(feature = "streaming")]
impl Iterator for ParseIterator {
    type Item = Result<Message, ChatpackError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| result.map_err(ChatpackError::from))
    }
}

/// Unified trait for parsing chat exports.
///
/// This trait combines the functionality of the previous `ChatParser` and
/// `StreamingParser` traits into a single, cohesive API.
///
/// # Implementation Notes
///
/// Parsers must implement:
/// - [`name`](Parser::name) - Parser identifier
/// - [`platform`](Parser::platform) - Platform this parser handles
/// - [`parse`](Parser::parse) - Load entire file into memory
/// - [`parse_str`](Parser::parse_str) - Parse from a string
///
/// Optionally override:
/// - [`stream`](Parser::stream) - Streaming for large files (default: falls back to parse)
/// - [`supports_streaming`](Parser::supports_streaming) - Whether native streaming is supported
///
/// # Example Implementation
///
/// ```rust,ignore
/// impl Parser for MyParser {
///     fn name(&self) -> &'static str { "MyParser" }
///     fn platform(&self) -> Platform { Platform::Telegram }
///
///     fn parse(&self, path: &Path) -> Result<Vec<Message>, ChatpackError> {
///         let content = std::fs::read_to_string(path)?;
///         self.parse_str(&content)
///     }
///
///     fn parse_str(&self, content: &str) -> Result<Vec<Message>, ChatpackError> {
///         // Parse logic here
///         Ok(vec![])
///     }
/// }
/// ```
pub trait Parser: Send + Sync {
    /// Returns the human-readable name of this parser.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "telegram")]
    /// # fn main() {
    /// use chatpack::parser::Parser;
    /// use chatpack::parsers::TelegramParser;
    ///
    /// let parser = TelegramParser::new();
    /// assert_eq!(parser.name(), "Telegram");
    /// # }
    /// # #[cfg(not(feature = "telegram"))]
    /// # fn main() {}
    /// ```
    fn name(&self) -> &'static str;

    /// Returns the platform this parser handles.
    fn platform(&self) -> Platform;

    /// Parses a chat export file and returns all messages.
    ///
    /// This method loads the entire file into memory, which is suitable
    /// for files up to ~500MB. For larger files, use [`stream`](Parser::stream).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the export file
    ///
    /// # Errors
    ///
    /// Returns [`ChatpackError`] if:
    /// - File cannot be read ([`ChatpackError::Io`])
    /// - Content cannot be parsed ([`ChatpackError::Parse`])
    fn parse(&self, path: &Path) -> Result<Vec<Message>, ChatpackError>;

    /// Parses chat content from a string.
    ///
    /// This is useful for:
    /// - WASM environments without file system access
    /// - Testing with inline data
    /// - Processing content already in memory
    ///
    /// # Arguments
    ///
    /// * `content` - Raw export content as a string
    ///
    /// # Errors
    ///
    /// Returns [`ChatpackError::Parse`] if content cannot be parsed.
    fn parse_str(&self, content: &str) -> Result<Vec<Message>, ChatpackError>;

    /// Parses a chat export file (convenience method accepting &str path).
    ///
    /// This is equivalent to `parse(Path::new(path))`.
    fn parse_file(&self, path: &str) -> Result<Vec<Message>, ChatpackError> {
        self.parse(Path::new(path))
    }

    /// Streams messages from a file for memory-efficient processing.
    ///
    /// By default, this falls back to loading the entire file and returning
    /// an iterator over the messages. Parsers that support native streaming
    /// should override this method.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the export file
    ///
    /// # Returns
    ///
    /// An iterator that yields `Result<Message, ChatpackError>` for each message.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "telegram")]
    /// # fn main() -> chatpack::Result<()> {
    /// use chatpack::parser::Parser;
    /// use chatpack::parsers::TelegramParser;
    /// use std::path::Path;
    ///
    /// let parser = TelegramParser::new();
    /// for result in parser.stream(Path::new("large_file.json"))? {
    ///     if let Ok(msg) = result {
    ///         println!("{}: {}", msg.sender, msg.content);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// # #[cfg(not(feature = "telegram"))]
    /// # fn main() {}
    /// ```
    fn stream(&self, path: &Path) -> Result<Box<dyn Iterator<Item = Result<Message, ChatpackError>> + Send>, ChatpackError> {
        // Default implementation: load everything into memory
        let messages = self.parse(path)?;
        Ok(Box::new(messages.into_iter().map(Ok)))
    }

    /// Streams messages (convenience method accepting &str path).
    fn stream_file(&self, path: &str) -> Result<Box<dyn Iterator<Item = Result<Message, ChatpackError>> + Send>, ChatpackError> {
        self.stream(Path::new(path))
    }

    /// Returns whether this parser supports native streaming.
    ///
    /// If `false`, the [`stream`](Parser::stream) method will load the entire
    /// file into memory before iterating.
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Returns the recommended buffer size for streaming.
    ///
    /// Only relevant if [`supports_streaming`](Parser::supports_streaming) returns `true`.
    fn recommended_buffer_size(&self) -> usize {
        64 * 1024 // 64KB default
    }
}

/// Creates a parser for the specified platform.
///
/// This is the main entry point for dynamic parser creation.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "telegram")]
/// # fn main() {
/// use chatpack::parser::{Platform, create_parser};
///
/// let parser = create_parser(Platform::Telegram);
/// assert_eq!(parser.name(), "Telegram");
/// # }
/// # #[cfg(not(feature = "telegram"))]
/// # fn main() {}
/// ```
///
/// # Panics
///
/// Panics if the corresponding parser feature is not enabled.
pub fn create_parser(platform: Platform) -> Box<dyn Parser> {
    match platform {
        #[cfg(feature = "telegram")]
        Platform::Telegram => Box::new(crate::parsers::TelegramParser::new()),
        #[cfg(feature = "whatsapp")]
        Platform::WhatsApp => Box::new(crate::parsers::WhatsAppParser::new()),
        #[cfg(feature = "instagram")]
        Platform::Instagram => Box::new(crate::parsers::InstagramParser::new()),
        #[cfg(feature = "discord")]
        Platform::Discord => Box::new(crate::parsers::DiscordParser::new()),
        // Fallback for when features are disabled
        #[allow(unreachable_patterns)]
        _ => panic!(
            "Parser for {:?} is not enabled. Enable the corresponding feature.",
            platform
        ),
    }
}

/// Creates a parser for the specified platform with streaming support.
///
/// This creates a parser configured for optimal streaming performance.
/// All platforms now support streaming.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "telegram")]
/// # fn main() -> chatpack::Result<()> {
/// use chatpack::parser::{Platform, create_streaming_parser};
///
/// let parser = create_streaming_parser(Platform::Telegram);
/// for result in parser.stream("large_file.json".as_ref())? {
///     // Process each message
/// }
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "telegram"))]
/// # fn main() {}
/// ```
///
/// # Panics
///
/// Panics if the corresponding parser feature is not enabled.
pub fn create_streaming_parser(platform: Platform) -> Box<dyn Parser> {
    match platform {
        #[cfg(feature = "telegram")]
        Platform::Telegram => Box::new(crate::parsers::TelegramParser::with_streaming()),
        #[cfg(feature = "whatsapp")]
        Platform::WhatsApp => Box::new(crate::parsers::WhatsAppParser::with_streaming()),
        #[cfg(feature = "instagram")]
        Platform::Instagram => Box::new(crate::parsers::InstagramParser::with_streaming()),
        #[cfg(feature = "discord")]
        Platform::Discord => Box::new(crate::parsers::DiscordParser::with_streaming()),
        // Fallback for when features are disabled
        #[allow(unreachable_patterns)]
        _ => panic!(
            "Streaming parser for {:?} is not enabled. Enable the corresponding feature.",
            platform
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_platform_from_str() {
        assert_eq!(Platform::from_str("telegram").unwrap(), Platform::Telegram);
        assert_eq!(Platform::from_str("tg").unwrap(), Platform::Telegram);
        assert_eq!(Platform::from_str("TELEGRAM").unwrap(), Platform::Telegram);
        assert_eq!(Platform::from_str("whatsapp").unwrap(), Platform::WhatsApp);
        assert_eq!(Platform::from_str("wa").unwrap(), Platform::WhatsApp);
        assert_eq!(Platform::from_str("instagram").unwrap(), Platform::Instagram);
        assert_eq!(Platform::from_str("ig").unwrap(), Platform::Instagram);
        assert_eq!(Platform::from_str("discord").unwrap(), Platform::Discord);
        assert_eq!(Platform::from_str("dc").unwrap(), Platform::Discord);
    }

    #[test]
    fn test_platform_from_str_error() {
        assert!(Platform::from_str("unknown").is_err());
    }

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::Telegram.to_string(), "Telegram");
        assert_eq!(Platform::WhatsApp.to_string(), "WhatsApp");
        assert_eq!(Platform::Instagram.to_string(), "Instagram");
        assert_eq!(Platform::Discord.to_string(), "Discord");
    }

    #[test]
    fn test_platform_default_extension() {
        assert_eq!(Platform::Telegram.default_extension(), "json");
        assert_eq!(Platform::WhatsApp.default_extension(), "txt");
        assert_eq!(Platform::Instagram.default_extension(), "json");
        assert_eq!(Platform::Discord.default_extension(), "json");
    }

    #[test]
    fn test_platform_all() {
        let all = Platform::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&Platform::Telegram));
        assert!(all.contains(&Platform::WhatsApp));
        assert!(all.contains(&Platform::Instagram));
        assert!(all.contains(&Platform::Discord));
    }

    #[cfg(feature = "telegram")]
    #[test]
    fn test_create_parser() {
        let parser = create_parser(Platform::Telegram);
        assert_eq!(parser.name(), "Telegram");
        assert_eq!(parser.platform(), Platform::Telegram);
    }
}
