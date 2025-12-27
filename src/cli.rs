//! Command-line interface definition using clap.
//!
//! This module defines:
//! - [`Args`] - CLI argument structure (for use with clap)
//! - [`Source`] - Supported chat sources
//! - [`OutputFormat`] - Output format options
//!
//! # Using Source and OutputFormat in Libraries
//!
//! These types are designed to be usable outside of CLI context:
//!
//! ```rust
//! use chatpack::cli::{Source, OutputFormat};
//! use chatpack::parsers::create_parser;
//!
//! // Use Source to create a parser
//! let parser = create_parser(Source::Telegram);
//!
//! // OutputFormat can be converted to/from strings
//! let format = OutputFormat::Csv;
//! println!("Format: {}", format); // "CSV"
//! ```

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

/// Compress chat exports from Telegram, WhatsApp, and Instagram
/// into token-efficient formats for LLMs.
#[derive(Parser, Debug, Clone)]
#[command(name = "chatpack")]
#[command(version, about, long_about = None)]
#[command(after_help = "EXAMPLES:
    chatpack telegram result.json
    chatpack tg chat.json -o optimized.csv
    chatpack wa whatsapp_chat.txt --after 2024-01-01
    chatpack ig messages.json --format jsonl
    chatpack discord chat.json
    chatpack dc chat.txt")]
pub struct Args {
    /// Chat source type
    #[arg(value_enum)]
    pub source: Source,

    /// Path to input file
    pub input: String,

    /// Path to output file
    #[arg(short, long, default_value = "optimized_chat.csv")]
    pub output: String,

    /// Output format
    #[arg(short, long, value_enum, default_value = "csv")]
    pub format: OutputFormat,

    /// Filter messages after this date (YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    pub after: Option<String>,

    /// Filter messages before this date (YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    pub before: Option<String>,

    /// Include timestamps in output
    #[arg(short = 't', long)]
    pub timestamps: bool,

    /// Include reply references in output
    #[arg(short = 'r', long)]
    pub replies: bool,

    /// Include message IDs in output
    #[arg(long)]
    pub ids: bool,

    /// Include edit timestamps in output
    #[arg(short = 'e', long)]
    pub edited: bool,

    /// Disable merging consecutive messages from same sender
    #[arg(long)]
    pub no_merge: bool,

    /// Filter messages from specific user
    #[arg(long, value_name = "USER")]
    pub from: Option<String>,

    #[arg(long)]
    pub streaming: bool,
}

/// Supported chat sources.
///
/// Each source corresponds to a specific export format:
/// - [`Telegram`](Source::Telegram) - JSON export from Telegram Desktop
/// - [`WhatsApp`](Source::WhatsApp) - TXT export from `WhatsApp`
/// - [`Instagram`](Source::Instagram) - JSON export from Instagram
/// - [`Discord`](Source::Discord) - JSON/TXT/CSV export from DiscordChatExporter
///
/// # Example
///
/// ```rust
/// use chatpack::cli::Source;
/// use chatpack::parsers::create_parser;
///
/// let parser = create_parser(Source::Telegram);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// Telegram JSON export
    #[value(alias = "tg")]
    Telegram,

    /// WhatsApp TXT export
    #[value(alias = "wa")]
    #[serde(alias = "wa")]
    WhatsApp,

    /// Instagram JSON export
    #[value(alias = "ig")]
    #[serde(alias = "ig")]
    Instagram,

    /// Discord JSON/TXT/CSV export (from DiscordChatExporter)
    #[value(alias = "dc")]
    #[serde(alias = "dc")]
    Discord,
}

impl Source {
    /// Returns the default file extension for this source.
    pub fn default_extension(&self) -> &'static str {
        match self {
            Source::WhatsApp => "txt",
            Source::Telegram | Source::Instagram | Source::Discord => "json",
        }
    }

    /// Returns all supported source names (including aliases).
    pub fn all_names() -> &'static [&'static str] {
        &[
            "telegram",
            "tg",
            "whatsapp",
            "wa",
            "instagram",
            "ig",
            "discord",
            "dc",
        ]
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Telegram => write!(f, "Telegram"),
            Source::WhatsApp => write!(f, "WhatsApp"),
            Source::Instagram => write!(f, "Instagram"),
            Source::Discord => write!(f, "Discord"),
        }
    }
}

