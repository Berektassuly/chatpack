//! Tests for output writers (JSON, JSONL, CSV)

use chatpack::core::{InternalMessage, OutputConfig};
use chatpack::core::output::{write_json, write_jsonl, write_csv};
use chrono::{TimeZone, Utc};
use std::fs;
use tempfile::tempdir;

fn sample_messages() -> Vec<InternalMessage> {
    let ts1 = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
    let ts2 = Utc.with_ymd_and_hms(2024, 1, 15, 10, 31, 0).unwrap();
    let ts3 = Utc.with_ymd_and_hms(2024, 1, 15, 10, 32, 0).unwrap();
    let edit_ts = Utc.with_ymd_and_hms(2024, 1, 15, 11, 0, 0).unwrap();
    
    vec![
        InternalMessage::new("Alice", "Hello!")
            .with_id(1)
            .with_timestamp(ts1),
        InternalMessage::new("Bob", "Hi Alice!")
            .with_id(2)
            .with_timestamp(ts2)
            .with_reply_to(1),
        InternalMessage::new("Alice", "How are you?")
            .with_id(3)
            .with_timestamp(ts3)
            .with_edited(edit_ts),
    ]
}

// ============================================================================
// JSON Writer Tests
// ============================================================================

mod json_writer_tests {
    use super::*;

    #[test]
    fn test_write_json_basic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.json");
        let path_str = path.to_str().unwrap();
        
        let messages = sample_messages();
        let config = OutputConfig::new();
        
        write_json(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("Hello!"));
    }

    #[test]
    fn test_write_json_with_all_metadata() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.json");
        let path_str = path.to_str().unwrap();
        
        let messages = sample_messages();
        let config = OutputConfig::all();
        
        write_json(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        
        assert!(content.contains("\"id\""));
        assert!(content.contains("\"timestamp\""));
        assert!(content.contains("\"reply_to\""));
        assert!(content.contains("\"edited\""));
    }

    #[test]
    fn test_write_json_minimal_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.json");
        let path_str = path.to_str().unwrap();
        
        let messages = sample_messages();
        let config = OutputConfig {
            include_timestamps: false,
            include_ids: false,
            include_replies: false,
            include_edited: false,
        };
        
        write_json(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        
        assert!(content.contains("Alice"));
        assert!(content.contains("Hello!"));
    }

    #[test]
    fn test_write_json_empty_messages() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.json");
        let path_str = path.to_str().unwrap();
        
        let messages: Vec<InternalMessage> = vec![];
        let config = OutputConfig::new();
        
        write_json(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("[]") || content.contains("[\n]"));
    }

    #[test]
    fn test_write_json_unicode() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.json");
        let path_str = path.to_str().unwrap();
        
        let messages = vec![
            InternalMessage::new("Алиса", "Привет! 🎉"),
            InternalMessage::new("田中", "こんにちは"),
        ];
        let config = OutputConfig::new();
        
        write_json(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Привет"));
        assert!(content.contains("🎉"));
        assert!(content.contains("こんにちは"));
    }
}

// ============================================================================
// JSONL Writer Tests
// ============================================================================

mod jsonl_writer_tests {
    use super::*;

    #[test]
    fn test_write_jsonl_basic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.jsonl");
        let path_str = path.to_str().unwrap();
        
        let messages = sample_messages();
        let config = OutputConfig::new();
        
        write_jsonl(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        
        assert_eq!(lines.len(), 3);
        
        for line in lines {
            assert!(serde_json::from_str::<serde_json::Value>(line).is_ok());
        }
    }

    #[test]
    fn test_write_jsonl_with_all_metadata() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.jsonl");
        let path_str = path.to_str().unwrap();
        
        let messages = sample_messages();
        let config = OutputConfig::all();
        
