//! # chatpack
//!
//! Compress chat exports from Telegram, WhatsApp, and Instagram
//! into token-efficient CSV for LLMs.
//!
//! ## Usage
//! ```bash
//! chatpack <source> <input_file> [-o output_file]
//! chatpack telegram chat.json -o output.csv
//! chatpack tg chat.json --after 2024-01-01
//! ```

mod cli;
mod core;
mod parsers;

use std::process;

use clap::Parser;

use cli::Args;
use core::{
    apply_filters, merge_consecutive, write_csv, FilterConfig, OutputConfig, ProcessingStats,
};
use parsers::create_parser;

fn main() {
    if let Err(e) = run() {
        eprintln!("❌ Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Create parser for the selected source
    let parser = create_parser(args.source);

    // Print header
    println!("📦 chatpack v{}", env!("CARGO_PKG_VERSION"));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📖 Source:  {}", args.source);
    println!("📂 Input:   {}", args.input);
    println!("💾 Output:  {}", args.output);

    // Build filter configuration
    let mut filter_config = FilterConfig::new();

    if let Some(ref after) = args.after {
        filter_config = filter_config.after_date(after)?;
        println!("📅 After:   {}", after);
    }

    if let Some(ref before) = args.before {
        filter_config = filter_config.before_date(before)?;
        println!("📅 Before:  {}", before);
    }

    if let Some(ref from) = args.from {
        filter_config = filter_config.from_user(from.clone());
        println!("👤 From:    {}", from);
    }

    println!();

    // Step 1: Parse
    println!("⏳ Parsing {}...", parser.name());
    let messages = parser.parse(&args.input)?;
    let original_count = messages.len();
    println!("   Found {} messages", original_count);

    // Step 2: Filter (BEFORE merge)
    let filtered = if filter_config.is_active() {
        println!("🔍 Filtering messages...");
        let filtered = apply_filters(messages, &filter_config);
        println!("   {} messages after filtering", filtered.len());
        filtered
    } else {
        messages
    };
    let filtered_count = filtered.len();

    // Step 3: Merge (unless disabled)
    let final_messages = if args.no_merge {
        println!("⏭️  Skipping merge (--no-merge)");
        filtered
    } else {
        println!("🔀 Merging consecutive messages...");
        let merged = merge_consecutive(filtered);
        println!(
            "   Compressed to {} entries ({:.1}% reduction)",
            merged.len(),
            ProcessingStats::new(filtered_count, merged.len()).compression_ratio()
        );
        merged
    };

    // Step 4: Build output configuration
    let mut output_config = OutputConfig::new();
    if args.timestamps {
        output_config = output_config.with_timestamps();
    }
    if args.ids {
        output_config = output_config.with_ids();
    }
    if args.replies {
        output_config = output_config.with_replies();
    }

    // Step 5: Write output
    println!("💾 Writing CSV...");
    write_csv(&final_messages, &args.output, &output_config)?;

    println!();
    println!("✅ Done! Token-optimized chat saved to {}", args.output);

    // Summary
    if filter_config.is_active() || !args.no_merge {
        println!();
        println!("📊 Summary:");
        println!("   Original:  {} messages", original_count);
        if filter_config.is_active() {
            println!("   Filtered:  {} messages", filtered_count);
        }
        println!("   Final:     {} entries", final_messages.len());
    }

    Ok(())
}
