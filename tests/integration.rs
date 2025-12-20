//! Integration tests for parsers with real files

use chatpack::core::{FilterConfig, ProcessingStats};
use chatpack::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::Once;

static INIT: Once = Once::new();

fn fixtures_dir() -> &'static str {
    "tests/fixtures"
}

fn ensure_fixtures() {
    INIT.call_once(|| {
        let dir = fixtures_dir();
        if !Path::new(dir).exists() {
            fs::create_dir_all(dir).unwrap();
        }

        // Telegram: Simple with explicit timestamps and unixtime for robustness
        let telegram_simple = r#"{
  "name": "Test Chat",
  "type": "personal_chat",
  "id": 123456789,
  "messages": [
    {"id": 1, "type": "message", "date": "2024-01-15T10:30:00", "date_unixtime": "1705314600", "from": "Alice", "text": "Hello!"},
    {"id": 2, "type": "message", "date": "2024-01-15T10:31:00", "date_unixtime": "1705314660", "from": "Bob", "text": "Hi Alice!"},
    {"id": 3, "type": "message", "date": "2024-01-15T10:31:30", "date_unixtime": "1705314690", "from": "Alice", "text": "How are you?"},
    {"id": 4, "type": "message", "date": "2024-01-15T10:32:00", "date_unixtime": "1705314720", "from": "Alice", "text": "I'm doing great!"}
  ]
}"#;
        fs::write(format!("{dir}/telegram_simple.json"), telegram_simple).unwrap();

        // Telegram: Complex
        let telegram_complex = r#"{
  "name": "Complex Chat",
  "type": "personal_chat",
  "id": 987654321,
  "messages": [
    {"id": 1, "type": "message", "date": "2024-02-20T15:00:00", "date_unixtime": "1708441200", "from": "Developer", "text": [
      "Check out this link: ",
      {"type": "link", "text": "https://example.com"},
      " - it's great!"
    ]},
    {"id": 2, "type": "message", "date": "2024-02-20T15:01:00", "date_unixtime": "1708441260", "from": "Developer", "text": [
      {"type": "bold", "text": "Important:"},
      " this is ",
      {"type": "italic", "text": "formatted"},
      " text"
    ], "reply_to_message_id": 1},
    {"id": 3, "type": "message", "date": "2024-02-20T15:02:00", "date_unixtime": "1708441320", "from": "Developer", "text": "Edited message", "edited": "2024-02-20T15:05:00", "edited_unixtime": "1708441500"},
    {"id": 4, "type": "service", "date": "2024-02-20T15:03:00", "date_unixtime": "1708441380", "action": "pin_message"},
    {"id": 5, "type": "message", "date": "2024-02-20T15:04:00", "date_unixtime": "1708441440", "from": "Developer", "text": ""}
  ]
}"#;
        fs::write(format!("{dir}/telegram_complex.json"), telegram_complex).unwrap();

        // WhatsApp: iOS Bracketed Format (Reliable detection)
        let whatsapp_us = "[1/15/24, 10:30:00 AM] Alice: Hello everyone!
[1/15/24, 10:31:00 AM] Bob: Hi Alice!
[1/15/24, 10:32:00 AM] Alice: How is everyone doing?
[1/15/24, 10:32:00 AM] Alice: I hope you're all well
[1/15/24, 10:33:00 AM] Charlie: Messages and calls are end-to-end encrypted. No one outside of this chat, not even WhatsApp, can read or listen to them.
[1/15/24, 10:34:00 AM] Bob: I'm doing great!
[1/15/24, 10:35:00 AM] Alice: <Media omitted>
[1/15/24, 10:36:00 AM] Alice: Check out this link https://example.com";
        fs::write(format!("{dir}/whatsapp_us.txt"), whatsapp_us).unwrap();

        // WhatsApp: EU Format
        let whatsapp_eu = "[15.01.24, 10:30:00] Alice: Привет всем!
[15.01.24, 10:31:00] Bob: Привет!
[15.01.24, 10:32:00] Alice: Как дела?
[15.01.24, 10:33:00] Alice: Надеюсь все хорошо
[15.01.24, 10:34:00] Charlie: Сообщения и звонки защищены сквозным шифрованием.
[15.01.24, 10:35:00] Bob: Все отлично!";
        fs::write(format!("{dir}/whatsapp_eu.txt"), whatsapp_eu).unwrap();

        // Instagram: Full structure with magic_words to ensure auto-detection
        let instagram = r#"{
  "participants": [
    {
      "name": "user_one"
    },
    {
      "name": "user_two"
    }
  ],
  "messages": [
    {
      "sender_name": "user_one",
      "timestamp_ms": 1705315800000,
      "content": "Hey! How are you?",
      "is_geoblocked_for_viewer": false
    },
    {
      "sender_name": "user_two",
      "timestamp_ms": 1705315860000,
      "content": "I'm good, thanks!",
      "is_geoblocked_for_viewer": false
    },
    {
      "sender_name": "user_one",
      "timestamp_ms": 1705315920000,
      "content": "Great to hear!",
      "is_geoblocked_for_viewer": false
    },
    {
      "sender_name": "user_one",
      "timestamp_ms": 1705315950000,
      "content": "What are you up to?",
      "is_geoblocked_for_viewer": false
    },
    {
      "sender_name": "user_two",
      "timestamp_ms": 1705316000000,
      "content": "Check this shared link: https://instagram.com/p/xyz",
      "share": {
        "link": "https://instagram.com/p/xyz",
        "share_text": "Check this out"
      },
      "is_geoblocked_for_viewer": false
    },
    {
      "sender_name": "user_one",
      "timestamp_ms": 1705316060000,
      "is_geoblocked_for_viewer": false
    }
  ],
  "title": "Test Instagram Chat",
  "is_still_participant": true,
  "thread_type": "Regular",
  "thread_path": "inbox/test_123",
  "magic_words": [],
  "joinable_mode": {
    "mode": 1,
    "link": ""
  }
}"#;
        fs::write(format!("{dir}/instagram.json"), instagram).unwrap();
    });
}

