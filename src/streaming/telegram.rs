//! Streaming parser for Telegram JSON exports.
//!
//! Telegram exports are structured as:
//! ```json
//! {
//!   "name": "Chat Name",
//!   "type": "personal_chat",
//!   "messages": [
//!     {"id": 1, "type": "message", ...},
//!     {"id": 2, "type": "message", ...}
//!   ]
//! }
//! ```
//!
//! This parser streams the messages array without loading the entire file.

use std::fs::File;
use std::io::{BufRead, BufReader, Seek};
use std::path::Path;

use crate::Message;
use crate::error::ChatpackError;
use crate::parsing::telegram::{TelegramRawMessage, parse_telegram_message};

use super::{MessageIterator, StreamingConfig, StreamingError, StreamingParser, StreamingResult};

/// Streaming parser for Telegram JSON exports.
///
/// This parser reads the file sequentially, parsing one message at a time.
/// Memory usage is O(1) relative to file size.
///
/// # Example
///
/// ```rust,no_run
/// use chatpack::streaming::TelegramStreamingParser;
/// use chatpack::streaming::StreamingParser;
///
/// let parser = TelegramStreamingParser::new();
///
/// for result in parser.stream("large_telegram_export.json").unwrap() {
///     if let Ok(msg) = result {
///         println!("{}: {}", msg.sender, msg.content);
///     }
/// }
/// ```
pub struct TelegramStreamingParser {
    config: StreamingConfig,
}

impl TelegramStreamingParser {
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
}

impl Default for TelegramStreamingParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingParser for TelegramStreamingParser {
    fn name(&self) -> &'static str {
        "Telegram (Streaming)"
    }

    fn stream(&self, file_path: &str) -> Result<Box<dyn MessageIterator>, ChatpackError> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let file_size = file.metadata()?.len();

        let reader = BufReader::with_capacity(self.config.buffer_size, file);
        let iterator = TelegramMessageIterator::new(reader, file_size, self.config)?;

        Ok(Box::new(iterator))
    }

    fn recommended_buffer_size(&self) -> usize {
        self.config.buffer_size
    }
}

/// Iterator over Telegram messages.
pub struct TelegramMessageIterator<R: BufRead + Seek> {
    reader: R,
    file_size: u64,
    bytes_read: u64,
    config: StreamingConfig,
    buffer: String,
    finished: bool,
    brace_depth: i32,
}

impl<R: BufRead + Seek> TelegramMessageIterator<R> {
    /// Creates a new iterator, seeking to the messages array.
    fn new(mut reader: R, file_size: u64, config: StreamingConfig) -> StreamingResult<Self> {
        // Find the start of "messages" array
        let mut buffer = String::with_capacity(config.buffer_size);
        let mut total_read = 0u64;

        loop {
            buffer.clear();
            let bytes = reader.read_line(&mut buffer)?;
            if bytes == 0 {
                return Err(StreamingError::InvalidFormat(
                    "Could not find 'messages' array in file".into(),
                ));
            }
            total_read += bytes as u64;

            if buffer.contains("\"messages\"") && buffer.contains('[') {
                break;
            }

            // Safety limit: if we've read 10MB and haven't found messages, give up
            if total_read > 10 * 1024 * 1024 {
                return Err(StreamingError::InvalidFormat(
                    "File header too large or 'messages' array not found".into(),
                ));
            }
        }

        Ok(Self {
            reader,
            file_size,
            bytes_read: total_read,
            buffer: String::with_capacity(config.max_message_size),
            finished: false,
            brace_depth: 0,
            config,
        })
    }

    /// Reads the next JSON object from the messages array.
    fn read_next_object(&mut self) -> StreamingResult<Option<String>> {
        self.buffer.clear();
        self.brace_depth = 0;
        let mut found_start = false;

        loop {
            let mut line = String::new();
            let bytes = self.reader.read_line(&mut line)?;

            if bytes == 0 {
                self.finished = true;
                if found_start {
                    return Err(StreamingError::UnexpectedEof);
                }
                return Ok(None);
            }

            self.bytes_read += bytes as u64;

            // Check for end of messages array
            if !found_start && line.trim().starts_with(']') {
                self.finished = true;
                return Ok(None);
            }

            // Skip empty lines and commas between objects
            let trimmed = line.trim();
            if !found_start && (trimmed.is_empty() || trimmed == ",") {
                continue;
            }

            // Count braces
            for ch in line.chars() {
                match ch {
                    '{' => {
                        if !found_start {
                            found_start = true;
                        }
                        self.brace_depth += 1;
                    }
                    '}' => {
                        self.brace_depth -= 1;
                    }
                    _ => {}
                }
            }

            if found_start {
                self.buffer.push_str(&line);

                // Check buffer size limit
                if self.buffer.len() > self.config.max_message_size {
                    return Err(StreamingError::BufferOverflow {
                        max_size: self.config.max_message_size,
                        actual_size: self.buffer.len(),
                    });
                }

                // Complete object found
                if self.brace_depth == 0 {
                    // Remove trailing comma if present
                    let result = self.buffer.trim().trim_end_matches(',').to_string();
                    return Ok(Some(result));
                }
            }
        }
    }

