# Benchmarks

This document captures two kinds of performance information:

- real-world token-compression measurements from a Telegram export sample
- the current Criterion benchmark suite in `benches/parsing.rs`

Runtime numbers depend heavily on CPU, disk, compiler version, and Criterion settings. Run the local commands below when you need fresh numbers for a release or regression check.

## Token Compression

Sample: Telegram export with 34,478 messages, measured with the OpenAI `cl100k_base` tokenizer.

| Format | Input Tokens | Output Tokens | Reduction | Ratio |
|--------|--------------|---------------|-----------|-------|
| Raw JSON | 11,177,258 | - | baseline | 1x |
| CSV | - | 849,915 | 92.4% | 13.2x |
| JSONL | - | 1,029,130 | 90.8% | 10.9x |
| JSON | - | 1,333,586 | 88.1% | 8.4x |

CSV wins for LLM context because the schema appears once in the header instead of being repeated for every message. JSONL is usually a better fit for RAG ingestion because each line can become one independent document.

## Current Benchmark Suite

The benchmark source is `benches/parsing.rs`.

| Group | Sizes | What it measures |
|-------|-------|------------------|
| `telegram_parsing` | 100, 1K, 10K, 50K | In-memory Telegram JSON parsing |
| `whatsapp_parsing` | 100, 1K, 10K, 50K | In-memory WhatsApp TXT parsing |
| `instagram_parsing` | 100, 1K, 10K, 50K | In-memory Instagram JSON parsing |
| `discord_parsing` | 100, 1K, 10K, 50K | In-memory Discord JSON parsing |
| `telegram_streaming` | 100, 1K, 10K, 50K | Native Telegram streaming parser |
| `telegram_streaming_tricky_strings` | 100, 1K, 10K, 50K | Telegram streaming with braces, quotes, and escaped text |
| `instagram_streaming` | 100, 1K, 10K, 50K | Native Instagram streaming parser |
| `instagram_streaming_tricky_strings` | 100, 1K, 10K, 50K | Instagram streaming with braces, quotes, and escaped text |
| `merge_consecutive` | 100, 1K, 10K, 100K | Consecutive-message merge pass |
| `filter_by_sender` | 100, 1K, 10K, 100K | Sender filtering |
| `filter_by_date` | 100, 1K, 10K, 100K | Date-range filtering |
| `output_csv` | 100, 1K, 10K | CSV serialization |
| `output_json` | 100, 1K, 10K | JSON serialization |
| `output_jsonl` | 100, 1K, 10K | JSONL serialization |
| `full_pipeline` | 1K, 10K, 50K | Telegram parse -> merge -> CSV output |

## Published Criterion Snapshot

The table below is derived from the Criterion `new/estimates.json` artifacts published on the [`gh-pages` benchmark branch](https://github.com/Berektassuly/chatpack/tree/gh-pages/benchmarks). Treat these as public benchmark artifacts for the current published snapshot; absolute timings still depend on CPU, runner, compiler version, and Criterion settings.

Throughput is approximated as benchmark size divided by Criterion's `mean.point_estimate`.

| Benchmark | Size | Mean Time | Approx. Throughput |
|-----------|------|-----------|--------------------|
| `full_pipeline` | 50K | 31.3 ms | ~1.6M msg/s |
| `telegram_parsing` | 50K | 33.3 ms | ~1.5M msg/s |
| `telegram_streaming` | 50K | 43.3 ms | ~1.15M msg/s |
| `telegram_streaming_tricky_strings` | 50K | 50.3 ms | ~994K msg/s |
| `instagram_parsing` | 50K | 15.1 ms | ~3.3M msg/s |
| `discord_parsing` | 50K | 26.3 ms | ~1.9M msg/s |
| `whatsapp_parsing` | 50K | 99.8 ms | ~501K msg/s |
| `filter_by_sender` | 100K | 7.9 ms | ~12.7M msg/s |
| `merge_consecutive` | 100K | 8.9 ms | ~11.2M msg/s |
| `output_csv` | 10K | 0.77 ms | ~13M msg/s |
| `output_jsonl` | 10K | 0.92 ms | ~10.8M msg/s |

## CI Coverage

GitHub Actions runs:

| Workflow | Purpose |
|----------|---------|
| `ci.yml` | Build and test on Linux, macOS, and Windows; run formatting and Clippy |
| `coverage.yml` | Generate coverage with `cargo tarpaulin` and upload to Codecov |
| `bench.yml` | Run Criterion benchmarks and publish benchmark artifacts |

## Run Benchmarks

```bash
# Run the benchmark suite
cargo bench --bench parsing

# Run one group
cargo bench --bench parsing -- telegram_parsing

# Save a baseline
cargo bench --bench parsing -- --save-baseline main

# Compare against a saved baseline
cargo bench --bench parsing -- --baseline main
```

For CI-friendly runs without plots:

```bash
cargo bench --bench parsing -- --noplot --save-baseline current
```

## Interpreting Results

CSV is usually the best format for copying a chat into an LLM context window. JSONL is usually the best format for retrieval pipelines because downstream tools can stream or index each message independently. JSON is easiest when another API expects a single structured array.

Last reviewed: May 28, 2026.
