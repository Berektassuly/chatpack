# Stress Testing Guide

## Quick Start

```bash
# Build everything
cargo build --release

# Generate 100K toxic Telegram messages (~50MB)
cargo run --release --bin gen_test -- 100000 heavy_test.json telegram

# Generate 100K toxic WhatsApp messages
cargo run --release --bin gen_test -- 100000 heavy_test.txt whatsapp

# Generate 1M messages (~500MB)  
cargo run --release --bin gen_test -- 1000000 mega_test.json telegram

# Run stress test
./target/release/chatpack tg heavy_test.json -o stress_result.csv
```

## Toxic Data Types Generated

| Type | Description | Purpose |
|------|-------------|---------|
| Giant strings | 100KB+ messages | Memory stress |
| Zalgo text | Combining characters | Unicode handling |
| Emoji spam | 50+ emojis per message | UTF-8 handling |
| Semicolons | `msg;with;semi;colons` | CSV escaping |
| Quotes | `msg "with" quotes` | CSV/JSON escaping |
| Newlines | Multi-line content | Parser robustness |
| Empty | `""`, `"   "` | Edge cases |
| Control chars | `\x00\x01\x02` | Binary safety |
| Mixed Unicode | RU + JP + AR + emoji | Encoding |
| Garbage lines | Invalid format | WhatsApp robustness |

## Expected Benchmark Results

| Messages | File Size | Parse Time | Throughput |
|----------|-----------|------------|------------|
| 10K | ~5 MB | <1s | ~50K msg/s |
| 100K | ~50 MB | ~2s | ~50K msg/s |
| 1M | ~500 MB | ~20s | ~50K msg/s |

## Memory Optimization (Future)

For 1GB+ files, we should implement streaming JSON parsing:

```rust
// Current (loads all into memory):
let export: TelegramExport = serde_json::from_str(&content)?;

// Streaming (process one message at a time):
use serde_json::StreamDeserializer;
use std::io::BufReader;

let file = File::open(path)?;
let reader = BufReader::new(file);

// Skip to "messages" array
// Then stream each message:
let stream = serde_json::Deserializer::from_reader(reader)
    .into_iter::<TelegramMessage>();

for result in stream {
    match result {
        Ok(msg) => process_message(msg),
        Err(e) => eprintln!("Skipped invalid message: {}", e),
    }
}
```

## CSV Escaping Verification

The `csv` crate automatically handles:
- Semicolons inside fields → quoted
- Quotes inside fields → escaped as `""`
- Newlines inside fields → quoted
- UTF-8 → passed through

Test with:
```bash
cargo run --release --bin gen_test -- 1000 escaping_test.json telegram
./target/release/chatpack tg escaping_test.json -o test.csv
# Verify CSV opens correctly in Excel/LibreOffice
```

## WhatsApp Robustness

The parser handles garbage lines gracefully:
1. Lines not matching any format → treated as continuation
2. Continuation without prior message → silently skipped  
3. Empty lines → skipped
4. System messages → filtered

Test:
```bash
cargo run --release --bin gen_test -- 10000 wa_stress.txt whatsapp
./target/release/chatpack wa wa_stress.txt
```