impl std::str::FromStr for Source {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "telegram" | "tg" => Ok(Source::Telegram),
            "whatsapp" | "wa" => Ok(Source::WhatsApp),
            "instagram" | "ig" => Ok(Source::Instagram),
            "discord" | "dc" => Ok(Source::Discord),
            _ => Err(format!(
                "Unknown source: '{}'. Expected one of: {}",
                s,
                Source::all_names().join(", ")
            )),
        }
    }
}

/// Output format options.
///
/// Different formats serve different purposes:
/// - [`Csv`](OutputFormat::Csv) - Best for LLM context (13x compression)
/// - [`Json`](OutputFormat::Json) - Structured array, good for APIs
/// - [`Jsonl`](OutputFormat::Jsonl) - One JSON per line, ideal for RAG/ML
///
/// # Example
///
/// ```rust
/// use chatpack::cli::OutputFormat;
///
/// let format = OutputFormat::Jsonl;
/// println!("Extension: {}", format.extension()); // "jsonl"
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// CSV with semicolon delimiter (default, best for LLMs)
    #[default]
    Csv,

    /// JSON array of messages
    Json,

    /// JSON Lines - one JSON object per line (ideal for ML/RAG)
    Jsonl,
}

impl OutputFormat {
    /// Returns the file extension for this format (without dot).
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Csv => "csv",
            OutputFormat::Json => "json",
            OutputFormat::Jsonl => "jsonl",
        }
    }

    /// Returns all supported format names.
    pub fn all_names() -> &'static [&'static str] {
        &["csv", "json", "jsonl"]
    }

    /// Returns the MIME type for this format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            OutputFormat::Csv => "text/csv",
            OutputFormat::Json => "application/json",
            OutputFormat::Jsonl => "application/x-ndjson",
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Csv => write!(f, "CSV"),
            OutputFormat::Json => write!(f, "JSON"),
            OutputFormat::Jsonl => write!(f, "JSONL"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(OutputFormat::Csv),
            "json" => Ok(OutputFormat::Json),
            "jsonl" | "ndjson" => Ok(OutputFormat::Jsonl),
            _ => Err(format!(
                "Unknown format: '{}'. Expected one of: {}",
                s,
                OutputFormat::all_names().join(", ")
            )),
        }
    }
}

// Conversion to library format type
impl From<OutputFormat> for crate::format::OutputFormat {
    fn from(format: OutputFormat) -> crate::format::OutputFormat {
        match format {
            OutputFormat::Csv => crate::format::OutputFormat::Csv,
            OutputFormat::Json => crate::format::OutputFormat::Json,
            OutputFormat::Jsonl => crate::format::OutputFormat::Jsonl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_display() {
        assert_eq!(Source::Telegram.to_string(), "Telegram");
        assert_eq!(Source::WhatsApp.to_string(), "WhatsApp");
        assert_eq!(Source::Instagram.to_string(), "Instagram");
        assert_eq!(Source::Discord.to_string(), "Discord");
    }

    #[test]
    fn test_source_from_str() {
        assert_eq!("telegram".parse::<Source>().unwrap(), Source::Telegram);
        assert_eq!("tg".parse::<Source>().unwrap(), Source::Telegram);
        assert_eq!("whatsapp".parse::<Source>().unwrap(), Source::WhatsApp);
        assert_eq!("wa".parse::<Source>().unwrap(), Source::WhatsApp);
        assert_eq!("discord".parse::<Source>().unwrap(), Source::Discord);
        assert_eq!("dc".parse::<Source>().unwrap(), Source::Discord);
        assert!("unknown".parse::<Source>().is_err());
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(OutputFormat::Csv.extension(), "csv");
        assert_eq!(OutputFormat::Json.extension(), "json");
        assert_eq!(OutputFormat::Jsonl.extension(), "jsonl");
    }

    #[test]
    fn test_format_from_str() {
        assert_eq!("csv".parse::<OutputFormat>().unwrap(), OutputFormat::Csv);
        assert_eq!(
            "jsonl".parse::<OutputFormat>().unwrap(),
            OutputFormat::Jsonl
        );
        assert_eq!(
            "ndjson".parse::<OutputFormat>().unwrap(),
            OutputFormat::Jsonl
        );
    }

    #[test]
    fn test_source_serde() {
        let source = Source::Telegram;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"telegram\"");

        let parsed: Source = serde_json::from_str("\"wa\"").unwrap();
        assert_eq!(parsed, Source::WhatsApp);

        let parsed: Source = serde_json::from_str("\"dc\"").unwrap();
        assert_eq!(parsed, Source::Discord);
    }

    #[test]
    fn test_format_serde() {
        let format = OutputFormat::Jsonl;
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, "\"jsonl\"");
    }
}
