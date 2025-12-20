//! JSON Lines (JSONL) output writer.
//!
//! JSONL format is ideal for:
//! - Machine learning pipelines
//! - RAG (Retrieval-Augmented Generation)
//! - Streaming processing
//! - Large datasets that don't fit in memory

use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};

use serde::Serialize;

use crate::core::models::{InternalMessage, OutputConfig};

/// Minimal message structure for JSONL output.
/// Only includes fields enabled in `OutputConfig`.
#[derive(Serialize)]
struct JsonlMessage {
    sender: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    edited: Option<String>,
}

impl JsonlMessage {
    fn from_internal(msg: &InternalMessage, config: &OutputConfig) -> Self {
        Self {
            sender: msg.sender.clone(),
            content: msg.content.clone(),
            timestamp: if config.include_timestamps {
                msg.timestamp
                    .map(|ts| ts.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            } else {
                None
            },
            id: if config.include_ids { msg.id } else { None },
            reply_to: if config.include_replies {
                msg.reply_to
            } else {
                None
            },
            edited: if config.include_edited {
                msg.edited
                    .map(|ts| ts.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            } else {
                None
            },
        }
    }
}

/// Writes messages to JSONL (JSON Lines) format.
///
/// Each line is a valid JSON object:
/// ```jsonl
/// {"sender":"Alice","content":"Hello"}
/// {"sender":"Bob","content":"Hi"}
/// ```
///
/// This format is ideal for:
/// - Streaming processing (one record at a time)
/// - ML training data
/// - RAG document ingestion
pub fn write_jsonl(
    messages: &[InternalMessage],
    output_path: &str,
    config: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    for msg in messages {
        let json_msg = JsonlMessage::from_internal(msg, config);
        let line = serde_json::to_string(&json_msg)?;
        writeln!(writer, "{line}")?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_jsonl_basic() {
        let messages = vec![
            InternalMessage::new("Alice", "Hello"),
            InternalMessage::new("Bob", "Hi"),
        ];

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let config = OutputConfig::new();
        write_jsonl(&messages, path, &config).unwrap();

        // Read and verify each line is valid JSON
        let file = std::fs::File::open(path).unwrap();
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

        assert_eq!(lines.len(), 2);

        // Parse each line as JSON to verify validity
        let msg1: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(msg1["sender"], "Alice");
        assert_eq!(msg1["content"], "Hello");

        let msg2: serde_json::Value = serde_json::from_str(&lines[1]).unwrap();
        assert_eq!(msg2["sender"], "Bob");
        assert_eq!(msg2["content"], "Hi");
    }

    #[test]
    fn test_write_jsonl_with_metadata() {
        use chrono::TimeZone;

        let ts = chrono::Utc
            .with_ymd_and_hms(2024, 6, 15, 12, 30, 0)
            .unwrap();
        let edited = chrono::Utc.with_ymd_and_hms(2024, 6, 15, 13, 0, 0).unwrap();

        let msg = InternalMessage::new("Alice", "Hello")
            .timestamp(ts)
            .id(123)
            .edited(edited);

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let config = OutputConfig::new()
            .with_timestamps()
            .with_ids()
            .with_edited();
        write_jsonl(&[msg], path, &config).unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(content.trim()).unwrap();

        assert_eq!(parsed["timestamp"], "2024-06-15T12:30:00Z");
        assert_eq!(parsed["id"], 123);
        assert_eq!(parsed["edited"], "2024-06-15T13:00:00Z");
    }

    #[test]
    fn test_jsonl_no_trailing_comma() {
        let messages = vec![InternalMessage::new("Alice", "Hello")];

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        write_jsonl(&messages, path, &OutputConfig::new()).unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        // Should not contain array brackets or commas between objects
        assert!(!content.contains('['));
        assert!(!content.contains(']'));
    }
}
