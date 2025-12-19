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
use std::time::Instant;

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
    let total_start = Instant::now();
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
    let parse_start = Instant::now();
    let messages = parser.parse(&args.input)?;
    let parse_time = parse_start.elapsed();
    let original_count = messages.len();
    println!("   Found {} messages ({:.2}s)", original_count, parse_time.as_secs_f64());

    // Step 2: Filter (BEFORE merge)
    let filtered = if filter_config.is_active() {
        println!("🔍 Filtering messages...");
        let filter_start = Instant::now();
        let filtered = apply_filters(messages, &filter_config);
        let filter_time = filter_start.elapsed();
        println!("   {} messages after filtering ({:.2}s)", filtered.len(), filter_time.as_secs_f64());
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
        let merge_start = Instant::now();
        let merged = merge_consecutive(filtered);
        let merge_time = merge_start.elapsed();
        println!(
            "   Compressed to {} entries ({:.1}% reduction, {:.2}s)",
            merged.len(),
            ProcessingStats::new(filtered_count, merged.len()).compression_ratio(),
            merge_time.as_secs_f64()
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
    let write_start = Instant::now();
    match args.format {
        OutputFormat::Csv => write_csv(&final_messages, &output_path, &output_config)?,
        OutputFormat::Json => write_json(&final_messages, &output_path, &output_config)?,
        OutputFormat::Jsonl => write_jsonl(&final_messages, &output_path, &output_config)?,
    }
    let write_time = write_start.elapsed();
    println!("   Written in {:.2}s", write_time.as_secs_f64());

    let total_time = total_start.elapsed();

    println!();
    println!("✅ Done! Output saved to {}", output_path);

    // Summary with benchmarks
    println!();
    println!("📊 Summary:");
    println!("   Original:  {} messages", original_count);
    if filter_config.is_active() {
        println!("   Filtered:  {} messages", filtered_count);
    }
    println!("   Final:     {} entries", final_messages.len());
    
    // Performance stats
    println!();
    println!("⚡ Performance:");
    println!("   Total time:  {:.2}s", total_time.as_secs_f64());
    let msgs_per_sec = original_count as f64 / total_time.as_secs_f64();
    println!("   Throughput:  {:.0} messages/sec", msgs_per_sec);

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