        write_jsonl(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"id\""));
        assert!(content.contains("\"reply_to\""));
        assert!(content.contains("\"edited\""));
    }

    #[test]
    fn test_write_jsonl_no_trailing_newline_issues() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.jsonl");
        let path_str = path.to_str().unwrap();
        
        let messages = sample_messages();
        let config = OutputConfig::new();
        
        write_jsonl(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        
        assert!(!content.contains(",\n}"));
        assert!(!content.contains(",\n]"));
    }

    #[test]
    fn test_write_jsonl_empty_messages() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.jsonl");
        let path_str = path.to_str().unwrap();
        
        let messages: Vec<InternalMessage> = vec![];
        let config = OutputConfig::new();
        
        write_jsonl(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.is_empty() || content.trim().is_empty());
    }
}

// ============================================================================
// CSV Writer Tests
// ============================================================================

mod csv_writer_tests {
    use super::*;

    #[test]
    fn test_write_csv_escapes_commas() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.csv");
        let path_str = path.to_str().unwrap();
        
        let messages = vec![
            InternalMessage::new("Alice", "Hello, World!"),
        ];
        let config = OutputConfig::new();
        
        write_csv(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"Hello, World!\"") || content.contains("Hello, World!"));
    }

    #[test]
    fn test_write_csv_escapes_quotes() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.csv");
        let path_str = path.to_str().unwrap();
        
        let messages = vec![
            InternalMessage::new("Alice", "She said \"hello\""),
        ];
        let config = OutputConfig::new();
        
        write_csv(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("hello"));
    }

    #[test]
    fn test_write_csv_empty_messages() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.csv");
        let path_str = path.to_str().unwrap();
        
        let messages: Vec<InternalMessage> = vec![];
        let config = OutputConfig::new();
        
        write_csv(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_write_csv_unicode() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.csv");
        let path_str = path.to_str().unwrap();
        
        let messages = vec![
            InternalMessage::new("Алиса", "Привет! 🎉"),
        ];
        let config = OutputConfig::new();
        
        write_csv(&messages, path_str, &config).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Алиса"));
        assert!(content.contains("Привет"));
    }

    #[test]
    fn test_write_csv_multiline_content() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.csv");
        let path_str = path.to_str().unwrap();
        
        let messages = vec![
            InternalMessage::new("Alice", "Line 1\nLine 2\nLine 3"),
        ];
        let config = OutputConfig::new();
        
        write_csv(&messages, path_str, &config).unwrap();
        
        assert!(path.exists());
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_special_characters_in_content() {
        let dir = tempdir().unwrap();
        
        let messages = vec![
            InternalMessage::new("Alice", "Test <>&\"'"),
            InternalMessage::new("Bob", "Tab:\tNewline:\n"),
            InternalMessage::new("Charlie", "Backslash: \\"),
        ];
        
        let config = OutputConfig::new();
        
        let json_path = dir.path().join("output.json");
        write_json(&messages, json_path.to_str().unwrap(), &config).unwrap();
        
        let jsonl_path = dir.path().join("output.jsonl");
        write_jsonl(&messages, jsonl_path.to_str().unwrap(), &config).unwrap();
        
        let csv_path = dir.path().join("output.csv");
        write_csv(&messages, csv_path.to_str().unwrap(), &config).unwrap();
        
        assert!(json_path.exists());
        assert!(jsonl_path.exists());
        assert!(csv_path.exists());
    }

    #[test]
    fn test_very_long_content() {
        let dir = tempdir().unwrap();
        
        let long_content = "A".repeat(10000);
        let messages = vec![
            InternalMessage::new("Alice", &long_content),
        ];
        
        let config = OutputConfig::new();
        
        let json_path = dir.path().join("output.json");
        write_json(&messages, json_path.to_str().unwrap(), &config).unwrap();
        
        let content = fs::read_to_string(&json_path).unwrap();
        assert!(content.len() > 10000);
    }

    #[test]
    fn test_empty_sender() {
        let dir = tempdir().unwrap();
        
        let messages = vec![
            InternalMessage::new("", "Message with empty sender"),
        ];
        
        let config = OutputConfig::new();
        
        let json_path = dir.path().join("output.json");
        write_json(&messages, json_path.to_str().unwrap(), &config).unwrap();
        
        assert!(json_path.exists());
    }
}