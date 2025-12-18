//! # chatpack
//!
//! Compress chat exports from Telegram, WhatsApp, and Instagram
//! into token-efficient CSV for LLMs.
//!
//! ## Usage
//! ```bash
//! chatpack <source> <input_file> [output_file]
//! chatpack telegram chat.json output.csv
//! chatpack tg chat.json  # defaults to optimized_chat.csv
//! ```

mod core;
mod parsers;

use std::env;
use std::error::Error;
use std::process;

use core::process_and_write;
use parsers::{create_parser, ChatSource};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_OUTPUT: &str = "optimized_chat.csv";

fn main() {
    if let Err(e) = run() {
        eprintln!("❌ Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage(&args[0]);
        process::exit(1);
    }

    // Parse arguments
    let source_arg = &args[1];
    let input_path = &args[2];
    let output_path = args.get(3).map(|s| s.as_str()).unwrap_or(DEFAULT_OUTPUT);

    // Handle help/version flags
    if source_arg == "--help" || source_arg == "-h" {
        print_usage(&args[0]);
        return Ok(());
    }
    if source_arg == "--version" || source_arg == "-v" {
        println!("chatpack v{}", VERSION);
        return Ok(());
    }

    // Get chat source
    let source = ChatSource::from_arg(source_arg).ok_or_else(|| {
        format!(
            "Unknown source: '{}'. Available: {:?}",
            source_arg,
            ChatSource::available()
        )
    })?;

    // Create appropriate parser
    let parser = create_parser(source);

    // Run the pipeline
    println!("📦 chatpack v{}", VERSION);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📖 Source:  {}", parser.name());
    println!("📂 Input:   {}", input_path);
    println!("💾 Output:  {}", output_path);
    println!();

    // Parse
    println!("⏳ Parsing {}...", parser.name());
    let messages = parser.parse(input_path)?;
    println!("   Found {} messages", messages.len());

    // Process and write
    println!("🔀 Merging consecutive messages...");
    let stats = process_and_write(messages, output_path)?;

    println!(
        "   Compressed to {} entries ({:.1}% reduction)",
        stats.merged_count,
        stats.compression_ratio()
    );

    println!();
    println!("✅ Done! Token-optimized chat saved to {}", output_path);

    Ok(())
}

fn print_usage(program: &str) {
    eprintln!(
        r#"
chatpack v{} — Compress chat exports into token-efficient CSV for LLMs

USAGE:
    {} <source> <input_file> [output_file]

SOURCES:
    telegram, tg    Telegram JSON export (result.json)
    whatsapp, wa    WhatsApp TXT export (_chat.txt)
    instagram, ig   Instagram JSON export (messages.json)

EXAMPLES:
    {} telegram result.json
    {} tg chat.json optimized.csv
    {} wa whatsapp_chat.txt
    {} ig messages.json insta_chat.csv

OPTIONS:
    -h, --help      Show this help message
    -v, --version   Show version

OUTPUT:
    Default output file: {}
    Format: CSV with semicolon (;) delimiter
    Columns: Sender, Content
"#,
        VERSION, program, program, program, program, program, DEFAULT_OUTPUT
    );
}
