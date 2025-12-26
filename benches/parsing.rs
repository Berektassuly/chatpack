//! Benchmarks for chatpack parsing and processing operations.
//!
//! Run with: `cargo bench`
//! Run specific group: `cargo bench --bench parsing -- telegram`

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

use chatpack::core::output::{to_csv, to_json, to_jsonl};
use chatpack::core::{
    FilterConfig, InternalMessage, OutputConfig, apply_filters, merge_consecutive,
};
use chatpack::parsers::{
    ChatParser, DiscordParser, InstagramParser, TelegramParser, WhatsAppParser,
};

use chrono::{Duration, TimeZone, Utc};

// =============================================================================
// Test Data Generators
// =============================================================================

fn generate_telegram_json(count: usize) -> String {
    let mut messages = Vec::with_capacity(count);
    for i in 0..count {
        let sender = if i % 2 == 0 { "Alice" } else { "Bob" };
        let timestamp = 1705314600 + (i as i64 * 60);
        messages.push(format!(
            r#"{{"id": {}, "type": "message", "date_unixtime": "{}", "from": "{}", "text": "Message number {}"}}"#,
            i, timestamp, sender, i
        ));
    }
    format!(
        r#"{{"name": "Test Chat", "type": "personal_chat", "messages": [{}]}}"#,
        messages.join(",\n")
    )
}

fn generate_whatsapp_txt(count: usize) -> String {
    let mut lines = Vec::with_capacity(count);
    for i in 0..count {
        let sender = if i % 2 == 0 { "Alice" } else { "Bob" };
        let hour = i % 24;
        let minute = i % 60;
        lines.push(format!(
            "[15.01.24, {:02}:{:02}:00] {}: Message number {}",
            hour, minute, sender, i
        ));
    }
    lines.join("\n")
}

fn generate_instagram_json(count: usize) -> String {
    let mut messages = Vec::with_capacity(count);
    for i in 0..count {
        let sender = if i % 2 == 0 { "alice_user" } else { "bob_user" };
        let timestamp = 1705314600000i64 + (i as i64 * 60000);
        messages.push(format!(
            r#"{{"sender_name": "{}", "timestamp_ms": {}, "content": "Message number {}"}}"#,
            sender, timestamp, i
        ));
    }
    format!(
        r#"{{"participants": [{{"name": "alice_user"}}, {{"name": "bob_user"}}], "messages": [{}]}}"#,
        messages.join(",\n")
    )
}

fn generate_discord_json(count: usize) -> String {
    let mut messages = Vec::with_capacity(count);
    for i in 0..count {
        let sender = if i % 2 == 0 { "Alice" } else { "Bob" };
        let hour = i % 24;
        let minute = i % 60;
        messages.push(format!(
            r#"{{"id": "{}", "timestamp": "2024-01-15T{:02}:{:02}:00.000+00:00", "content": "Message number {}", "author": {{"name": "{}"}}}}"#,
            i, hour, minute, i, sender
        ));
    }
    format!(
        r#"{{"guild": {{"name": "Test Server"}}, "channel": {{"name": "general"}}, "messages": [{}]}}"#,
        messages.join(",\n")
    )
}

fn generate_messages(count: usize) -> Vec<InternalMessage> {
    let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();
    (0..count)
        .map(|i| {
            let sender = if i % 2 == 0 {
                "Alice".to_string()
            } else {
                "Bob".to_string()
            };
            let ts = base_time + Duration::minutes(i as i64);
            InternalMessage::with_metadata(
                sender,
                format!("Message number {}", i),
                Some(ts),
                Some(i as u64),
                None,
                None,
            )
        })
        .collect()
}

// =============================================================================
// Parsing Benchmarks
// =============================================================================

fn bench_telegram_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("telegram_parsing");
    let parser = TelegramParser::new();

    for size in [100_usize, 1_000, 10_000, 50_000] {
        let json = generate_telegram_json(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &json, |b, json| {
            b.iter(|| {
                let messages = parser.parse_str(black_box(json)).unwrap();
                black_box(messages)
            });
        });
    }
    group.finish();
}

fn bench_whatsapp_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("whatsapp_parsing");
    let parser = WhatsAppParser::new();

    for size in [100_usize, 1_000, 10_000, 50_000] {
        let txt = generate_whatsapp_txt(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &txt, |b, txt| {
            b.iter(|| {
                let messages = parser.parse_str(black_box(txt)).unwrap();
                black_box(messages)
            });
        });
    }
    group.finish();
}

