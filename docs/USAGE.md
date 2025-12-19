# 📖 Usage Guide

Complete guide to using chatpack.

## Quick Start

```bash
# Basic usage
chatpack <source> <input_file>

# Sources: tg (telegram), wa (whatsapp), ig (instagram)
chatpack tg result.json
chatpack wa chat.txt  
chatpack ig message_1.json
```

## Output Formats

### CSV (default) — Maximum compression

Best for pasting into ChatGPT/Claude. **13x smaller** than raw JSON.

```bash
chatpack tg chat.json              # outputs optimized_chat.csv
chatpack tg chat.json -f csv       # explicit
```

Output:
```csv
Sender;Content
Alice;Hey! How are you?
Bob;Good thanks!
Alice;Want to grab lunch?
```

### JSON — Structured array

When you need valid JSON structure.

```bash
chatpack tg chat.json -f json
```

Output:
```json
[
  {"sender": "Alice", "content": "Hey! How are you?"},
  {"sender": "Bob", "content": "Good thanks!"},
  {"sender": "Alice", "content": "Want to grab lunch?"}
]
```

### JSONL — One JSON per line

Best for data pipelines. Each line is independent JSON.

```bash
chatpack tg chat.json -f jsonl
```

Output:
```
{"sender":"Alice","content":"Hey! How are you?"}
{"sender":"Bob","content":"Good thanks!"}
{"sender":"Alice","content":"Want to grab lunch?"}
```

**Use JSONL when:**
- Loading into vector databases
- Processing with streaming tools
- Building datasets for fine-tuning

---

## Filters

### By date

```bash
# Messages after January 1, 2024
chatpack tg chat.json --after 2024-01-01

# Messages before June 1, 2024
chatpack tg chat.json --before 2024-06-01

# Messages in Q1 2024
chatpack tg chat.json --after 2024-01-01 --before 2024-04-01
```

Date format: `YYYY-MM-DD`

### By sender

```bash
# Only messages from Alice (case-insensitive)
chatpack tg chat.json --from Alice
chatpack tg chat.json --from "Alice Smith"
```

### Combined filters

```bash
# Alice's messages in January 2024
chatpack tg chat.json --from Alice --after 2024-01-01 --before 2024-02-01
```

---

## Metadata Options

By default, chatpack outputs only `Sender` and `Content`. Add metadata with flags:

```bash
# Include timestamps
chatpack tg chat.json -t
chatpack tg chat.json --timestamps

# Include message IDs
chatpack tg chat.json --ids

# Include reply references (which message this replies to)
chatpack tg chat.json -r
chatpack tg chat.json --replies

# Include edit timestamps
chatpack tg chat.json -e
chatpack tg chat.json --edited

# All metadata
chatpack tg chat.json -t -r -e --ids
```

### Output with all metadata (CSV)

```csv
ID;Timestamp;Sender;Content;ReplyTo;Edited
12345;2024-01-15 10:30:00;Alice;Hey!;;
12346;2024-01-15 10:31:00;Bob;Hi there!;12345;
12347;2024-01-15 10:35:00;Alice;Updated message;;2024-01-15 10:36:00
```

---

## Message Merging

By default, chatpack merges consecutive messages from the same sender:

**Before merge:**
```
Alice: Hey
Alice: How are you?
Alice: Want to chat?
Bob: Sure!
```

**After merge:**
```
Alice: Hey
How are you?
Want to chat?
Bob: Sure!
```

This reduces token count significantly.

### Disable merging

```bash
chatpack tg chat.json --no-merge
```

---

## Output File

### Default naming

```bash
chatpack tg chat.json           # → optimized_chat.csv
chatpack tg chat.json -f json   # → optimized_chat.json
chatpack tg chat.json -f jsonl  # → optimized_chat.jsonl
```

### Custom output

```bash
chatpack tg chat.json -o my_output.csv
chatpack tg chat.json -o /path/to/output.csv
```

---

## Examples

### Feed to ChatGPT/Claude

```bash
chatpack tg chat.json -o context.csv
# Copy content of context.csv
# Paste into ChatGPT with prompt:
# "Based on this conversation, summarize what we discussed about..."
```

### Export specific person's messages

```bash
chatpack wa chat.txt --from "Mom" -o mom_messages.csv
```

### Create dataset with timestamps

```bash
chatpack tg chat.json -f jsonl -t -o dataset.jsonl
```

### Analyze specific time period

```bash
chatpack tg chat.json --after 2024-01-01 --before 2024-02-01 -f json -o january.json
```

### Keep all metadata for archival

```bash
chatpack tg chat.json -t -r -e --ids --no-merge -f jsonl -o archive.jsonl
```

---

## CLI Reference

```
chatpack <SOURCE> <INPUT> [OPTIONS]

Arguments:
  <SOURCE>    Chat source: tg, telegram, wa, whatsapp, ig, instagram
  <INPUT>     Input file path

Options:
  -o, --output <FILE>     Output file [default: optimized_chat.csv]
  -f, --format <FORMAT>   Output format: csv, json, jsonl [default: csv]
  -t, --timestamps        Include timestamps
  -r, --replies           Include reply references
  -e, --edited            Include edit timestamps
      --ids               Include message IDs
      --no-merge          Don't merge consecutive messages
      --after <DATE>      Filter: messages after date (YYYY-MM-DD)
      --before <DATE>     Filter: messages before date (YYYY-MM-DD)
      --from <USER>       Filter: messages from specific sender
  -h, --help              Print help
  -V, --version           Print version
```