// ============================================================================
// Telegram Parser Tests
// ============================================================================

mod telegram_tests {
    use super::*;

    #[test]
    fn test_parse_simple_chat() {
        ensure_fixtures();
        let parser = create_parser(Source::Telegram);
        let messages = parser
            .parse(&format!("{}/telegram_simple.json", fixtures_dir()))
            .unwrap();

        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0].sender, "Alice");
        assert_eq!(messages[0].content, "Hello!");
        assert_eq!(messages[1].sender, "Bob");
        assert!(messages[0].timestamp.is_some());
    }

    #[test]
    fn test_parse_complex_chat() {
        ensure_fixtures();
        let parser = create_parser(Source::Telegram);
        let messages = parser
            .parse(&format!("{}/telegram_complex.json", fixtures_dir()))
            .unwrap();

        assert!(messages.len() >= 2);
        assert!(messages[0].content.contains("https://example.com"));

        let has_reply = messages.iter().any(|m| m.reply_to.is_some());
        assert!(has_reply);

        let has_edited = messages.iter().any(|m| m.edited.is_some());
        assert!(has_edited);
    }

    #[test]
    fn test_merge_consecutive() {
        ensure_fixtures();
        let parser = create_parser(Source::Telegram);
        let messages = parser
            .parse(&format!("{}/telegram_simple.json", fixtures_dir()))
            .unwrap();

        let original_count = messages.len();
        let merged = merge_consecutive(messages);

        assert!(merged.len() <= original_count);
    }

    #[test]
    fn test_parser_name() {
        let parser = create_parser(Source::Telegram);
        assert_eq!(parser.name(), "Telegram");
    }
}

// ============================================================================
// WhatsApp Parser Tests
// ============================================================================

mod whatsapp_tests {
    use super::*;

