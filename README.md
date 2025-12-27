# 📦 chatpack

> Prepare chat data for RAG / LLM ingestion. Compress exports **13x** with CSV format.

🌐 **Try online:** [chatpack.berektassuly.com](https://chatpack.berektassuly.com) — no installation required!

[![Article](https://img.shields.io/badge/Read_Article-How_I_Compressed_11M_Tokens-blueviolet?style=for-the-badge&logo=hashnode)](https://berektassuly.com/chatpack-compress-chat-exports-for-llm-rust)

[![CI](https://github.com/berektassuly/chatpack/actions/workflows/ci.yml/badge.svg)](https://github.com/berektassuly/chatpack/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/berektassuly/chatpack/branch/main/graph/badge.svg)](https://codecov.io/gh/berektassuly/chatpack)
[![Crates.io](https://img.shields.io/crates/v/chatpack.svg)](https://crates.io/crates/chatpack)
[![docs.rs](https://docs.rs/chatpack/badge.svg)](https://docs.rs/chatpack)
[![Downloads](https://img.shields.io/crates/d/chatpack.svg)](https://crates.io/crates/chatpack)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## The Problem

You want to ask Claude/ChatGPT about your conversations, but:
- Raw exports are **80% metadata noise**
- JSON structure wastes tokens on brackets and keys
- Context windows are expensive

## The Solution

```
┌─────────────────┐     ┌──────────┐     ┌─────────────────┐
│ Telegram JSON   │     │          │     │ Clean CSV       │
│ WhatsApp TXT    │ ──▶ │ chatpack │ ──▶ │ Ready for LLM   │
│ Instagram JSON  │     │          │     │ 13x less tokens │
│ Discord Export  │     │          │     │                 │
└─────────────────┘     └──────────┘     └─────────────────┘
```

## Real Numbers

| Format | Input (Telegram JSON) | Output | Savings |
|--------|----------------------|--------|---------|
| **CSV** | 11.2M tokens | 850K tokens | **92% (13x)** 🔥 |
| JSONL | 11.2M tokens | 1.0M tokens | 91% (11x) |
| JSON | 11.2M tokens | 1.3M tokens | 88% (8x) |

> 💡 **Use CSV for maximum token savings.** JSONL is good for RAG pipelines. JSON keeps full structure but wastes tokens.

## Features

- 🚀 **Fast** — 1.6M+ messages/sec (full pipeline)
- 📱 **Multi-platform** — Telegram, WhatsApp, Instagram, Discord
- 🔀 **Smart merge** — Consecutive messages from same sender → one entry
- 🎯 **Filters** — By date, by sender
- 📄 **Formats** — CSV (13x compression), JSON, JSONL (for RAG)
- 📡 **Streaming** — O(1) memory for large files
- ⚡ **Async** — Tokio-based async parsers (optional)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
chatpack = "0.5"
```

### Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `full` | All parsers + all output formats | ✅ |
| `telegram` | Telegram JSON parser | ✅ |
| `whatsapp` | WhatsApp TXT parser | ✅ |
| `instagram` | Instagram JSON parser | ✅ |
| `discord` | Discord JSON/TXT/CSV parser | ✅ |
| `csv-output` | CSV output format | ✅ |
| `json-output` | JSON/JSONL output formats | ✅ |
| `streaming` | Streaming parsers for large files | ✅ |
| `async` | Async/await support with tokio | ❌ |

Enable only what you need:

```toml
# Minimal: just Telegram parser with CSV output
chatpack = { version = "0.5", default-features = false, features = ["telegram", "csv-output"] }

# With async support
chatpack = { version = "0.5", features = ["async"] }
```

## Quick Start

### Basic example

```rust
use chatpack::prelude::*;
use chatpack::parser::{Platform, create_parser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a Telegram export
    let parser = create_parser(Platform::Telegram);
    let messages = parser.parse("telegram_export.json".as_ref())?;

    // Merge consecutive messages from the same sender
    let merged = merge_consecutive(messages);

    // Write to CSV (13x compression)
    write_csv(&merged, "output.csv", &OutputConfig::new())?;

    println!("Processed {} messages", merged.len());
    Ok(())
}
```

### Filter messages

```rust
use chatpack::prelude::*;
use chatpack::parser::{Platform, create_parser};

fn main() -> Result<()> {
    let parser = create_parser(Platform::Telegram);
    let messages = parser.parse("chat.json".as_ref())?;

    // Filter by sender
    let config = FilterConfig::new().with_user("Alice".to_string());
    let alice_only = apply_filters(messages.clone(), &config);

    // Filter by date range
    let config = FilterConfig::new()
        .after_date("2024-01-01")?
        .before_date("2024-06-01")?;
    let filtered = apply_filters(messages, &config);

    Ok(())
}
```

### Output formats

```rust
use chatpack::prelude::*;

fn main() -> Result<()> {
    let messages = vec![
        Message::new("Alice", "Hello!"),
        Message::new("Bob", "Hi there!"),
    ];

    // Minimal output (sender + content only)
    let config = OutputConfig::new();

    // Full metadata (timestamps, IDs, replies, edits)
    let config = OutputConfig::all();

    // Custom selection
    let config = OutputConfig::new()
        .with_timestamps()
        .with_ids();

    // Write to different formats
    write_json(&messages, "output.json", &config)?;
    write_jsonl(&messages, "output.jsonl", &config)?;
    write_csv(&messages, "output.csv", &config)?;

    Ok(())
}
```

### Streaming large files

For files that don't fit in memory, use streaming parsers:

```rust
use chatpack::streaming::{StreamingParser, TelegramStreamingParser, StreamingConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = StreamingConfig::new()
        .with_buffer_size(128 * 1024)  // 128KB buffer
        .with_skip_invalid(true);

    let parser = TelegramStreamingParser::with_config(config);

    // Process messages one at a time - O(1) memory!
    for result in parser.stream("huge_export.json")? {
        match result {
            Ok(msg) => println!("{}: {}", msg.sender, msg.content),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

### Async parsing (requires `async` feature)

```rust
use chatpack::async_parser::{AsyncParser, AsyncTelegramParser};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = AsyncTelegramParser::new();
    let messages = parser.parse("telegram_export.json").await?;

    for msg in messages {
        println!("{}: {}", msg.sender, msg.content);
    }

    Ok(())
}
```

### Processing statistics

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
    println!("Compression: {:.1}%", stats.compression_ratio());
    println!("Messages saved: {}", stats.messages_saved());
}
```

## API Overview

### Core Types

| Type | Description |
|------|-------------|
| `Message` | Universal message representation with optional metadata |
| `OutputConfig` | Controls which fields are included in output |
| `FilterConfig` | Filters by date range and/or sender |
| `ProcessingStats` | Statistics about compression and merging |

### Parsers

| Parser | Platform | Format |
|--------|----------|--------|
| `TelegramParser` | Telegram | JSON export |
| `WhatsAppParser` | WhatsApp | TXT export (auto-detects locale) |
| `InstagramParser` | Instagram | JSON (with Mojibake fix) |
| `DiscordParser` | Discord | JSON/TXT/CSV via DiscordChatExporter |

### Streaming Parsers

| Parser | Memory | Use Case |
|--------|--------|----------|
| `TelegramStreamingParser` | O(1) | Files > 1GB |
| `InstagramStreamingParser` | O(1) | Files > 1GB |
| `DiscordStreamingParser` | O(1) | Files > 1GB |
| `WhatsAppStreamingParser` | O(1) | Files > 1GB |

📚 **Full API documentation:** [docs.rs/chatpack](https://docs.rs/chatpack)

## Supported Platforms

| Source | Format | Features |
|--------|--------|----------|
| Telegram | JSON | IDs, timestamps, replies, edits |
| WhatsApp | TXT | Auto-detect locale (US/EU/RU), multiline |
| Instagram | JSON | Mojibake fix, empty message filter |
| Discord | JSON/TXT/CSV | Via DiscordChatExporter, attachments, stickers |

## Performance

| Metric | Value |
|--------|-------|
| Full pipeline | 1.6-1.7 M messages/sec |
| Parsing (Instagram) | 2.6-2.8 M messages/sec |
| Parsing (Telegram) | 1.4-2.0 M messages/sec |
| Parsing (Discord) | 1.5-1.8 M messages/sec |
| Operations (merge/filter) | 11-14 M messages/sec |
| CSV compression | 13x (92% token reduction) |
| Streaming memory | ~50MB for 10GB file |

> Run `cargo bench --bench parsing` to reproduce benchmarks.

## Documentation

| Guide | Description |
|-------|-------------|
| 📤 [Export Guide](docs/EXPORT_GUIDE.md) | How to export from Telegram, WhatsApp, Instagram, Discord |
| 📊 [Benchmarks](docs/BENCHMARKS.md) | Performance stats and compression metrics |
| 📚 [API Docs](https://docs.rs/chatpack) | Full library documentation |

## License

[MIT](LICENSE) © [Mukhammedali Berektassuly](https://berektassuly.com)
