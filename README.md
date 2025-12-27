# chatpack

**Token-efficient chat export processing for LLM and RAG pipelines.**

[![CI](https://github.com/berektassuly/chatpack/actions/workflows/ci.yml/badge.svg)](https://github.com/berektassuly/chatpack/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/berektassuly/chatpack/branch/main/graph/badge.svg)](https://codecov.io/gh/berektassuly/chatpack)
[![Crates.io](https://img.shields.io/crates/v/chatpack.svg)](https://crates.io/crates/chatpack)
[![docs.rs](https://docs.rs/chatpack/badge.svg)](https://docs.rs/chatpack)
[![Downloads](https://img.shields.io/crates/d/chatpack.svg)](https://crates.io/crates/chatpack)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

[Website](https://chatpack.berektassuly.com) |
[API Docs](https://docs.rs/chatpack) |
[Examples](#examples) |
[Changelog](CHANGELOG.md)

---

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Examples](#examples)
- [API Reference](#api-reference)
- [Supported Platforms](#supported-platforms)
- [Performance](#performance)
- [Feature Flags](#feature-flags)
- [Minimum Supported Rust Version](#minimum-supported-rust-version)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

Chatpack parses chat exports from Telegram, WhatsApp, Instagram, and Discord, converting them into token-efficient formats for LLM analysis and RAG ingestion.

Raw chat exports waste 80%+ of tokens on JSON structure, metadata, and formatting. Chatpack removes this noise, achieving **13x compression** (92% token reduction) with CSV output.

```
┌─────────────────┐                      ┌─────────────────┐
│ Telegram JSON   │                      │ Clean CSV       │
│ WhatsApp TXT    │  ──▶  chatpack  ──▶ │ 13x fewer tokens│
│ Instagram JSON  │                      │ LLM-ready       │
│ Discord Export  │                      │ RAG-optimized   │
└─────────────────┘                      └─────────────────┘
```

### Token Compression Results

| Format | Input | Output | Compression |
|--------|-------|--------|-------------|
| **CSV** | 11.2M tokens | 850K tokens | **13x (92%)** |
| JSONL | 11.2M tokens | 1.0M tokens | 11x (91%) |
| JSON | 11.2M tokens | 1.3M tokens | 8x (88%) |

---

## Installation

Add chatpack to your `Cargo.toml`:

```toml
[dependencies]
chatpack = "0.5"
```

Or install via cargo:

```bash
cargo add chatpack
```

### Minimal Installation

Include only what you need:

```toml
# Telegram parser with CSV output only
chatpack = { version = "0.5", default-features = false, features = ["telegram", "csv-output"] }

# WhatsApp and Instagram with JSON output
chatpack = { version = "0.5", default-features = false, features = ["whatsapp", "instagram", "json-output"] }

# Full async support
chatpack = { version = "0.5", features = ["async"] }
```

See [Feature Flags](#feature-flags) for all options.

---

## Quick Start

```rust
use chatpack::prelude::*;
use chatpack::parser::{Platform, create_parser};

fn main() -> Result<()> {
    // Parse Telegram export
    let parser = create_parser(Platform::Telegram);
    let messages = parser.parse("export.json".as_ref())?;

    // Merge consecutive messages from same sender
    let merged = merge_consecutive(messages);

    // Write as CSV (13x compression)
    write_csv(&merged, "output.csv", &OutputConfig::new())?;

    println!("Processed {} messages", merged.len());
    Ok(())
}
```

---

## Examples

### Basic Parsing

```rust
use chatpack::prelude::*;
use chatpack::parser::{Platform, create_parser};

fn main() -> Result<()> {
    // Create parser for any supported platform
    let parser = create_parser(Platform::Telegram);
    // Also: Platform::WhatsApp, Platform::Instagram, Platform::Discord

    let messages = parser.parse("chat_export.json".as_ref())?;

    for msg in &messages {
        println!("{}: {}", msg.sender, msg.content);
    }

    Ok(())
}
```

### Filtering by Date and Sender

```rust
use chatpack::prelude::*;
use chatpack::parser::{Platform, create_parser};

fn main() -> Result<()> {
    let parser = create_parser(Platform::Telegram);
    let messages = parser.parse("chat.json".as_ref())?;

    // Filter by sender (case-insensitive)
    let config = FilterConfig::new().with_user("Alice".to_string());
    let alice_messages = apply_filters(messages.clone(), &config);

    // Filter by date range
    let config = FilterConfig::new()
        .after_date("2024-01-01")?
        .before_date("2024-06-01")?;
    let q1_messages = apply_filters(messages.clone(), &config);

    // Combine filters
    let config = FilterConfig::new()
        .with_user("Bob".to_string())
        .after_date("2024-03-01")?;
    let bob_march = apply_filters(messages, &config);

    Ok(())
}
```

### Output Formats

```rust
use chatpack::prelude::*;

fn main() -> Result<()> {
    let messages = vec![
        Message::new("Alice", "Hello!"),
        Message::new("Bob", "Hi there!"),
    ];

    // Minimal output (sender + content only)
    let config = OutputConfig::new();

    // Full metadata
    let full_config = OutputConfig::all();

    // Custom fields
    let custom_config = OutputConfig::new()
        .with_timestamps()
        .with_ids();

    // CSV - maximum compression (13x)
    write_csv(&messages, "output.csv", &config)?;

    // JSONL - one object per line, ideal for RAG
    write_jsonl(&messages, "output.jsonl", &config)?;

    // JSON - structured array
    write_json(&messages, "output.json", &config)?;

    Ok(())
}
```

### Streaming Large Files

For files larger than available memory, use streaming parsers with O(1) memory:

```rust
use chatpack::streaming::{StreamingParser, TelegramStreamingParser, StreamingConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = StreamingConfig::new()
        .with_buffer_size(128 * 1024)  // 128KB buffer
        .with_skip_invalid(true);

    let parser = TelegramStreamingParser::with_config(config);

    // Process one message at a time
    for result in parser.stream("10gb_export.json")? {
        match result {
            Ok(msg) => println!("{}: {}", msg.sender, msg.content),
            Err(e) => eprintln!("Skipped invalid: {}", e),
        }
    }

    Ok(())
}
```

### Async Parsing

Requires `async` feature:

```rust
use chatpack::async_parser::{AsyncParser, AsyncTelegramParser};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = AsyncTelegramParser::new();
    let messages = parser.parse("export.json").await?;

    println!("Loaded {} messages", messages.len());
    Ok(())
}
```

### Processing Statistics

```rust
use chatpack::prelude::*;

fn main() {
    let messages = vec![
        Message::new("Alice", "Hi"),
        Message::new("Alice", "How are you?"),
        Message::new("Bob", "Good!"),
    ];

    let original_count = messages.len();
    let merged = merge_consecutive(messages);

    let stats = ProcessingStats::new(original_count, merged.len());
    println!("Original: {} messages", original_count);
    println!("Merged: {} messages", merged.len());
    println!("Reduction: {:.1}%", stats.compression_ratio());
}
```

---

## API Reference

### Core Types

| Type | Description |
|------|-------------|
| [`Message`](https://docs.rs/chatpack/latest/chatpack/struct.Message.html) | Universal message with optional metadata (timestamp, id, reply_to, edited) |
| [`OutputConfig`](https://docs.rs/chatpack/latest/chatpack/core/models/struct.OutputConfig.html) | Controls output fields (timestamps, IDs, replies, edits) |
| [`FilterConfig`](https://docs.rs/chatpack/latest/chatpack/core/filter/struct.FilterConfig.html) | Filters by date range and/or sender |
| [`ProcessingStats`](https://docs.rs/chatpack/latest/chatpack/core/processor/struct.ProcessingStats.html) | Compression and merge statistics |
| [`OutputFormat`](https://docs.rs/chatpack/latest/chatpack/format/enum.OutputFormat.html) | Enum: Csv, Json, Jsonl |

### Parsers

| Type | Platform | Format |
|------|----------|--------|
| [`TelegramParser`](https://docs.rs/chatpack/latest/chatpack/parsers/struct.TelegramParser.html) | Telegram | JSON (Desktop export) |
| [`WhatsAppParser`](https://docs.rs/chatpack/latest/chatpack/parsers/struct.WhatsAppParser.html) | WhatsApp | TXT (auto-detects US/EU/RU locale) |
| [`InstagramParser`](https://docs.rs/chatpack/latest/chatpack/parsers/struct.InstagramParser.html) | Instagram | JSON (with Mojibake encoding fix) |
| [`DiscordParser`](https://docs.rs/chatpack/latest/chatpack/parsers/struct.DiscordParser.html) | Discord | JSON/TXT/CSV (DiscordChatExporter) |

### Streaming Parsers

| Type | Memory | Buffer |
|------|--------|--------|
| `TelegramStreamingParser` | O(1) | Configurable |
| `WhatsAppStreamingParser` | O(1) | Line-based |
| `InstagramStreamingParser` | O(1) | Configurable |
| `DiscordStreamingParser` | O(1) | Configurable |

### Functions

| Function | Description |
|----------|-------------|
| `create_parser(Platform)` | Factory for platform parsers |
| `create_streaming_parser(Platform)` | Factory for streaming parsers |
| `merge_consecutive(Vec<Message>)` | Merge consecutive messages from same sender |
| `apply_filters(Vec<Message>, &FilterConfig)` | Filter by date/sender |
| `write_csv`, `write_json`, `write_jsonl` | Write to file |
| `to_csv`, `to_json`, `to_jsonl` | Convert to string |

---

## Supported Platforms

| Platform | Export Format | Auto-Detection | Special Handling |
|----------|---------------|----------------|------------------|
| **Telegram** | JSON | — | Nested text arrays, service message filtering |
| **WhatsApp** | TXT | 5 date formats (US/EU/RU) | Multiline messages, system message filtering |
| **Instagram** | JSON | — | Mojibake UTF-8 fix, reversed message order |
| **Discord** | JSON/TXT/CSV | Format from extension | Attachments, stickers, nicknames |

### WhatsApp Date Formats

Automatically detected from first 20 lines:

```
[1/15/24, 10:30:45 AM]     # US
[15.01.24, 10:30:45]       # EU (dot, bracketed)
15.01.2024, 10:30 -        # EU (dot, no bracket)
15/01/2024, 10:30 -        # EU (slash)
[15/01/2024, 10:30:45]     # EU (slash, bracketed)
```

---

## Performance

Benchmarked on Apple M1, single-threaded:

| Operation | Throughput | Notes |
|-----------|------------|-------|
| Full pipeline | 1.6-1.7 M msg/sec | Parse → Merge → CSV |
| Instagram parsing | 2.6-2.8 M msg/sec | Fastest parser |
| Telegram parsing | 1.4-2.0 M msg/sec | Complex JSON handling |
| Discord parsing | 1.5-1.8 M msg/sec | Multiple format support |
| WhatsApp parsing | 1.0-1.2 M msg/sec | Regex-based |
| Merge/Filter | 11-14 M msg/sec | In-memory operations |

### Memory Usage

| Mode | Memory | Use Case |
|------|--------|----------|
| Standard | ~3x file size | Files < 500MB |
| Streaming | ~50MB constant | Files > 1GB |

### Compression

| Output | Token Reduction | Best For |
|--------|-----------------|----------|
| CSV | 92% (13x) | Maximum compression, spreadsheet analysis |
| JSONL | 91% (11x) | RAG pipelines, line-by-line processing |
| JSON | 88% (8x) | Structured output, API responses |

Run benchmarks:

```bash
cargo bench --bench parsing
```

---

## Feature Flags

| Feature | Description | Dependencies | Default |
|---------|-------------|--------------|---------|
| `full` | All parsers + outputs + streaming | all below | ✅ |
| `telegram` | Telegram JSON parser | `serde_json` | ✅ |
| `whatsapp` | WhatsApp TXT parser | `regex` | ✅ |
| `instagram` | Instagram JSON parser | `serde_json` | ✅ |
| `discord` | Discord multi-format parser | `serde_json`, `regex`, `csv` | ✅ |
| `csv-output` | CSV output writer | `csv` | ✅ |
| `json-output` | JSON/JSONL output writers | `serde_json` | ✅ |
| `streaming` | O(1) memory streaming parsers | — | ✅ |
| `async` | Tokio-based async parsers | `tokio`, `tokio-stream`, `async-trait` | ❌ |

### Dependency Matrix

```
telegram   →  serde_json
whatsapp   →  regex
instagram  →  serde_json
discord    →  serde_json + regex + csv
csv-output →  csv
json-output → serde_json
streaming  →  (no extra deps)
async      →  tokio + tokio-stream + async-trait
```

---

## Minimum Supported Rust Version

Chatpack requires **Rust 2024 edition** (rustc 1.85+).

The MSRV is tested in CI and will only be bumped in minor or major releases.

---

## Contributing

Contributions are welcome! Please:

1. Check existing [issues](https://github.com/berektassuly/chatpack/issues)
2. Open an issue for discussion before large changes
3. Run `cargo test` and `cargo clippy` before submitting PRs

### Development

```bash
# Run all tests
cargo test

# Run with all features
cargo test --all-features

# Run benchmarks
cargo bench --bench parsing

# Check lints
cargo clippy --all-features
```

---

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

**[Try chatpack online](https://chatpack.berektassuly.com)** — no installation required!