    #[test]
    fn test_parse_us_format() {
        ensure_fixtures();
        let parser = create_parser(Source::WhatsApp);
        let messages = parser
            .parse(&format!("{}/whatsapp_us.txt", fixtures_dir()))
            .unwrap();

        assert!(!messages.is_empty());

        let senders: Vec<&str> = messages.iter().map(|m| m.sender.as_str()).collect();
        assert!(senders.contains(&"Alice"));
        assert!(senders.contains(&"Bob"));
    }

    #[test]
    fn test_parse_eu_format() {
        ensure_fixtures();
        let parser = create_parser(Source::WhatsApp);
        let messages = parser
            .parse(&format!("{}/whatsapp_eu.txt", fixtures_dir()))
            .unwrap();

        assert!(!messages.is_empty());
        let has_cyrillic = messages.iter().any(|m| m.content.contains("Привет"));
        assert!(has_cyrillic);
    }

    #[test]
    fn test_system_messages_filtered() {
        ensure_fixtures();
        let parser = create_parser(Source::WhatsApp);
        let messages = parser
            .parse(&format!("{}/whatsapp_us.txt", fixtures_dir()))
            .unwrap();

        let has_system = messages.iter().any(|m| {
            m.content.contains("end-to-end encrypted") || m.content.contains("сквозным шифрованием")
        });
        assert!(!has_system);
    }

    #[test]
    fn test_media_preserved() {
        ensure_fixtures();
        let parser = create_parser(Source::WhatsApp);
        let messages = parser
            .parse(&format!("{}/whatsapp_us.txt", fixtures_dir()))
            .unwrap();

        let has_media = messages.iter().any(|m| m.content.contains("Media omitted"));
        assert!(has_media);
    }

    #[test]
    fn test_parser_name() {
        let parser = create_parser(Source::WhatsApp);
        assert_eq!(parser.name(), "WhatsApp");
    }

    #[test]
    fn test_consecutive_merge() {
        ensure_fixtures();
        let parser = create_parser(Source::WhatsApp);
        let messages = parser
            .parse(&format!("{}/whatsapp_us.txt", fixtures_dir()))
            .unwrap();

        let original = messages.len();
        let merged = merge_consecutive(messages);

        assert!(merged.len() <= original);
    }
}

// ============================================================================
// Instagram Parser Tests
// ============================================================================

mod instagram_tests {
    use super::*;

    #[test]
    fn test_parse_instagram() {
        ensure_fixtures();
        let parser = create_parser(Source::Instagram);
        let messages = parser
            .parse(&format!("{}/instagram.json", fixtures_dir()))
            .unwrap();

        assert!(!messages.is_empty());

        let senders: Vec<&str> = messages.iter().map(|m| m.sender.as_str()).collect();
        assert!(senders.contains(&"user_one"));
        assert!(senders.contains(&"user_two"));
    }

    #[test]
    fn test_shared_links() {
        ensure_fixtures();
        let parser = create_parser(Source::Instagram);
        let messages = parser
            .parse(&format!("{}/instagram.json", fixtures_dir()))
            .unwrap();

        let has_shared = messages.iter().any(|m| {
            m.content.contains("Shared") || m.content.contains("https://instagram.com/p/xyz")
        });
        assert!(has_shared, "Should contain 'Shared' or the actual link");
    }

    #[test]
    fn test_empty_content_filtered() {
        ensure_fixtures();
        let parser = create_parser(Source::Instagram);
        let messages = parser
            .parse(&format!("{}/instagram.json", fixtures_dir()))
            .unwrap();

        let has_empty = messages.iter().any(|m| m.content.is_empty());
        assert!(!has_empty);
    }

    #[test]
    fn test_parser_name() {
        let parser = create_parser(Source::Instagram);
        assert_eq!(parser.name(), "Instagram");
    }

    #[test]
    fn test_timestamps() {
        ensure_fixtures();
        let parser = create_parser(Source::Instagram);
        let messages = parser
            .parse(&format!("{}/instagram.json", fixtures_dir()))
            .unwrap();

        assert!(messages.iter().all(|m| m.timestamp.is_some()));
    }
}