    /// Parses a JSON string into a Message using shared parsing logic.
    fn parse_message_from_json(json_str: &str) -> StreamingResult<Option<Message>> {
        let msg: TelegramRawMessage = serde_json::from_str(json_str)?;
        Ok(parse_telegram_message(&msg))
    }
}

impl<R: BufRead + Seek + Send> MessageIterator for TelegramMessageIterator<R> {
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

impl<R: BufRead + Seek + Send> Iterator for TelegramMessageIterator<R> {
    type Item = StreamingResult<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            match self.read_next_object() {
                Ok(Some(json_str)) => {
                    match Self::parse_message_from_json(&json_str) {
                        Ok(Some(msg)) => return Some(Ok(msg)),
                        Ok(None) => {} // Skip non-messages, try next
                        Err(_) if self.config.skip_invalid => {} // Skip invalid
                        Err(e) => return Some(Err(e)),
                    }
                }
                Ok(None) => return None, // End of array
                Err(_) if self.config.skip_invalid => {}
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::telegram::extract_telegram_text;
    use std::io::Cursor;

    fn create_test_json() -> String {
        r#"{
  "name": "Test Chat",
  "type": "personal_chat",
  "messages": [
    {"id": 1, "type": "message", "date_unixtime": "1705314600", "from": "Alice", "text": "Hello!"},
    {"id": 2, "type": "message", "date_unixtime": "1705314660", "from": "Bob", "text": "Hi!"},
    {"id": 3, "type": "service", "action": "pin_message"},
    {"id": 4, "type": "message", "date_unixtime": "1705314720", "from": "Alice", "text": "Bye!"}
  ]
}"#
        .to_string()
    }

    // =========================================================================
    // Constructor tests
    // =========================================================================

    #[test]
    fn test_parser_new() {
        let parser = TelegramStreamingParser::new();
        assert_eq!(parser.name(), "Telegram (Streaming)");
    }

    #[test]
    fn test_parser_default() {
        let parser = TelegramStreamingParser::default();
        assert_eq!(parser.name(), "Telegram (Streaming)");
    }

    #[test]
    fn test_parser_with_config() {
        let config = StreamingConfig::default()
            .with_buffer_size(512 * 1024)
            .with_max_message_size(2 * 1024 * 1024)
            .with_skip_invalid(true);
        let parser = TelegramStreamingParser::with_config(config);
        assert_eq!(parser.name(), "Telegram (Streaming)");
    }

    #[test]
    fn test_recommended_buffer_size() {
        let parser = TelegramStreamingParser::new();
        assert!(parser.recommended_buffer_size() > 0);
    }

    // =========================================================================
    // Basic parsing tests
    // =========================================================================

    #[test]
    fn test_streaming_parser_basic() {
        let json = create_test_json();
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let mut iterator =
            TelegramMessageIterator::new(reader, json.len() as u64, StreamingConfig::default())
                .unwrap();

        let messages: Vec<_> = iterator.by_ref().filter_map(Result::ok).collect();

        assert_eq!(messages.len(), 3); // Service message skipped
        assert_eq!(messages[0].sender, "Alice");
        assert_eq!(messages[0].content, "Hello!");
        assert_eq!(messages[1].sender, "Bob");
        assert_eq!(messages[2].content, "Bye!");
    }

    #[test]
    fn test_empty_messages_array() {
        let json = r#"{"name": "Chat", "messages": []}"#;
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let mut iterator =
            TelegramMessageIterator::new(reader, json.len() as u64, StreamingConfig::default())
                .unwrap();

        let messages: Vec<_> = iterator.by_ref().filter_map(Result::ok).collect();
        assert!(messages.is_empty());
    }

    // =========================================================================
    // Progress and iterator trait tests
    // =========================================================================

    #[test]
    fn test_progress_reporting() {
        let json = create_test_json();
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let mut iterator =
            TelegramMessageIterator::new(reader, json.len() as u64, StreamingConfig::default())
                .unwrap();

        // Consume all messages
        let _: Vec<_> = iterator.by_ref().collect();

        let progress = iterator.progress().unwrap();
        assert!(progress > 90.0); // Should be close to 100%
    }

    #[test]
    fn test_progress_with_zero_file_size() {
        let json = create_test_json();
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let iterator =
            TelegramMessageIterator::new(reader, 0, StreamingConfig::default()).unwrap();

        assert!(iterator.progress().is_none());
    }

