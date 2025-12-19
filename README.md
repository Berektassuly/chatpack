# 📦 chatpack

> Compress chat exports from Telegram, WhatsApp, and Instagram into token-efficient formats for LLMs.

[![Crates.io](https://img.shields.io/crates/v/chatpack.svg)](https://crates.io/crates/chatpack)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Why?

LLM context windows are expensive. A typical Telegram export is 80% metadata noise. **chatpack** strips it down to what matters: `sender` and `content`.

```
Before: 34,478 tokens (raw JSON)
After:  26,169 tokens (chatpack CSV)
        ━━━━━━━━━━━━━━━━━━━━━━━━
        24% reduction ✨
```

## Features

- 🚀 **Fast** — 20K+ messages/sec
- 📱 **Multi-platform** — Telegram, WhatsApp, Instagram
- 🔀 **Smart merge** — Consecutive messages from same sender → one entry
- 🎯 **Filters** — By date, by sender
- 📄 **Formats** — CSV, JSON, JSONL

## Installation

```bash
cargo install chatpack
```

Or build from source:
```bash
git clone https://github.com/berektassuly/chatpack
cd chatpack
cargo build --release
```

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

## Supported Export Formats

### Telegram
Export via: **Settings → Advanced → Export Telegram Data**
- ✅ JSON format
- ✅ Message IDs, timestamps, replies, edits
- ✅ Nested text objects (bold, links, etc.)

### WhatsApp
Export via: **Chat → ⋮ → More → Export chat → Without media**
- ✅ TXT format (all locales)
- ✅ Auto-detects date format (US, EU, RU)
- ✅ Multiline messages
- ✅ Filters system messages

### Instagram
Export via: **Settings → Your activity → Download your information**
- ✅ JSON format
- ✅ Fixes Mojibake encoding (Cyrillic, etc.)
- ✅ Filters empty shares/reactions

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

## Use Cases

### Feed chat to LLM
```bash
chatpack tg chat.json -o context.csv
# Then paste context.csv into ChatGPT/Claude
```

### Build RAG dataset
```bash
chatpack tg chat.json -f jsonl -t -o dataset.jsonl
# Each line is a document with timestamp
```

### Analyze specific period
```bash
chatpack tg chat.json --after 2024-01-01 --before 2024-02-01 -f json
```

### Export single person's messages
```bash
chatpack wa chat.txt --from "Mom" -o mom_messages.csv
```

## License

MIT © [Mukhammedali Berektassuly](https://berektassuly.com)