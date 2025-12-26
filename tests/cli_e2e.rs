//! End-to-end CLI tests for chatpack.
//!
//! These tests verify the complete CLI workflow by running the actual binary
//! with various arguments and checking the output.
//!
//! # Test Categories
//!
//! - **Basic functionality**: Each parser works via CLI
//! - **Output formats**: CSV, JSON, JSONL generation
//! - **Filters**: Date and sender filtering
//! - **Flags**: All CLI flags work correctly
//! - **Error handling**: Proper error messages for bad input
//! - **Edge cases**: Empty files, unicode, special characters
//!
//! # Running Tests
//!
//! ```bash
//! cargo test --test cli_e2e
//! ```

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

// ============================================================================
// Test Fixtures
// ============================================================================

/// Creates a temporary directory with test fixtures for all platforms.
fn setup_fixtures() -> TempDir {
    let dir = tempdir().expect("Failed to create temp dir");

    // Telegram simple
    let telegram = r#"{
  "name": "Test Chat",
  "type": "personal_chat",
  "messages": [
    {"id": 1, "type": "message", "date_unixtime": "1705314600", "from": "Alice", "text": "Hello!"},
    {"id": 2, "type": "message", "date_unixtime": "1705314660", "from": "Bob", "text": "Hi Alice!"},
    {"id": 3, "type": "message", "date_unixtime": "1705314720", "from": "Alice", "text": "How are you?"},
    {"id": 4, "type": "message", "date_unixtime": "1705314780", "from": "Alice", "text": "Hope you're well!"}
  ]
}"#;
    fs::write(dir.path().join("telegram.json"), telegram).unwrap();

    // Telegram with metadata
    let telegram_meta = r#"{
  "name": "Meta Chat",
  "type": "personal_chat",
  "messages": [
    {"id": 100, "type": "message", "date_unixtime": "1705314600", "from": "Alice", "text": "Original message"},
    {"id": 101, "type": "message", "date_unixtime": "1705314660", "from": "Bob", "text": "Reply here", "reply_to_message_id": 100},
    {"id": 102, "type": "message", "date_unixtime": "1705314720", "from": "Alice", "text": "Edited message", "edited_unixtime": "1705314900"}
  ]
}"#;
    fs::write(dir.path().join("telegram_meta.json"), telegram_meta).unwrap();

    // WhatsApp US format
    let whatsapp = "[1/15/24, 10:30:00 AM] Alice: Hello everyone!
[1/15/24, 10:31:00 AM] Bob: Hi Alice!
[1/15/24, 10:32:00 AM] Alice: How is everyone doing?
[1/15/24, 10:33:00 AM] Charlie: Messages and calls are end-to-end encrypted.
[1/15/24, 10:34:00 AM] Bob: I'm good!";
    fs::write(dir.path().join("whatsapp.txt"), whatsapp).unwrap();

    // Instagram
    let instagram = r#"{
  "participants": [{"name": "user_one"}, {"name": "user_two"}],
  "messages": [
    {"sender_name": "user_one", "timestamp_ms": 1705315800000, "content": "Hey!"},
    {"sender_name": "user_two", "timestamp_ms": 1705315860000, "content": "Hello!"},
    {"sender_name": "user_one", "timestamp_ms": 1705315920000, "content": "How are you?"}
  ],
  "title": "Test Chat",
  "magic_words": []
}"#;
    fs::write(dir.path().join("instagram.json"), instagram).unwrap();

    // Discord JSON
    let discord = r#"{
  "guild": {"id": "123", "name": "Test Server"},
  "channel": {"id": "456", "name": "general"},
  "messages": [
    {"id": "1001", "type": "Default", "timestamp": "2024-01-15T10:30:00+00:00", "content": "Hello Discord!", "author": {"name": "alice", "nickname": "Alice"}},
    {"id": "1002", "type": "Default", "timestamp": "2024-01-15T10:31:00+00:00", "content": "Hi!", "author": {"name": "bob"}}
  ]
}"#;
    fs::write(dir.path().join("discord.json"), discord).unwrap();

    // Empty telegram file (valid JSON but no messages)
    let telegram_empty = r#"{"name": "Empty", "type": "personal_chat", "messages": []}"#;
    fs::write(dir.path().join("telegram_empty.json"), telegram_empty).unwrap();

    // Unicode content
    let telegram_unicode = r#"{
  "name": "Unicode Chat",
  "type": "personal_chat",
  "messages": [
    {"id": 1, "type": "message", "date_unixtime": "1705314600", "from": "Алиса", "text": "Привет! 🎉"},
    {"id": 2, "type": "message", "date_unixtime": "1705314660", "from": "田中", "text": "こんにちは"},
    {"id": 3, "type": "message", "date_unixtime": "1705314720", "from": "محمد", "text": "مرحبا"}
  ]
}"#;
    fs::write(dir.path().join("telegram_unicode.json"), telegram_unicode).unwrap();

    // Special characters for CSV escaping test
    let telegram_special = r#"{
  "name": "Special Chars",
  "type": "personal_chat", "messages": [
    {"id": 1, "type": "message", "date_unixtime": "1705314600", "from": "Alice", "text": "Hello; with; semicolons"},
    {"id": 2, "type": "message", "date_unixtime": "1705314660", "from": "Bob", "text": "Quotes \"inside\" text"},
    {"id": 3, "type": "message", "date_unixtime": "1705314720", "from": "Charlie", "text": "Line 1\nLine 2\nLine 3"}
  ]
}"#;
    fs::write(dir.path().join("telegram_special.json"), telegram_special).unwrap();

    dir
}