// ============================================================================
// Filter Tests with Real Data
// ============================================================================

mod filter_integration_tests {
    use super::*;

    #[test]
    fn test_filter_by_sender() {
        ensure_fixtures();
        let parser = create_parser(Source::Telegram);
        let messages = parser
            .parse(&format!("{}/telegram_simple.json", fixtures_dir()))
            .unwrap();

        let config = FilterConfig::new().with_user("Alice".to_string());
        let filtered = apply_filters(messages, &config);

        assert!(filtered.iter().all(|m| m.sender == "Alice"));
    }

    #[test]
    fn test_filter_by_date() {
        ensure_fixtures();
        let parser = create_parser(Source::Telegram);
        let messages = parser
            .parse(&format!("{}/telegram_simple.json", fixtures_dir()))
            .unwrap();

        let config = FilterConfig::new()
            .after_date("2024-01-14")
            .unwrap()
            .before_date("2024-01-16")
            .unwrap();

        let filtered = apply_filters(messages.clone(), &config);
        assert_eq!(filtered.len(), messages.len());
    }

    #[test]
    fn test_filter_excludes_outside_range() {
        ensure_fixtures();
        let parser = create_parser(Source::Telegram);
        let messages = parser
            .parse(&format!("{}/telegram_simple.json", fixtures_dir()))
            .unwrap();

        let config = FilterConfig::new().before_date("2024-01-01").unwrap();

        let filtered = apply_filters(messages, &config);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_combined_filters() {
        ensure_fixtures();
        let parser = create_parser(Source::WhatsApp);
        let messages = parser
            .parse(&format!("{}/whatsapp_us.txt", fixtures_dir()))
            .unwrap();

        let config = FilterConfig::new().with_user("Alice".to_string());
        let filtered = apply_filters(messages, &config);

        assert!(
            filtered
                .iter()
                .all(|m| m.sender.eq_ignore_ascii_case("Alice"))
        );
    }
}

// ============================================================================
// Processing Stats Tests
// ============================================================================

mod stats_tests {
    use super::*;

    #[test]
    fn test_stats_with_real_data() {
        ensure_fixtures();
        let parser = create_parser(Source::Telegram);
        let messages = parser
            .parse(&format!("{}/telegram_simple.json", fixtures_dir()))
            .unwrap();

        let original = messages.len();
        let merged = merge_consecutive(messages);

        let stats = ProcessingStats::new(original, merged.len());

        assert_eq!(stats.original_count, original);
        assert!(stats.compression_ratio() >= 0.0);
        assert!(stats.compression_ratio() <= 100.0);
    }

    #[test]
    fn test_stats_display() {
        let stats = ProcessingStats::new(100, 80);
        let display = format!("{stats}");

        assert!(display.contains("100"));
        assert!(display.contains("80"));
    }

    #[test]
    fn test_stats_with_filtered() {
        let stats = ProcessingStats::new(100, 50).with_filtered(75);

        assert_eq!(stats.filtered_count, Some(75));
        assert_eq!(stats.messages_saved(), 25);
    }
}

// ============================================================================
// parse_auto Tests
// ============================================================================

mod parse_auto_tests {
    use super::*;
    use chatpack::parsers::parse_auto;

    #[test]
    fn test_auto_detect_telegram() {
        ensure_fixtures();
        let result = parse_auto(&format!("{}/telegram_simple.json", fixtures_dir()));
        assert!(result.is_ok());

        let messages = result.unwrap();
        assert!(!messages.is_empty());
    }

    #[test]
    fn test_auto_detect_whatsapp() {
        ensure_fixtures();
        let result = parse_auto(&format!("{}/whatsapp_us.txt", fixtures_dir()));
        assert!(result.is_ok(), "Should auto-detect WhatsApp US format");
    }

