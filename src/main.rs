//! # chatpack
//!
//! Compress chat exports from Telegram, WhatsApp, and Instagram
//! into token-efficient formats for LLMs.
//!
//! ## Usage
//! ```bash
//! chatpack <source> <input_file> [-o output_file] [-f format]
//! chatpack telegram chat.json -o output.csv
//! chatpack tg chat.json --format jsonl
//! ```

mod cli;
mod core;
mod parsers;

use std::process;

use clap::Parser;

use cli::{Args, OutputFormat};
use core::{
    apply_filters, merge_consecutive, write_csv, write_json, write_jsonl,
    FilterConfig, OutputConfig, ProcessingStats,
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

    // Determine output extension based on format
    let output_path = adjust_output_extension(&args.output, args.format);

    // Print header
    println!("📦 chatpack v{}", env!("CARGO_PKG_VERSION"));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📖 Source:  {}", args.source);
    println!("📂 Input:   {}", args.input);
    println!("💾 Output:  {}", output_path);
    println!("📄 Format:  {}", args.format);

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
    if args.edited {
        output_config = output_config.with_edited();
    }

    // Step 5: Write output in selected format
    println!("💾 Writing {}...", args.format);
    match args.format {
        OutputFormat::Csv => write_csv(&final_messages, &output_path, &output_config)?,
        OutputFormat::Json => write_json(&final_messages, &output_path, &output_config)?,
        OutputFormat::Jsonl => write_jsonl(&final_messages, &output_path, &output_config)?,
    }

    println!();
    println!("✅ Done! Output saved to {}", output_path);

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

/// Adjusts output file extension based on format if using default output.
fn adjust_output_extension(output: &str, format: OutputFormat) -> String {
    // If user specified a custom output, use it as-is
    if output != "optimized_chat.csv" {
        return output.to_string();
    }

    // Otherwise, adjust extension based on format
    match format {
        OutputFormat::Csv => "optimized_chat.csv".to_string(),
        OutputFormat::Json => "optimized_chat.json".to_string(),
        OutputFormat::Jsonl => "optimized_chat.jsonl".to_string(),
    }
}