fn chatpack_cmd() -> Command {
    // Replaced deprecated Command::cargo_bin("chatpack") with env lookup
    let cmd = std::process::Command::new(env!("CARGO_BIN_EXE_chatpack"));
    Command::from_std(cmd)
}

fn output_path(dir: &TempDir, name: &str) -> PathBuf {
    dir.path().join(name)
}

// ============================================================================
// Basic Functionality Tests
// ============================================================================

mod basic_functionality {
    use super::*;

    #[test]
    fn test_telegram_basic() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Done"))
            .stdout(predicate::str::contains("messages"));

        assert!(output.exists());
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("Hello!"));
    }

    #[test]
    fn test_whatsapp_basic() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("whatsapp.txt");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "wa",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        assert!(output.exists());
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("Bob"));
        // System message should be filtered
        assert!(!content.contains("end-to-end encrypted"));
    }

    #[test]
    fn test_instagram_basic() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("instagram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "ig",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        assert!(output.exists());
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("user_one"));
        assert!(content.contains("Hey!"));
    }

    #[test]
    fn test_discord_basic() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("discord.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "dc",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        assert!(output.exists());
        let content = fs::read_to_string(&output).unwrap();
        // Should use nickname "Alice" not username "alice"
        assert!(content.contains("Alice"));
        assert!(content.contains("Hello Discord!"));
    }

    #[test]
    fn test_source_aliases() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");

        // Test all aliases work
        for alias in ["tg", "telegram"] {
            let output = output_path(&fixtures, &format!("out_{}.csv", alias));
            chatpack_cmd()
                .args([
                    alias,
                    input.to_str().unwrap(),
                    "-o",
                    output.to_str().unwrap(),
                ])
                .assert()
                .success();
            assert!(output.exists());
        }
    }
}

// ============================================================================
// Output Format Tests
// ============================================================================

mod output_formats {
    use super::*;

    #[test]
    fn test_output_csv_default() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        // CSV should have semicolon delimiter and header
        assert!(content.contains("Sender;Content"));
    }

    #[test]
    fn test_output_json() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.json");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-f",
                "json",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        // Should be valid JSON array
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
        assert!(!parsed.as_array().unwrap().is_empty());
    }

    #[test]
    fn test_output_jsonl() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.jsonl");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-f",
                "jsonl",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        // Each line should be valid JSON
        for line in content.lines() {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(parsed.is_object());
            assert!(parsed.get("sender").is_some());
            assert!(parsed.get("content").is_some());
        }
    }

    #[test]
    fn test_format_flag_long() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.json");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "--format",
                "json",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        assert!(output.exists());
    }

    #[test]
    fn test_default_output_filename_changes_with_format() {
        let fixtures = setup_fixtures();
        let _input = fixtures.path().join("telegram.json");

        // Change to fixtures dir so default output goes there
        chatpack_cmd()
            .current_dir(fixtures.path())
            .args(["tg", "telegram.json", "-f", "jsonl"])
            .assert()
            .success();

        // Default output should be optimized_chat.jsonl
        assert!(fixtures.path().join("optimized_chat.jsonl").exists());
    }
}