    #[test]
    fn test_auto_nonexistent_file() {
        let result = parse_auto("nonexistent_file.json");
        assert!(result.is_err());
    }
}

// ============================================================================
// Output Config Tests
// ============================================================================

mod output_config_tests {
    use super::*;

    #[test]
    fn test_output_config_default() {
        let config = OutputConfig::new();
        assert!(!config.include_timestamps);
        assert!(!config.include_ids);
    }

    #[test]
    fn test_output_config_all() {
        let config = OutputConfig::all();
        assert!(config.include_timestamps);
        assert!(config.include_ids);
        assert!(config.include_replies);
        assert!(config.include_edited);
    }

    #[test]
    fn test_output_config_builder() {
        let config = OutputConfig::new().with_ids().with_replies();

        assert!(config.include_ids);
        assert!(config.include_replies);
    }

    #[test]
    fn test_output_config_has_any() {
        let config = OutputConfig::new().with_ids();
        assert!(config.has_any());

        let empty = OutputConfig {
            include_timestamps: false,
            include_ids: false,
            include_replies: false,
            include_edited: false,
        };
        assert!(!empty.has_any());
    }
}

// ============================================================================
// InternalMessage Tests
// ============================================================================

mod message_tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_message_builder_chain() {
        let ts = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let edit_ts = Utc.with_ymd_and_hms(2024, 1, 15, 11, 0, 0).unwrap();

        let msg = InternalMessage::new("Alice", "Hello")
            .with_id(123)
            .with_timestamp(ts)
            .with_reply_to(122)
            .with_edited(edit_ts);

        assert_eq!(msg.sender, "Alice");
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.id, Some(123));
        assert_eq!(msg.timestamp, Some(ts));
        assert_eq!(msg.reply_to, Some(122));
        assert_eq!(msg.edited, Some(edit_ts));
    }

    #[test]
    fn test_message_has_metadata() {
        let simple = InternalMessage::new("Bob", "Hi");
        assert!(!simple.has_metadata());

        let with_id = InternalMessage::new("Bob", "Hi").with_id(1);
        assert!(with_id.has_metadata());
    }

    #[test]
    fn test_message_is_empty() {
        let empty = InternalMessage::new("Alice", "");
        assert!(empty.is_empty());

        let not_empty = InternalMessage::new("Alice", "Hello");
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_message_clone() {
        let msg = InternalMessage::new("Alice", "Hello").with_id(1);
        let cloned = msg.clone();

        assert_eq!(msg.sender, cloned.sender);
        assert_eq!(msg.content, cloned.content);
        assert_eq!(msg.id, cloned.id);
    }

    #[test]
    fn test_message_debug() {
        let msg = InternalMessage::new("Alice", "Hello");
        let debug = format!("{msg:?}");

        assert!(debug.contains("Alice"));
        assert!(debug.contains("Hello"));
    }

    #[test]
    fn test_message_partial_eq() {
        let msg1 = InternalMessage::new("Alice", "Hello");
        let msg2 = InternalMessage::new("Alice", "Hello");
        let msg3 = InternalMessage::new("Bob", "Hello");

        assert_eq!(msg1, msg2);
        assert_ne!(msg1, msg3);
    }
}

// ============================================================================
// CLI Types Tests
// ============================================================================

