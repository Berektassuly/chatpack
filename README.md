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