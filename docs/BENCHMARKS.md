# 📊 Benchmarks & Statistics

Real-world compression metrics and performance data.

## Token Compression

Tested with Telegram export (34,478 messages), measured with OpenAI tokenizer (cl100k_base).

### By Output Format

| Format | Input Tokens | Output Tokens | Compression | Ratio |
|--------|-------------|---------------|-------------|-------|
| Raw JSON | 11,177,258 | — | baseline | 1x |
| **CSV** | — | 849,915 | **92.4%** | **13.2x** 🔥 |
| JSONL | — | 1,029,130 | 90.8% | 10.9x |
| JSON | — | 1,333,586 | 88.1% | 8.4x |

### Why CSV wins

| Factor | CSV | JSON | JSONL |
|--------|-----|------|-------|
| Brackets `{}[]` | ❌ None | ✅ `[{},{}]` | ✅ `{}` per line |
| Key names | ❌ Header only | ✅ Every message | ✅ Every message |
| Quotes | Minimal | Every string | Every string |
| Delimiter | `;` (1 char) | `,` + spaces | newline |

### Estimated cost savings (GPT-4o)

| Context size | Raw JSON | CSV | Savings |
|--------------|----------|-----|---------|
| 10K messages | $0.56 | $0.04 | $0.52 |
| 100K messages | $5.60 | $0.43 | $5.17 |
| 1M messages | $56.00 | $4.30 | $51.70 |

*Based on $5/1M input tokens*

---

## Criterion Benchmark Results

All benchmarks run with `cargo bench --bench parsing` on release build.

### Parsing Performance

| Platform | 100 msgs | 1K msgs | 10K msgs | 50K msgs |
|----------|----------|---------|----------|----------|
| **Instagram** | 37.9 µs | 377 µs | 3.6 ms | 18.0 ms |
| **Telegram** | 49.2 µs | 487 µs | 7.2 ms | 32.0 ms |
| **Discord** | 60.1 µs | 567 µs | 5.6 ms | 32.1 ms |
| **WhatsApp** | 2.6 ms | 4.4 ms | 22.3 ms | 102.2 ms |

### Throughput (messages/second)

| Platform | Throughput |
|----------|------------|
| **Instagram** | 2.6-2.8 M/s |
| **Telegram** | 1.4-2.0 M/s |
| **Discord** | 1.5-1.8 M/s |
| **WhatsApp** | 38-489 K/s |

> WhatsApp uses regex-based text parsing, hence slower than JSON parsers.

### Operations Performance

| Operation | 100 msgs | 1K msgs | 10K msgs | 100K msgs |
|-----------|----------|---------|----------|-----------|
| **Merge consecutive** | 8.9 µs | 84 µs | 690 µs | 7.0 ms |
| **Filter by sender** | 8.2 µs | 75 µs | 744 µs | 7.4 ms |
| **Filter by date** | 8.1 µs | 77 µs | 764 µs | 7.4 ms |

| Operation | Throughput |
|-----------|------------|
| Merge consecutive | 11-14 M/s |
| Filter by sender | 12-13 M/s |
| Filter by date | 12-13 M/s |

### Output Format Performance

| Format | 100 msgs | 1K msgs | 10K msgs |
|--------|----------|---------|----------|
| **CSV** | 8.1 µs | 77 µs | 874 µs |
| **JSONL** | 10.4 µs | 102 µs | 998 µs |
| **JSON** | 16.6 µs | 158 µs | 1.5 ms |

| Format | Throughput |
|--------|------------|
| CSV | 11-12 M/s |
| JSONL | 9-10 M/s |
| JSON | 6.0-6.6 M/s |

### Full Pipeline (Parse → Merge → Filter → Output)

| Messages | Time | Throughput |
|----------|------|------------|
| 1K | 602 µs | 1.66 M/s |
| 10K | 5.9 ms | 1.70 M/s |
| 50K | 29.8 ms | 1.68 M/s |

---

## Message Merging

Consecutive messages from the same sender are merged into one entry.

### Real-world results

| Chat | Original | After Merge | Reduction |
|------|----------|-------------|-----------|
| Telegram group | 34,478 | 26,169 | 24% |
| WhatsApp personal | 1,751 | 809 | 54% |
| Instagram DM | 3,292 | 1,660 | 50% |
| Discord channel | 1,232 | 583 | 53% |

### When merging helps most

- **Group chats with few participants** — people send many short messages
- **Personal chats** — rapid back-and-forth
- **Voice message transcripts** — often split into fragments
- **Discord announcements** — admins often send consecutive updates

---

## Memory Usage

chatpack loads entire file into memory. Expected usage:

| File Size | RAM Usage |
|-----------|-----------|
| 10 MB | ~30 MB |
| 100 MB | ~300 MB |
| 1 GB | ~3 GB |

*For files >1GB, consider splitting by date ranges.*

---

## Format Comparison

### For LLM context (ChatGPT/Claude)

| Criterion | CSV | JSON | JSONL |
|-----------|-----|------|-------|
| Token efficiency | ⭐⭐⭐ | ⭐ | ⭐ |
| Readability | ⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| Copy-paste friendly | ⭐⭐⭐ | ⭐⭐ | ⭐ |
| **Recommendation** | ✅ Best | OK | Not ideal |

### For RAG/Vector DB

| Criterion | CSV | JSON | JSONL |
|-----------|-----|------|-------|
| One doc per line | ❌ | ❌ | ⭐⭐⭐ |
| Streaming parse | ⭐ | ❌ | ⭐⭐⭐ |
| Schema flexibility | ⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **Recommendation** | Not ideal | OK | ✅ Best |

### For archival/analysis

| Criterion | CSV | JSON | JSONL |
|-----------|-----|------|-------|
| Full metadata | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| Excel/Sheets | ⭐⭐⭐ | ❌ | ❌ |
| jq/scripting | ⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **Recommendation** | For spreadsheets | ✅ Best | Also good |

---

## Stress Test Results

Toxic data generator with:
- 100KB+ messages
- Zalgo text (combining characters)
- 50+ emojis per message
- Special characters: `;`, `"`, `\n`, `\t`
- Unicode: Cyrillic, Japanese, Arabic
- Empty/whitespace messages

### Results

| Test | Status |
|------|--------|
| No crashes | ✅ Pass |
| CSV escaping correct | ✅ Pass |
| Unicode preserved | ✅ Pass |
| Empty filtered | ✅ Pass |
| Throughput | 17-24K msg/s |

---

## Run Your Own Benchmarks

```bash
# Run all benchmarks
cargo bench --bench parsing

# Run specific benchmark
cargo bench --bench parsing -- telegram_parsing

# Save baseline for comparison
cargo bench --bench parsing -- --save-baseline main

# Compare against baseline
cargo bench --bench parsing -- --baseline main
```

---

*Last updated: December 2025*
*Benchmarks run on Linux with Criterion.rs*
