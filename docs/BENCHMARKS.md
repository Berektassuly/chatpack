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

## Processing Speed

### By platform (real data)

| Platform | Messages | File Size | Time | Throughput |
|----------|----------|-----------|------|------------|
| Telegram | 34,478 | ~10 MB | 0.21s | 162K msg/s |
| Discord TXT | 1,232 | 646 KB | 0.01s | 85K msg/s |

### By output format (34K Telegram messages)

| Format | Time | Speed |
|--------|------|-------|
| CSV | 0.21s | **162K msg/s** |
| JSON | 0.18s | **186K msg/s** |
| JSONL | 0.26s | **131K msg/s** |

### By operation (34K messages)

| Operation | Time |
|-----------|------|
| Parse JSON | 0.15-0.22s |
| Merge | 0.00-0.01s |
| Write output | 0.03-0.04s |
| **Total** | **0.18-0.26s** |

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

## Your Own Benchmarks

Run the included stress test:

```bash
# Generate 100K toxic messages
cargo run --release --bin gen_test -- 100000 heavy_test.json telegram

# Process and see stats
./target/release/chatpack tg heavy_test.json
```

Output includes:
```
⚡ Performance:
   Total time:  3.57s
   Throughput:  28011 messages/sec
```

---

*Last updated: December 2025*
*Contributions welcome! Add your benchmarks via PR.*