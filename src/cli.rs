//! Command-line interface definition using clap.

use clap::{Parser, ValueEnum};

/// Compress chat exports from Telegram, WhatsApp, and Instagram
/// into token-efficient formats for LLMs.
#[derive(Parser, Debug)]
#[command(name = "chatpack")]
#[command(version, about, long_about = None)]
#[command(after_help = "EXAMPLES:
    chatpack telegram result.json
    chatpack tg chat.json -o optimized.csv
    chatpack wa whatsapp_chat.txt --after 2024-01-01
    chatpack ig messages.json --format jsonl")]
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
}

/// Supported chat sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Source {
    /// Telegram JSON export
    #[value(alias = "tg")]
    Telegram,
    /// WhatsApp TXT export
    #[value(alias = "wa")]
    WhatsApp,
    /// Instagram JSON export
    #[value(alias = "ig")]
    Instagram,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Telegram => write!(f, "Telegram"),
            Source::WhatsApp => write!(f, "WhatsApp"),
            Source::Instagram => write!(f, "Instagram"),
        }
    }
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum OutputFormat {
    /// CSV with semicolon delimiter (default)
    #[default]
    Csv,
    /// JSON array of messages
    Json,
    /// JSON Lines - one JSON object per line (ideal for ML/RAG)
    Jsonl,
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