mod cli_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_source_from_str_aliases() {
        assert_eq!(Source::from_str("tg").unwrap(), Source::Telegram);
        assert_eq!(Source::from_str("wa").unwrap(), Source::WhatsApp);
        assert_eq!(Source::from_str("ig").unwrap(), Source::Instagram);
        assert_eq!(Source::from_str("telegram").unwrap(), Source::Telegram);
        assert_eq!(Source::from_str("whatsapp").unwrap(), Source::WhatsApp);
        assert_eq!(Source::from_str("instagram").unwrap(), Source::Instagram);
    }

    #[test]
    fn test_source_from_str_case_insensitive() {
        assert_eq!(Source::from_str("TELEGRAM").unwrap(), Source::Telegram);
        assert_eq!(Source::from_str("WhatsApp").unwrap(), Source::WhatsApp);
        assert_eq!(Source::from_str("INSTAGRAM").unwrap(), Source::Instagram);
    }

    #[test]
    fn test_source_from_str_error() {
        assert!(Source::from_str("unknown").is_err());
        assert!(Source::from_str("").is_err());
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(
            OutputFormat::from_str("jsonl").unwrap(),
            OutputFormat::Jsonl
        );
        assert_eq!(OutputFormat::from_str("csv").unwrap(), OutputFormat::Csv);
    }

    #[test]
    fn test_output_format_from_str_error() {
        assert!(OutputFormat::from_str("xml").is_err());
    }

    #[test]
    fn test_source_default_extension() {
        assert_eq!(Source::Telegram.default_extension(), "json");
        assert_eq!(Source::WhatsApp.default_extension(), "txt");
        assert_eq!(Source::Instagram.default_extension(), "json");
    }

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Json.extension(), "json");
        assert_eq!(OutputFormat::Jsonl.extension(), "jsonl");
        assert_eq!(OutputFormat::Csv.extension(), "csv");
    }

    #[test]
    fn test_output_format_mime_type() {
        assert_eq!(OutputFormat::Json.mime_type(), "application/json");
        assert_eq!(OutputFormat::Jsonl.mime_type(), "application/x-ndjson");
        assert_eq!(OutputFormat::Csv.mime_type(), "text/csv");
    }

    #[test]
    fn test_source_all_names() {
        let names = Source::all_names();
        assert!(names.contains(&"telegram"));
        assert!(names.contains(&"whatsapp"));
        assert!(names.contains(&"instagram"));
    }

    #[test]
    fn test_output_format_all_names() {
        let names = OutputFormat::all_names();
        assert!(names.contains(&"json"));
        assert!(names.contains(&"jsonl"));
        assert!(names.contains(&"csv"));
    }

    #[test]
    fn test_source_display() {
        assert_eq!(format!("{}", Source::Telegram), "Telegram");
        assert_eq!(format!("{}", Source::WhatsApp), "WhatsApp");
        assert_eq!(format!("{}", Source::Instagram), "Instagram");
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

mod error_tests {
    use super::*;

    #[test]
    fn test_parse_nonexistent_file() {
        let parser = create_parser(Source::Telegram);
        let result = parser.parse("nonexistent.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json() {
        ensure_fixtures();
        let dir = fixtures_dir();
        fs::write(format!("{dir}/invalid.json"), "not valid json").unwrap();

        let parser = create_parser(Source::Telegram);
        let result = parser.parse(&format!("{dir}/invalid.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_invalid_date() {
        let result = FilterConfig::new().after_date("not-a-date");
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_error_display() {
        let err = FilterConfig::new().after_date("invalid").unwrap_err();
        let display = format!("{err}");
        assert!(display.contains("Invalid date format"));
    }
}

// ============================================================================
// Serde Tests
// ============================================================================

mod serde_tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_message_serialize_deserialize() {
        let ts = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let msg = InternalMessage::new("Alice", "Hello")
            .with_id(1)
            .with_timestamp(ts);

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: InternalMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_source_serde() {
        let source = Source::Telegram;
        let json = serde_json::to_string(&source).unwrap();
        let deserialized: Source = serde_json::from_str(&json).unwrap();

        assert_eq!(source, deserialized);
    }

    #[test]
    fn test_output_format_serde() {
        let format = OutputFormat::Json;
        let json = serde_json::to_string(&format).unwrap();
        let deserialized: OutputFormat = serde_json::from_str(&json).unwrap();

        assert_eq!(format, deserialized);
    }

    #[test]
    fn test_output_config_serde() {
        let config = OutputConfig::all();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OutputConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.include_timestamps, deserialized.include_timestamps);
        assert_eq!(config.include_ids, deserialized.include_ids);
    }
}