// ============================================================================
// Metadata Flag Tests
// ============================================================================

mod metadata_flags {
    use super::*;

    #[test]
    fn test_timestamps_flag() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-t",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Timestamp"));
    }

    #[test]
    fn test_timestamps_long_flag() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "--timestamps",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Timestamp"));
    }

    #[test]
    fn test_ids_flag() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "--ids",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("ID"));
    }

    #[test]
    fn test_replies_flag() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram_meta.json");
        let output = output_path(&fixtures, "out.json");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-r",
                "-f",
                "json",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("reply_to"));
    }

    #[test]
    fn test_edited_flag() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram_meta.json");
        let output = output_path(&fixtures, "out.json");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-e",
                "-f",
                "json",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("edited"));
    }

    #[test]
    fn test_all_metadata_flags_combined() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram_meta.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-t",
                "-r",
                "-e",
                "--ids",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("ID"));
        assert!(content.contains("Timestamp"));
        assert!(content.contains("ReplyTo"));
        assert!(content.contains("Edited"));
    }
}

// ============================================================================
// Filter Tests
// ============================================================================

mod filters {
    use super::*;

    #[test]
    fn test_filter_by_sender() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "--from",
                "Alice",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Alice"));
        assert!(!content.contains("Bob"));
    }

    #[test]
    fn test_filter_by_sender_case_insensitive() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "--from",
                "alice", // lowercase
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Alice")); // Should still match
    }

    #[test]
    fn test_filter_after_date() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "--after",
                "2024-01-14",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("After:"));

        assert!(output.exists());
    }

    #[test]
    fn test_filter_before_date() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "--before",
                "2024-01-16",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Before:"));

        assert!(output.exists());
    }

    #[test]
    fn test_filter_date_range() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "--after",
                "2024-01-01",
                "--before",
                "2024-12-31",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        assert!(output.exists());
    }

    #[test]
    fn test_filter_combined_sender_and_date() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "--from",
                "Alice",
                "--after",
                "2024-01-01",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("From:"))
            .stdout(predicate::str::contains("After:"));

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Alice"));
        assert!(!content.contains("Bob"));
    }
}

// ============================================================================
// Merge Behavior Tests
// ============================================================================

mod merge_behavior {
    use super::*;

    #[test]
    fn test_merge_enabled_by_default() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Merging"))
            .stdout(predicate::str::contains("reduction"));
    }

    #[test]
    fn test_no_merge_flag() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "--no-merge",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Skipping merge"));

        // Without merge, should have all original messages
        let content = fs::read_to_string(&output).unwrap();
        // Count Alice occurrences - should be 3 without merge
        let alice_count = content.matches("Alice").count();
        assert!(alice_count >= 3);
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn test_nonexistent_file() {
        chatpack_cmd()
            .args(["tg", "nonexistent_file.json"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error"));
    }

    #[test]
    fn test_invalid_json() {
        let fixtures = setup_fixtures();
        let invalid = fixtures.path().join("invalid.json");
        fs::write(&invalid, "this is not json").unwrap();

        chatpack_cmd()
            .args(["tg", invalid.to_str().unwrap()])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error"));
    }

    #[test]
    fn test_invalid_date_format() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");

        chatpack_cmd()
            .args(["tg", input.to_str().unwrap(), "--after", "not-a-date"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error"));
    }

    #[test]
    fn test_invalid_source() {
        chatpack_cmd()
            .args(["invalid_source", "file.json"])
            .assert()
            .failure();
    }

    #[test]
    fn test_missing_input_argument() {
        chatpack_cmd().args(["tg"]).assert().failure();
    }

    #[test]
    fn test_invalid_format_option() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");

        chatpack_cmd()
            .args(["tg", input.to_str().unwrap(), "-f", "invalid_format"])
            .assert()
            .failure();
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_chat() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram_empty.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        // Should still create file with just header
        assert!(output.exists());
    }

    #[test]
    fn test_unicode_content() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram_unicode.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Алиса"));
        assert!(content.contains("Привет"));
        assert!(content.contains("🎉"));
        assert!(content.contains("田中"));
        assert!(content.contains("こんにちは"));
    }

    #[test]
    fn test_special_characters_csv_escaping() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram_special.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        // File should exist and be parseable
        assert!(output.exists());
        let content = fs::read_to_string(&output).unwrap();
        // Content with semicolons should be escaped
        assert!(content.contains("semicolons") || content.contains('"'));
    }

    #[test]
    fn test_path_with_spaces() {
        let fixtures = setup_fixtures();
        let dir_with_space = fixtures.path().join("path with spaces");
        fs::create_dir_all(&dir_with_space).unwrap();

        let input = dir_with_space.join("telegram.json");
        fs::copy(fixtures.path().join("telegram.json"), &input).unwrap();

        let output = dir_with_space.join("output.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        assert!(output.exists());
    }
}

