//! Streaming parser for Discord exports.
//!
//! Supports JSONL format which is naturally streamable.

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek};
use std::path::Path;

use chrono::DateTime;
use serde::Deserialize;

use crate::core::InternalMessage;

use super::{MessageIterator, StreamingConfig, StreamingError, StreamingParser, StreamingResult};

/// Streaming parser for Discord exports.
///
/// Optimized for JSONL format where each line is a separate message.
/// Also handles standard JSON format by falling back to object-by-object parsing.
pub struct DiscordStreamingParser {
    config: StreamingConfig,
}

impl DiscordStreamingParser {
    /// Creates a new streaming parser with default configuration.
    pub fn new() -> Self {
        Self {
            config: StreamingConfig::default(),
        }
    }

    /// Creates a new streaming parser with custom configuration.
    pub fn with_config(config: StreamingConfig) -> Self {
        Self { config }
    }

    /// Detects if the file is JSONL format.
    fn is_jsonl(first_line: &str) -> bool {
        let trimmed = first_line.trim();
        // JSONL: each line is a complete JSON object
        // Regular JSON: starts with { and has nested structure
        trimmed.starts_with('{')
            && !trimmed.contains("\"messages\"")
            && !trimmed.contains("\"guild\"")
    }
}

impl Default for DiscordStreamingParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingParser for DiscordStreamingParser {
    fn name(&self) -> &'static str {
        "Discord (Streaming)"
    }

    fn stream(&self, file_path: &str) -> Result<Box<dyn MessageIterator>, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let file_size = file.metadata()?.len();

        let mut reader = BufReader::with_capacity(self.config.buffer_size, file);

        // Peek first line to detect format
        let mut first_line = String::new();
        reader.read_line(&mut first_line)?;
        reader.seek(std::io::SeekFrom::Start(0))?;

        if Self::is_jsonl(&first_line) {
            let iterator = DiscordJsonlIterator::new(reader, file_size, self.config);
            Ok(Box::new(iterator))
        } else {
            // For regular JSON, use a similar approach to Telegram
            let iterator = DiscordJsonIterator::new(reader, file_size, self.config)?;
            Ok(Box::new(iterator))
        }
    }
}

/// Iterator for JSONL Discord exports.
pub struct DiscordJsonlIterator<R: BufRead> {
    reader: R,
    file_size: u64,
    bytes_read: u64,
    config: StreamingConfig,
    line_buffer: String,
}

impl<R: BufRead> DiscordJsonlIterator<R> {
    fn new(reader: R, file_size: u64, config: StreamingConfig) -> Self {
        Self {
            reader,
            file_size,
            bytes_read: 0,
            config,
            line_buffer: String::with_capacity(4096),
        }
    }

    fn parse_line(line: &str) -> StreamingResult<Option<InternalMessage>> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let msg: DiscordRawMessage = serde_json::from_str(trimmed)?;

        if msg.content.trim().is_empty() {
            return Ok(None);
        }

        let sender = msg.author.nickname.unwrap_or(msg.author.name);

        let timestamp = DateTime::parse_from_rfc3339(&msg.timestamp)
            .ok()
            .map(|dt| dt.to_utc());

        let edited = msg
            .timestamp_edited
            .as_ref()
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.to_utc());

        let id = msg.id.parse::<u64>().ok();

        Ok(Some(InternalMessage::with_metadata(
            sender,
            msg.content,
            timestamp,
            id,
            None,
            edited,
        )))
    }
}

impl<R: BufRead + Send> MessageIterator for DiscordJsonlIterator<R> {
    fn progress(&self) -> Option<f64> {
        if self.file_size == 0 {
            return None;
        }
        Some((self.bytes_read as f64 / self.file_size as f64) * 100.0)
    }

    fn bytes_processed(&self) -> u64 {
        self.bytes_read
    }

    fn total_bytes(&self) -> Option<u64> {
        Some(self.file_size)
    }
}

impl<R: BufRead + Send> Iterator for DiscordJsonlIterator<R> {
    type Item = StreamingResult<InternalMessage>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.line_buffer.clear();
            match self.reader.read_line(&mut self.line_buffer) {
                Ok(0) => return None, // EOF
                Ok(n) => {
                    self.bytes_read += n as u64;
                    match Self::parse_line(&self.line_buffer) {
                        Ok(Some(msg)) => return Some(Ok(msg)),
                        Ok(None) => {}
                        Err(_) if self.config.skip_invalid => {}
                        Err(e) => return Some(Err(e)),
                    }
                }
                Err(e) => return Some(Err(e.into())),
            }
        }
    }
}

/// Iterator for regular JSON Discord exports.
pub struct DiscordJsonIterator<R: BufRead + Seek> {
    reader: R,
    file_size: u64,
    bytes_read: u64,
    config: StreamingConfig,
    buffer: String,
    finished: bool,
    brace_depth: i32,
}

