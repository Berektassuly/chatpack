# 📦 chatpack

> Prepare chat data for RAG / LLM ingestion. Compress exports **13x** with CSV format.

[![CI](https://github.com/berektassuly/chatpack/actions/workflows/ci.yml/badge.svg)](https://github.com/berektassuly/chatpack/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/berektassuly/chatpack/branch/main/graph/badge.svg)](https://codecov.io/gh/berektassuly/chatpack)
[![Crates.io](https://img.shields.io/crates/v/chatpack.svg)](https://crates.io/crates/chatpack)
[![docs.rs](https://docs.rs/chatpack/badge.svg)](https://docs.rs/chatpack)
[![Downloads](https://img.shields.io/crates/d/chatpack.svg)](https://crates.io/crates/chatpack)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Platforms:** Windows • macOS • Linux

## The Problem

You want to ask Claude/ChatGPT about your conversations, but:
- Raw exports are **80% metadata noise**
- JSON structure wastes tokens on brackets and keys
- Context windows are expensive

## The Solution

```
┌─────────────────┐     ┌──────────┐     ┌─────────────────┐
│ Telegram JSON   │     │          │     │ Clean CSV       │
│ WhatsApp TXT    │ ──▶│ chatpack │ ──▶ │ Ready for LLM   │
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

## Use Cases

### 💬 Chat with your chat history
```bash
chatpack tg telegram_export.json -o context.txt
# Paste into ChatGPT: "Based on this conversation, what did we decide about...?"
```

### 🔍 Build RAG pipeline
```bash
chatpack tg chat.json -f jsonl -t -o dataset.jsonl
# Each line = one document with timestamp for vector DB
```

### 📊 Analyze conversations
```bash
chatpack wa chat.txt --from "Alice" --after 2024-01-01 -f json
# Filter and export specific messages
```

## Features

- 🚀 **Fast** — 20K+ messages/sec
- 📱 **Multi-platform** — Telegram, WhatsApp, Instagram, Discord
- 🔀 **Smart merge** — Consecutive messages from same sender → one entry
- 🎯 **Filters** — By date, by sender
- 📄 **Formats** — CSV (13x compression), JSON, JSONL (for RAG)
- 📚 **Library** — Use as Rust crate in your projects

## Installation

### Pre-built binaries

| Platform | Download |
|----------|----------|
| Windows | [chatpack-windows-x64.exe](https://github.com/berektassuly/chatpack/releases/latest/download/chatpack-windows-x64.exe) |
| macOS (Intel) | [chatpack-macos-x64](https://github.com/berektassuly/chatpack/releases/latest/download/chatpack-macos-x64) |
| macOS (Apple Silicon) | [chatpack-macos-arm64](https://github.com/berektassuly/chatpack/releases/latest/download/chatpack-macos-arm64) |
| Linux | [chatpack-linux-x64](https://github.com/berektassuly/chatpack/releases/latest/download/chatpack-linux-x64) |

### Via Cargo

```bash
cargo install chatpack
```

### As a library

```toml
[dependencies]
chatpack = "0.2"
```

## Quick Start (CLI)

```bash
# Telegram
chatpack tg result.json

# WhatsApp  
chatpack wa chat.txt

# Instagram
chatpack ig message_1.json

# Discord
chatpack dc chat.json
```

**Output:** `optimized_chat.csv` — ready to paste into ChatGPT/Claude.

## Library Usage

### Basic example

```rust
use chatpack::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a Telegram export
    let parser = create_parser(Source::Telegram);
    let messages = parser.parse("telegram_export.json")?;

    // Merge consecutive messages from the same sender
    let merged = merge_consecutive(messages);

    // Write to JSON
    write_json(&merged, "output.json", &OutputConfig::new())?;

    Ok(())
}
```

### Auto-detect format

```rust
use chatpack::parsers::parse_auto;

// Automatically detects Telegram, WhatsApp, or Instagram
let messages = parse_auto("unknown_chat.json")?;
```

### Filter messages

```rust
use chatpack::prelude::*;

let parser = create_parser(Source::Telegram);
let messages = parser.parse("chat.json")?;

// Filter by sender
let config = FilterConfig::new()
    .with_user("Alice".to_string());
let alice_only = apply_filters(messages.clone(), &config);

// Filter by date range
let config = FilterConfig::new()
    .after_date("2024-01-01")?
    .before_date("2024-06-01")?;
let filtered = apply_filters(messages, &config);
```

### Output formats

```rust
use chatpack::prelude::*;

let messages = vec![
    InternalMessage::new("Alice", "Hello!"),
    InternalMessage::new("Bob", "Hi there!"),
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
```

### Processing statistics

```rust
use chatpack::prelude::*;

let original_count = messages.len();
let merged = merge_consecutive(messages);

let stats = ProcessingStats::new(original_count, merged.len());
println!("Compression: {:.1}%", stats.compression_ratio());
println!("Messages saved: {}", stats.messages_saved());
```

📚 **Full API documentation:** [docs.rs/chatpack](https://docs.rs/chatpack)

## CLI Reference

```bash
# Output formats
chatpack tg chat.json -f csv      # 13x compression (default)
chatpack tg chat.json -f json     # Structured array
chatpack tg chat.json -f jsonl    # One JSON per line (for RAG)

# Filters  
chatpack tg chat.json --after 2024-01-01
chatpack tg chat.json --before 2024-06-01
chatpack tg chat.json --from "Alice"

# Metadata
chatpack tg chat.json -t          # Add timestamps
chatpack tg chat.json -r          # Add reply references
chatpack tg chat.json -e          # Add edit timestamps
chatpack tg chat.json --ids       # Add message IDs
chatpack tg chat.json -t -r -e --ids  # All metadata

# Other options
chatpack tg chat.json --no-merge  # Don't merge consecutive messages
chatpack tg chat.json -o out.csv  # Custom output path
```

## Documentation

| Guide | Description |
|-------|-------------|
| 📤 [Export Guide](docs/EXPORT_GUIDE.md) | How to export from Telegram, WhatsApp, Instagram, Discord |
| 📖 [Usage Guide](docs/USAGE.md) | All commands, flags, filters, formats |
| 📊 [Benchmarks](docs/BENCHMARKS.md) | Performance stats and compression metrics |
| 🧪 [Stress Testing](docs/STRESS_TEST.md) | Generate toxic data and run stress tests |
| 📚 [API Docs](https://docs.rs/chatpack) | Full library documentation |

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
| Speed | 20-50K messages/sec |
| CSV compression | 13x (92% token reduction) |
| Tested file size | 500MB+ |

## License

[MIT](LICENSE) © [Mukhammedali Berektassuly](https://berektassuly.com)