# 📦 chatpack

> Feed your chat history to LLMs. Compress Telegram, WhatsApp, Instagram exports into token-efficient formats.

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
│ Telegram JSON   │     │          │     │ Clean CSV/JSONL │
│ WhatsApp TXT    │ ──▶ │ chatpack │ ──▶ │ Ready for LLM   │
│ Instagram JSON  │     │          │     │ 24% fewer tokens│
└─────────────────┘     └──────────┘     └─────────────────┘
```

## Real Numbers

| Metric | Before | After | Savings |
|--------|--------|-------|---------|
| Telegram (34K msgs) | 34,478 tokens | 26,169 tokens | **24%** |
| WhatsApp (1.7K msgs) | 12,340 tokens | 8,920 tokens | **28%** |
| File size | 4.2 MB | 1.1 MB | **74%** |

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
- 📄 **Formats** — CSV (token-efficient), JSON, JSONL (RAG-ready)

## Installation

### Pre-built binaries (recommended)

Download the latest release for your platform:

| Platform | Download |
|----------|----------|
| Windows | [chatpack-windows-x64.exe](https://github.com/berektassuly/chatpack/releases/latest/download/chatpack-windows-x64.exe) |
| macOS (Intel) | [chatpack-macos-x64](https://github.com/berektassuly/chatpack/releases/latest/download/chatpack-macos-x64) |
| macOS (Apple Silicon) | [chatpack-macos-arm64](https://github.com/berektassuly/chatpack/releases/latest/download/chatpack-macos-arm64) |
| Linux | [chatpack-linux-x64](https://github.com/berektassuly/chatpack/releases/latest/download/chatpack-linux-x64) |

**macOS/Linux:** After downloading, make it executable:
```bash
chmod +x chatpack-*
./chatpack-macos-arm64 tg chat.json
```

### Via Cargo

```bash
cargo install chatpack
```

### Build from source

```bash
git clone https://github.com/berektassuly/chatpack
cd chatpack
cargo build --release
```

## How to Export Your Chats

### 📱 Telegram (Desktop)

1. Open **Telegram Desktop** (not mobile!)
2. Go to **Settings** → **Advanced** → **Export Telegram data**
3. Select the chat you want to export
4. **Important settings:**
   - ✅ Format: **JSON**
   - ❌ Uncheck: Photos, Videos, Voice messages (saves space)
   - ✅ Check: Text messages
5. Click **Export** → Wait → Get `result.json`

```bash
chatpack tg result.json
```

### 💬 WhatsApp (Mobile)

**iPhone:**
1. Open chat → Tap contact name at top
2. Scroll down → **Export Chat**
3. Choose **Without Media**
4. Send to yourself (email, AirDrop, Files)

**Android:**
1. Open chat → Tap **⋮** (three dots)
2. **More** → **Export chat**
3. Choose **Without media**
4. Save/send the `.txt` file

```bash
chatpack wa "WhatsApp Chat with Mom.txt"
```

### 📸 Instagram (Web)

1. Go to [instagram.com](https://instagram.com) → Log in
2. **Settings** → **Your activity** → **Download your information**
3. **Request a download** → Select **Some of your information**
4. ✅ Check only **Messages**
5. **Format:** JSON, **Date range:** All time
6. Click **Submit request** → Wait for email (can take hours/days)
7. Download ZIP → Extract → Find `messages/inbox/username/message_1.json`

```bash
chatpack ig message_1.json
```

> ⚠️ Instagram exports have broken encoding (Mojibake). chatpack fixes it automatically!

---

## Usage

### Basic

```bash
# Telegram JSON export
chatpack tg result.json

# WhatsApp TXT export  
chatpack wa chat.txt

# Instagram JSON export
chatpack ig message_1.json
```

### Output Formats

```bash
# CSV (default) — best for token efficiency
chatpack tg chat.json -f csv

# JSON — structured array
chatpack tg chat.json -f json

# JSONL — one JSON per line, streaming-friendly
chatpack tg chat.json -f jsonl
```

### Filters

```bash
# Messages after date
chatpack tg chat.json --after 2024-01-01

# Messages before date
chatpack tg chat.json --before 2024-06-01

# Messages from specific user
chatpack tg chat.json --from "Alice"

# Combine filters
chatpack tg chat.json --after 2024-01-01 --from "Bob"
```

### Metadata Options

```bash
# Include timestamps
chatpack tg chat.json -t

# Include message IDs
chatpack tg chat.json --ids

# Include reply references
chatpack tg chat.json -r

# Include edit timestamps
chatpack tg chat.json -e

# All metadata
chatpack tg chat.json -t -r -e --ids
```

### Other Options

```bash
# Custom output file
chatpack tg chat.json -o my_output.csv

# Disable message merging
chatpack tg chat.json --no-merge
```

## Output Examples

### CSV (default)
```csv
Sender;Content
Alice;Hey! How are you?
Bob;Good thanks! Just finished the project.
Alice;Nice! Let's celebrate 🎉
```

### JSON
```json
[
  {"sender": "Alice", "content": "Hey! How are you?"},
  {"sender": "Bob", "content": "Good thanks! Just finished the project."},
  {"sender": "Alice", "content": "Nice! Let's celebrate 🎉"}
]
```

### JSONL
```jsonl
{"sender":"Alice","content":"Hey! How are you?"}
{"sender":"Bob","content":"Good thanks! Just finished the project."}
{"sender":"Alice","content":"Nice! Let's celebrate 🎉"}
```

## Technical Details

| Source | Format | Features |
|--------|--------|----------|
| Telegram | JSON | IDs, timestamps, replies, edits, nested text |
| WhatsApp | TXT | Auto-detect locale (US/EU/RU), multiline, system filter |
| Instagram | JSON | Mojibake fix, empty message filter |

## Performance

Tested on 500MB files with toxic data (Zalgo, emoji spam, 100KB strings):

| Metric | Value |
|--------|-------|
| Throughput | 17-24K msg/s |
| Memory | ~2x file size |
| Max tested | 516 MB, 100K messages |

## CLI Reference

```
chatpack <SOURCE> <INPUT> [OPTIONS]

Sources:
  tg, telegram    Telegram JSON export
  wa, whatsapp    WhatsApp TXT export
  ig, instagram   Instagram JSON export

Options:
  -o, --output <FILE>     Output file [default: optimized_chat.csv]
  -f, --format <FORMAT>   Output format: csv, json, jsonl [default: csv]
  -t, --timestamps        Include timestamps
  -r, --replies           Include reply references
  -e, --edited            Include edit timestamps
      --ids               Include message IDs
      --no-merge          Don't merge consecutive messages
      --after <DATE>      Filter: after date (YYYY-MM-DD)
      --before <DATE>     Filter: before date (YYYY-MM-DD)
      --from <USER>       Filter: from specific sender
  -h, --help              Print help
  -V, --version           Print version
```

## License

MIT © [Mukhammedali Berektassuly](https://berektassuly.com)