impl<R: BufRead + Seek> DiscordJsonIterator<R> {
    fn new(mut reader: R, file_size: u64, config: StreamingConfig) -> StreamingResult<Self> {
        let mut buffer = String::with_capacity(config.buffer_size);
        let mut total_read = 0u64;

        // Find "messages" array
        loop {
            buffer.clear();
            let bytes = reader.read_line(&mut buffer)?;
            if bytes == 0 {
                return Err(StreamingError::InvalidFormat(
                    "Could not find 'messages' array".into(),
                ));
            }
            total_read += bytes as u64;

            if buffer.contains("\"messages\"") && buffer.contains('[') {
                break;
            }

            if total_read > 10 * 1024 * 1024 {
                return Err(StreamingError::InvalidFormat(
                    "File header too large".into(),
                ));
            }
        }

        Ok(Self {
            reader,
            file_size,
            bytes_read: total_read,
            config,
            buffer: String::with_capacity(config.max_message_size),
            finished: false,
            brace_depth: 0,
        })
    }

    fn read_next_object(&mut self) -> StreamingResult<Option<String>> {
        self.buffer.clear();
        self.brace_depth = 0;
        let mut found_start = false;

        loop {
            let mut line = String::new();
            let bytes = self.reader.read_line(&mut line)?;

            if bytes == 0 {
                self.finished = true;
                return Ok(None);
            }

            self.bytes_read += bytes as u64;

            if !found_start && line.trim().starts_with(']') {
                self.finished = true;
                return Ok(None);
            }

            let trimmed = line.trim();
            if !found_start && (trimmed.is_empty() || trimmed == ",") {
                continue;
            }

            for ch in line.chars() {
                match ch {
                    '{' => {
                        if !found_start {
                            found_start = true;
                        }
                        self.brace_depth += 1;
                    }
                    '}' => self.brace_depth -= 1,
                    _ => {}
                }
            }

            if found_start {
                self.buffer.push_str(&line);

                if self.buffer.len() > self.config.max_message_size {
                    return Err(StreamingError::BufferOverflow {
                        max_size: self.config.max_message_size,
                        actual_size: self.buffer.len(),
                    });
                }

                if self.brace_depth == 0 {
                    return Ok(Some(self.buffer.trim().trim_end_matches(',').to_string()));
                }
            }
        }
    }

    fn parse_message(json_str: &str) -> StreamingResult<Option<InternalMessage>> {
        let msg: DiscordRawMessage = serde_json::from_str(json_str)?;

        let content = msg.content;

        // Skip empty content without attachments
        if content.trim().is_empty() {
            return Ok(None);
        }

        let sender = msg.author.nickname.unwrap_or(msg.author.name);

        let timestamp = DateTime::parse_from_rfc3339(&msg.timestamp)
            .ok()
            .map(|dt| dt.to_utc());

        let edited = msg
            .timestamp_edited
            .as_ref()
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.to_utc());

        let id = msg.id.parse::<u64>().ok();

        let reply_to = msg
            .reference
            .and_then(|r| r.message_id)
            .and_then(|id| id.parse::<u64>().ok());

        Ok(Some(InternalMessage::with_metadata(
            sender, content, timestamp, id, reply_to, edited,
        )))
    }
}

impl<R: BufRead + Seek + Send> MessageIterator for DiscordJsonIterator<R> {
    fn progress(&self) -> Option<f64> {
        if self.file_size == 0 {
            return None;
        }
        Some((self.bytes_read as f64 / self.file_size as f64) * 100.0)
    }

    fn bytes_processed(&self) -> u64 {
        self.bytes_read
    }

    fn total_bytes(&self) -> Option<u64> {
        Some(self.file_size)
    }
}

impl<R: BufRead + Seek + Send> Iterator for DiscordJsonIterator<R> {
    type Item = StreamingResult<InternalMessage>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            match self.read_next_object() {
                Ok(Some(json_str)) => match Self::parse_message(&json_str) {
                    Ok(Some(msg)) => return Some(Ok(msg)),
                    Ok(None) => {}
                    Err(_) if self.config.skip_invalid => {}
                    Err(e) => return Some(Err(e)),
                },
                Ok(None) => return None,
                Err(_) if self.config.skip_invalid => {}
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

/// Raw Discord message for deserialization.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscordRawMessage {
    id: String,
    timestamp: String,
    timestamp_edited: Option<String>,
    content: String,
    author: DiscordAuthor,
    reference: Option<DiscordReference>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscordAuthor {
    name: String,
    nickname: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscordReference {
    message_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_jsonl_detection() {
        // JSONL line
        assert!(DiscordStreamingParser::is_jsonl(
            r#"{"id":"1","timestamp":"2024-01-01T00:00:00Z","content":"hi","author":{"name":"bob"}}"#
        ));

        // Regular JSON start
        assert!(!DiscordStreamingParser::is_jsonl(
            r#"{"guild":{"id":"123"},"messages":["#
        ));
    }

    #[test]
    fn test_parser_name() {
        let parser = DiscordStreamingParser::new();
        assert_eq!(parser.name(), "Discord (Streaming)");
    }
}