    #[test]
    fn test_bytes_processed() {
        let json = create_test_json();
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let mut iterator =
            TelegramMessageIterator::new(reader, json.len() as u64, StreamingConfig::default())
                .unwrap();

        let initial_bytes = iterator.bytes_processed();
        assert!(initial_bytes > 0);

        let _ = iterator.next();
        let bytes_after = iterator.bytes_processed();
        assert!(bytes_after > initial_bytes);
    }

    #[test]
    fn test_total_bytes() {
        let json = create_test_json();
        let file_size = json.len() as u64;
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let iterator =
            TelegramMessageIterator::new(reader, file_size, StreamingConfig::default()).unwrap();

        assert_eq!(iterator.total_bytes(), Some(file_size));
    }

    // =========================================================================
    // Error handling tests
    // =========================================================================

    #[test]
    fn test_no_messages_array() {
        let json = r#"{"name": "Chat"}"#;
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let result =
            TelegramMessageIterator::new(reader, json.len() as u64, StreamingConfig::default());

        assert!(result.is_err());
        // Verify it's an InvalidFormat error
        if let Err(StreamingError::InvalidFormat(msg)) = result {
            assert!(msg.contains("messages"));
        } else {
            panic!("Expected InvalidFormat error");
        }
    }

    #[test]
    fn test_skip_invalid_messages() {
        let json = r#"{
  "name": "Chat",
  "messages": [
    {"invalid": "object"},
    {"id": 1, "type": "message", "date_unixtime": "1705314600", "from": "Alice", "text": "Valid!"}
  ]
}"#;
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let config = StreamingConfig::default().with_skip_invalid(true);
        let mut iterator =
            TelegramMessageIterator::new(reader, json.len() as u64, config).unwrap();

        let messages: Vec<_> = iterator.by_ref().filter_map(Result::ok).collect();

        // Should skip invalid and return valid message
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Valid!");
    }

    #[test]
    fn test_invalid_message_without_skip() {
        let json = r#"{
  "name": "Chat",
  "messages": [
    {"id": "invalid_id_type"}
  ]
}"#;
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let config = StreamingConfig::default().with_skip_invalid(false);
        let mut iterator =
            TelegramMessageIterator::new(reader, json.len() as u64, config).unwrap();

        // First message should be an error
        let first = iterator.next();
        assert!(first.is_some());
        assert!(first.unwrap().is_err());
    }

    // =========================================================================
    // Content type tests
    // =========================================================================

    #[test]
    fn test_extract_text_with_formatting() {
        let value = serde_json::json!([
            "Hello, ",
            {"type": "bold", "text": "world"},
            "!"
        ]);
        assert_eq!(extract_telegram_text(&value), "Hello, world!");
    }

    #[test]
    fn test_parser_name() {
        let parser = TelegramStreamingParser::new();
        assert_eq!(parser.name(), "Telegram (Streaming)");
    }

    #[test]
    fn test_multiline_message() {
        let json = r#"{
  "name": "Chat",
  "messages": [
    {
      "id": 1,
      "type": "message",
      "date_unixtime": "1705314600",
      "from": "Alice",
      "text": "Line1\nLine2\nLine3"
    }
  ]
}"#;
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let mut iterator =
            TelegramMessageIterator::new(reader, json.len() as u64, StreamingConfig::default())
                .unwrap();

        let messages: Vec<_> = iterator.by_ref().filter_map(Result::ok).collect();
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_iterator_finished_returns_none() {
        let json = r#"{"name": "Chat", "messages": [{"id": 1, "type": "message", "date_unixtime": "1000", "from": "A", "text": "Hi"}]}"#;
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let mut iterator =
            TelegramMessageIterator::new(reader, json.len() as u64, StreamingConfig::default())
                .unwrap();

        // Consume all messages
        let _: Vec<_> = iterator.by_ref().collect();

        // Additional calls should return None
        assert!(iterator.next().is_none());
        assert!(iterator.next().is_none());
    }

    #[test]
    fn test_service_messages_skipped() {
        let json = r#"{
  "name": "Chat",
  "messages": [
    {"id": 1, "type": "service", "action": "pin_message"},
    {"id": 2, "type": "service", "action": "create_group"}
  ]
}"#;
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let mut iterator =
            TelegramMessageIterator::new(reader, json.len() as u64, StreamingConfig::default())
                .unwrap();

        let messages: Vec<_> = iterator.by_ref().filter_map(Result::ok).collect();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_formatted_text_array() {
        let json = r#"{
  "name": "Chat",
  "messages": [
    {"id": 1, "type": "message", "date_unixtime": "1705314600", "from": "Alice", "text": ["Hello ", {"type": "bold", "text": "World"}, "!"]}
  ]
}"#;
        let cursor = Cursor::new(json.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let mut iterator =
            TelegramMessageIterator::new(reader, json.len() as u64, StreamingConfig::default())
                .unwrap();

        let messages: Vec<_> = iterator.by_ref().filter_map(Result::ok).collect();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello World!");
    }
}