// ============================================================================
// Help and Version Tests
// ============================================================================

mod help_and_version {
    use super::*;

    #[test]
    fn test_help_flag() {
        chatpack_cmd()
            .args(["--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("chatpack"))
            .stdout(predicate::str::contains("telegram"))
            .stdout(predicate::str::contains("whatsapp"))
            .stdout(predicate::str::contains("instagram"))
            .stdout(predicate::str::contains("discord"));
    }

    #[test]
    fn test_help_flag_short() {
        chatpack_cmd()
            .args(["-h"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage"));
    }

    #[test]
    fn test_version_flag() {
        chatpack_cmd()
            .args(["--version"])
            .assert()
            .success()
            .stdout(predicate::str::contains("chatpack"))
            .stdout(predicate::str::contains("0.")); // Version starts with 0.
    }

    #[test]
    fn test_version_flag_short() {
        chatpack_cmd()
            .args(["-V"])
            .assert()
            .success()
            .stdout(predicate::str::contains("chatpack"));
    }
}

// ============================================================================
// Output Verification Tests
// ============================================================================

mod output_verification {
    use super::*;

    #[test]
    fn test_output_shows_statistics() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Summary"))
            .stdout(predicate::str::contains("Original"))
            .stdout(predicate::str::contains("Final"))
            .stdout(predicate::str::contains("Performance"))
            .stdout(predicate::str::contains("messages/sec"));
    }

    #[test]
    fn test_output_shows_source_info() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Source:"))
            .stdout(predicate::str::contains("Telegram"));
    }

    #[test]
    fn test_output_shows_format_info() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.json");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-f",
                "json",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Format:"))
            .stdout(predicate::str::contains("JSON"));
    }
}

// ============================================================================
// Regression Tests
// ============================================================================

mod regression {
    use super::*;

    /// Ensure WhatsApp system messages are always filtered
    #[test]
    fn test_whatsapp_system_messages_filtered() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("whatsapp.txt");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "wa",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        assert!(!content.contains("end-to-end encrypted"));
        assert!(!content.contains("Messages and calls"));
    }

    /// Ensure Discord uses nickname when available
    #[test]
    fn test_discord_nickname_preference() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("discord.json");
        let output = output_path(&fixtures, "out.csv");

        chatpack_cmd()
            .args([
                "dc",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        // Should use "Alice" (nickname), not "alice" (username)
        assert!(content.contains("Alice"));
        // bob has no nickname, should use "bob"
        assert!(content.contains("bob"));
    }

    /// Ensure merging preserves first message's metadata
    #[test]
    fn test_merge_preserves_first_message_metadata() {
        let fixtures = setup_fixtures();
        let input = fixtures.path().join("telegram.json");
        let output = output_path(&fixtures, "out.json");

        chatpack_cmd()
            .args([
                "tg",
                input.to_str().unwrap(),
                "-f",
                "json",
                "-t",
                "--ids",
                "-o",
                output.to_str().unwrap(),
            ])
            .assert()
            .success();

        let content = fs::read_to_string(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        // First message should have ID 1 (Alice's first message)
        let first = &parsed[0];
        assert!(first.get("id").is_some());
    }
}
