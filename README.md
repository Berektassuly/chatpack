# 📦 chatpack

> Feed your chat history to LLMs. Compress exports **13x** with CSV format.

[![CI](https://github.com/berektassuly/chatpack/actions/workflows/ci.yml/badge.svg)](https://github.com/berektassuly/chatpack/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/berektassuly/chatpack/branch/main/graph/badge.svg)](https://codecov.io/gh/berektassuly/chatpack)
[![Crates.io](https://img.shields.io/crates/v/chatpack.svg)](https://crates.io/crates/chatpack)
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
│ WhatsApp TXT    │ ──▶ │ chatpack │ ──▶│ Ready for LLM   │
│ Instagram JSON  │     │          │     │ 13x less tokens │
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
- 📱 **Multi-platform** — Telegram, WhatsApp, Instagram
- 🔀 **Smart merge** — Consecutive messages from same sender → one entry
- 🎯 **Filters** — By date, by sender
- 📄 **Formats** — CSV (13x compression), JSON, JSONL (for RAG)

## Installation

### Pre-built binaries (recommended)

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

## Quick Start

```bash
# Telegram
chatpack tg result.json

# WhatsApp  
chatpack wa chat.txt

# Instagram
chatpack ig message_1.json
```

**Output:** `optimized_chat.csv` — ready to paste into ChatGPT/Claude.

## Documentation

| Guide | Description |
|-------|-------------|
| 📤 [Export Guide](docs/EXPORT_GUIDE.md) | How to export from Telegram, WhatsApp, Instagram |
| 📖 [Usage Guide](docs/USAGE.md) | All commands, flags, filters, formats |
| 📊 [Benchmarks](docs/BENCHMARKS.md) | Performance stats and compression metrics |
| 🧪 [Stress Testing](docs/STRESS_TEST.md) | Generate toxic data and run stress tests |

## Quick Reference

```bash
# Output formats
chatpack tg chat.json -f csv      # 13x compression (default)
chatpack tg chat.json -f json     # Structured array
chatpack tg chat.json -f jsonl    # One JSON per line

# Filters  
chatpack tg chat.json --after 2024-01-01
chatpack tg chat.json --from "Alice"

# Metadata
chatpack tg chat.json -t          # Add timestamps
chatpack tg chat.json -t -r -e    # All metadata
```

## Technical Details

| Source | Format | Features |
|--------|--------|----------|
| Telegram | JSON | IDs, timestamps, replies, edits |
| WhatsApp | TXT | Auto-detect locale (US/EU/RU), multiline |
| Instagram | JSON | Mojibake fix, empty message filter |

## Performance

| Metric | Value |
|--------|-------|
| Speed | 20-50K messages/sec |
| CSV compression | 13x (92% token reduction) |
| Tested file size | 500MB+ |

## License

[MIT](LICENSE) © [Mukhammedali Berektassuly](https://berektassuly.com)