//! Command-line interface definition using clap.

use clap::{Parser, ValueEnum};

/// Compress chat exports from Telegram, WhatsApp, and Instagram
/// into token-efficient CSV for LLMs.
#[derive(Parser, Debug)]
#[command(name = "chatpack")]
#[command(version, about, long_about = None)]
#[command(after_help = "EXAMPLES:
    chatpack telegram result.json
    chatpack tg chat.json -o optimized.csv
    chatpack wa whatsapp_chat.txt --after 2024-01-01
    chatpack ig messages.json --before 2024-12-31")]
pub struct Args {
    /// Chat source type
    #[arg(value_enum)]
    pub source: Source,

    /// Path to input file
    pub input: String,

    /// Path to output file
    #[arg(short, long, default_value = "optimized_chat.csv")]
    pub output: String,

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