fn bench_instagram_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("instagram_parsing");
    let parser = InstagramParser::new();

    for size in [100_usize, 1_000, 10_000, 50_000] {
        let json = generate_instagram_json(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &json, |b, json| {
            b.iter(|| {
                let messages = parser.parse_str(black_box(json)).unwrap();
                black_box(messages)
            });
        });
    }
    group.finish();
}

fn bench_discord_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("discord_parsing");
    let parser = DiscordParser::new();

    for size in [100_usize, 1_000, 10_000, 50_000] {
        let json = generate_discord_json(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &json, |b, json| {
            b.iter(|| {
                let messages = parser.parse_str(black_box(json)).unwrap();
                black_box(messages)
            });
        });
    }
    group.finish();
}

// =============================================================================
// Processing Benchmarks
// =============================================================================

fn bench_merge_consecutive(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_consecutive");

    for size in [100_usize, 1_000, 10_000, 100_000] {
        let messages = generate_messages(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &messages,
            |b, messages| {
                b.iter(|| {
                    let merged = merge_consecutive(black_box(messages.clone()));
                    black_box(merged)
                });
            },
        );
    }
    group.finish();
}

fn bench_filter_by_sender(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_by_sender");

    for size in [100_usize, 1_000, 10_000, 100_000] {
        let messages = generate_messages(size);
        let config = FilterConfig::new().with_user("Alice".to_string());

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &messages,
            |b, messages| {
                b.iter(|| {
                    let filtered = apply_filters(black_box(messages.clone()), &config);
                    black_box(filtered)
                });
            },
        );
    }
    group.finish();
}

fn bench_filter_by_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_by_date");
    let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();

    for size in [100_usize, 1_000, 10_000, 100_000] {
        let messages: Vec<InternalMessage> = (0..size)
            .map(|i| {
                let ts = base_time - Duration::hours(i as i64);
                InternalMessage::with_metadata(
                    "Alice".to_string(),
                    format!("Message {}", i),
                    Some(ts),
                    Some(i as u64),
                    None,
                    None,
                )
            })
            .collect();

        let config = FilterConfig::new().with_after(base_time - Duration::hours(size as i64 / 2));
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &messages,
            |b, messages| {
                b.iter(|| {
                    let filtered = apply_filters(black_box(messages.clone()), &config);
                    black_box(filtered)
                });
            },
        );
    }
    group.finish();
}

// =============================================================================
// Output Benchmarks
// =============================================================================

fn bench_output_csv(c: &mut Criterion) {
    let mut group = c.benchmark_group("output_csv");
    let config = OutputConfig::default();

    for size in [100_usize, 1_000, 10_000] {
        let messages = generate_messages(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &messages,
            |b, messages| {
                b.iter(|| {
                    let csv = to_csv(black_box(messages), &config).unwrap();
                    black_box(csv)
                });
            },
        );
    }
    group.finish();
}

fn bench_output_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("output_json");
    let config = OutputConfig::default();

    for size in [100_usize, 1_000, 10_000] {
        let messages = generate_messages(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &messages,
            |b, messages| {
                b.iter(|| {
                    let json = to_json(black_box(messages), &config).unwrap();
                    black_box(json)
                });
            },
        );
    }
    group.finish();
}

fn bench_output_jsonl(c: &mut Criterion) {
    let mut group = c.benchmark_group("output_jsonl");
    let config = OutputConfig::default();

    for size in [100_usize, 1_000, 10_000] {
        let messages = generate_messages(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &messages,
            |b, messages| {
                b.iter(|| {
                    let jsonl = to_jsonl(black_box(messages), &config).unwrap();
                    black_box(jsonl)
                });
            },
        );
    }
    group.finish();
}

// =============================================================================
// End-to-End Pipeline Benchmark
// =============================================================================

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");
    let parser = TelegramParser::new();
    let output_config = OutputConfig::default();

    for size in [1_000_usize, 10_000, 50_000] {
        let json = generate_telegram_json(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &json, |b, json| {
            b.iter(|| {
                // Full pipeline: parse -> merge -> output
                let messages = parser.parse_str(black_box(json)).unwrap();
                let merged = merge_consecutive(messages);
                let csv = to_csv(&merged, &output_config).unwrap();
                black_box(csv)
            });
        });
    }
    group.finish();
}

// =============================================================================
// Criterion Configuration
// =============================================================================

criterion_group!(
    benches,
    bench_telegram_parsing,
    bench_whatsapp_parsing,
    bench_instagram_parsing,
    bench_discord_parsing,
    bench_merge_consecutive,
    bench_filter_by_sender,
    bench_filter_by_date,
    bench_output_csv,
    bench_output_json,
    bench_output_jsonl,
    bench_full_pipeline,
);

criterion_main!(benches);
