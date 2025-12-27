//! # chatpack CLI
//!
//! Command-line interface for chatpack library.

use std::path::Path;
use std::process;
use std::time::Instant;

use clap::Parser as ClapParser;

use chatpack::cli::Args;
use chatpack::core::{
    FilterConfig, OutputConfig, ProcessingStats, apply_filters, merge_consecutive,
};
use chatpack::format::{OutputFormat, write_to_format};
use chatpack::parser::{create_parser, create_streaming_parser, Platform};
use chatpack::{ChatpackError, Message};

fn main() {
    if let Err(e) = run() {
        eprintln!("❌ Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), ChatpackError> {
    let total_start = Instant::now();
    let args = <Args as ClapParser>::parse();

    // Determine output extension based on format
    let output_path = adjust_output_extension(&args.output, args.format);

    // Print header
    println!("📦 chatpack v{}", env!("CARGO_PKG_VERSION"));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📖 Source:  {}", args.source);
    println!("📂 Input:   {}", args.input);
    println!("💾 Output:  {}", output_path);
    println!("📄 Format:  {}", args.format);
    if args.streaming {
        println!("🌊 Mode:    Streaming");
    }

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
        filter_config = filter_config.with_user(from.clone());
        println!("👤 From:    {}", from);
    }

    println!();

    // Parse messages (streaming or regular)
    let (messages, original_count, parse_time) = if args.streaming {
        parse_streaming(&args)?
    } else {
        parse_regular(&args)?
    };

    println!(
        "   Found {} messages ({:.2}s)",
        original_count,
        parse_time.as_secs_f64()
    );

    // Step 2: Filter (BEFORE merge)
    let filtered = if filter_config.is_active() {
        println!("🔍 Filtering messages...");
        let filter_start = Instant::now();
        let filtered = apply_filters(messages, &filter_config);
        let filter_time = filter_start.elapsed();
        println!(
            "   {} messages after filtering ({:.2}s)",
            filtered.len(),
            filter_time.as_secs_f64()
        );
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
    let lib_format: OutputFormat = args.format.into();
    println!("💾 Writing {}...", lib_format);
    let write_start = Instant::now();
    write_to_format(&final_messages, &output_path, lib_format, &output_config)?;
    let write_time = write_start.elapsed();
    println!("   Written in {:.2}s", write_time.as_secs_f64());

    let total_time = total_start.elapsed();

    println!();
    println!("✅ Done! Output saved to {}", output_path);

    // Summary
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

/// Parse using regular (in-memory) parser
fn parse_regular(
    args: &Args,
) -> Result<(Vec<Message>, usize, std::time::Duration), ChatpackError> {
    let platform: Platform = args.source.into();
    let parser = create_parser(platform);
    println!("⏳ Parsing {}...", parser.name());
    let parse_start = Instant::now();
    let messages = parser.parse(Path::new(&args.input))?;
    let count = messages.len();
    Ok((messages, count, parse_start.elapsed()))
}

/// Parse using streaming parser (memory-efficient)
fn parse_streaming(
    args: &Args,
) -> Result<(Vec<Message>, usize, std::time::Duration), ChatpackError> {
    let platform: Platform = args.source.into();
    let parser = create_streaming_parser(platform);

    println!("⏳ Streaming {}...", parser.name());
    let parse_start = Instant::now();

    let messages: Vec<_> = parser.stream(Path::new(&args.input))?.filter_map(Result::ok).collect();

    let count = messages.len();
    Ok((messages, count, parse_start.elapsed()))
}

/// Adjusts output file extension based on format if using default output.
fn adjust_output_extension(output: &str, format: chatpack::cli::OutputFormat) -> String {
    if output != "optimized_chat.csv" {
        return output.to_string();
    }

    // Convert to library format for extension
    let lib_format: OutputFormat = format.into();
    format!("optimized_chat.{}", lib_format.extension())
}
