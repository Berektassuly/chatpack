//! Integration tests for streaming parsers.

use chatpack::cli::Source;
use chatpack::streaming::{
    StreamingConfig, StreamingParser, TelegramStreamingParser, create_streaming_parser,
};
use std::io::Write;
use tempfile::NamedTempFile;

/// Helper to create a test Telegram JSON file
fn create_telegram_test_file(count: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();

    let mut messages = Vec::with_capacity(count);
    for i in 0..count {
        let sender = if i % 2 == 0 { "Alice" } else { "Bob" };
        let timestamp = 1705314600 + (i as i64 * 60);
        messages.push(format!(
            r#"    {{"id": {}, "type": "message", "date_unixtime": "{}", "from": "{}", "text": "Message number {}"}}"#,
            i, timestamp, sender, i
        ));
    }

    let json = format!(
        r#"{{
  "name": "Test Chat",
  "type": "personal_chat",
  "messages": [
{}
  ]
}}"#,
        messages.join(",\n")
    );

    file.write_all(json.as_bytes()).unwrap();
    file
}

#[test]
fn test_streaming_parser_factory() {
    assert!(create_streaming_parser(Source::Telegram).is_some());
    assert!(create_streaming_parser(Source::Discord).is_some());
    assert!(create_streaming_parser(Source::WhatsApp).is_none());
    assert!(create_streaming_parser(Source::Instagram).is_none());
}

#[test]
fn test_telegram_streaming_basic() {
    let file = create_telegram_test_file(100);
    let parser = TelegramStreamingParser::new();

    let messages: Vec<_> = parser
        .stream(file.path().to_str().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    assert_eq!(messages.len(), 100);
    assert_eq!(messages[0].sender, "Alice");
    assert_eq!(messages[1].sender, "Bob");
}

#[test]
fn test_streaming_progress_reporting() {
    let file = create_telegram_test_file(100);
    let parser = TelegramStreamingParser::new();

    let mut iter = parser.stream(file.path().to_str().unwrap()).unwrap();

    // Consume all messages
    let mut last_progress = 0.0;
    while let Some(_) = iter.next() {
        if let Some(progress) = iter.progress() {
            assert!(progress >= last_progress);
            last_progress = progress;
        }
    }

    // Final progress should be close to 100%
    assert!(last_progress > 90.0);
}

#[test]
fn test_streaming_large_file() {
    let file = create_telegram_test_file(10_000);
    let parser = TelegramStreamingParser::new();

    let count = parser
        .stream(file.path().to_str().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .count();

    assert_eq!(count, 10_000);
}

#[test]
fn test_streaming_skips_service_messages() {
    let mut file = NamedTempFile::new().unwrap();
    let json = r#"{
  "name": "Test",
  "messages": [
    {"id": 1, "type": "message", "date_unixtime": "1705314600", "from": "Alice", "text": "Hello"},
    {"id": 2, "type": "service", "action": "pin_message"},
    {"id": 3, "type": "message", "date_unixtime": "1705314660", "from": "Bob", "text": "Hi"}
  ]
}"#;
    file.write_all(json.as_bytes()).unwrap();

    let parser = TelegramStreamingParser::new();
    let messages: Vec<_> = parser
        .stream(file.path().to_str().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    assert_eq!(messages.len(), 2);
}

#[test]
fn test_streaming_handles_empty_content() {
    let mut file = NamedTempFile::new().unwrap();
    let json = r#"{
  "name": "Test",
  "messages": [
    {"id": 1, "type": "message", "date_unixtime": "1705314600", "from": "Alice", "text": "Hello"},
    {"id": 2, "type": "message", "date_unixtime": "1705314620", "from": "Bob", "text": ""},
    {"id": 3, "type": "message", "date_unixtime": "1705314660", "from": "Alice", "text": "Bye"}
  ]
}"#;
    file.write_all(json.as_bytes()).unwrap();

    let parser = TelegramStreamingParser::new();
    let messages: Vec<_> = parser
        .stream(file.path().to_str().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    // Empty messages should be filtered out
    assert_eq!(messages.len(), 2);
}

#[test]
fn test_streaming_with_custom_config() {
    let file = create_telegram_test_file(50);

    let config = StreamingConfig::new()
        .with_buffer_size(32 * 1024)
        .with_skip_invalid(false);

    let parser = TelegramStreamingParser::with_config(config);
    let count = parser
        .stream(file.path().to_str().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .count();

    assert_eq!(count, 50);
}

#[test]
fn test_streaming_bytes_tracking() {
    let file = create_telegram_test_file(100);
    let parser = TelegramStreamingParser::new();

    let mut iter = parser.stream(file.path().to_str().unwrap()).unwrap();

    // Get total bytes
    let total = iter.total_bytes().unwrap();
    assert!(total > 0);

    // Consume all
    while iter.next().is_some() {}

    // Bytes processed should be close to total
    let processed = iter.bytes_processed();
    assert!(processed > 0);
    assert!(processed <= total);
}
