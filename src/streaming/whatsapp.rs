//! Streaming parser for WhatsApp TXT exports.
//!
//! WhatsApp exports are text files with various date formats:
//! - US: `[1/15/24, 10:30:45 AM] Sender: Message`
//! - EU (dot): `[15.01.24, 10:30:45] Sender: Message`
//! - EU (no bracket): `26.10.2025, 20:40 - Sender: Message`
//! - EU (slash): `15/01/2024, 10:30 - Sender: Message`
//!
//! This parser streams line-by-line, handling multi-line messages.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;

use crate::error::ChatpackError;
use crate::Message;

use super::{MessageIterator, StreamingConfig, StreamingParser, StreamingResult};

/// Streaming parser for WhatsApp TXT exports.
pub struct WhatsAppStreamingParser {
    config: StreamingConfig,
}

impl WhatsAppStreamingParser {
    pub fn new() -> Self {
        Self {
            config: StreamingConfig::default(),
        }
    }

    pub fn with_config(config: StreamingConfig) -> Self {
        Self { config }
    }
}

impl Default for WhatsAppStreamingParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingParser for WhatsAppStreamingParser {
    fn name(&self) -> &'static str {
        "WhatsApp (Streaming)"
    }

    fn stream(&self, file_path: &str) -> Result<Box<dyn MessageIterator>, ChatpackError> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let file_size = file.metadata()?.len();

        let reader = BufReader::with_capacity(self.config.buffer_size, file);
        let iterator = WhatsAppMessageIterator::new(reader, file_size, self.config)?;

        Ok(Box::new(iterator))
    }

    fn recommended_buffer_size(&self) -> usize {
        self.config.buffer_size
    }
}

/// Detected date format variants.
#[derive(Debug, Clone, Copy, PartialEq)]
enum DateFormat {
    US,
    EuDotBracketed,
    EuDotNoBracket,
    EuSlash,
    EuSlashBracketed,
}

impl DateFormat {
    fn pattern(self) -> &'static str {
        match self {
            DateFormat::US => {
                r"^\[(\d{1,2}/\d{1,2}/\d{2,4}),\s(\d{1,2}:\d{2}(?::\d{2})?(?:\s?[APap][Mm])?)\]\s([^:]+):\s?(.*)"
            }
            DateFormat::EuDotBracketed => {
                r"^\[(\d{2}\.\d{2}\.\d{2,4}),\s(\d{2}:\d{2}(?::\d{2})?)\]\s([^:]+):\s?(.*)"
            }
            DateFormat::EuDotNoBracket => {
                r"^(\d{2}\.\d{2}\.\d{2,4}),\s(\d{2}:\d{2}(?::\d{2})?)\s-\s([^:]+):\s?(.*)"
            }
            DateFormat::EuSlash => {
                r"^(\d{2}/\d{2}/\d{2,4}),\s(\d{2}:\d{2}(?::\d{2})?)\s-\s([^:]+):\s?(.*)"
            }
            DateFormat::EuSlashBracketed => {
                r"^\[(\d{2}/\d{2}/\d{2,4}),\s(\d{2}:\d{2}(?::\d{2})?)\]\s([^:]+):\s?(.*)"
            }
        }
    }

    fn date_parse_formats(self) -> &'static [&'static str] {
        match self {
            DateFormat::US => &[
                "%m/%d/%y, %I:%M:%S %p",
                "%m/%d/%y, %I:%M %p",
                "%m/%d/%Y, %I:%M:%S %p",
                "%m/%d/%Y, %I:%M %p",
                "%m/%d/%y, %H:%M:%S",
                "%m/%d/%y, %H:%M",
                "%m/%d/%Y, %H:%M:%S",
                "%m/%d/%Y, %H:%M",
            ],
            DateFormat::EuDotBracketed | DateFormat::EuDotNoBracket => &[
                "%d.%m.%y, %H:%M:%S",
                "%d.%m.%y, %H:%M",
                "%d.%m.%Y, %H:%M:%S",
                "%d.%m.%Y, %H:%M",
            ],
            DateFormat::EuSlash | DateFormat::EuSlashBracketed => &[
                "%d/%m/%y, %H:%M:%S",
                "%d/%m/%y, %H:%M",
                "%d/%m/%Y, %H:%M:%S",
                "%d/%m/%Y, %H:%M",
            ],
        }
    }
}

#[derive(Debug, Default)]
struct PendingMessage {
    sender: String,
    content: String,
    timestamp: Option<DateTime<Utc>>,
}

impl PendingMessage {
    fn is_empty(&self) -> bool {
        self.sender.is_empty()
    }

    fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    fn into_message(self) -> Option<Message> {
        if self.sender.is_empty() || self.content.trim().is_empty() {
            return None;
        }

        if is_system_message(&self.sender, &self.content) {
            return None;
        }

        Some(Message::with_metadata(
            self.sender,
            self.content.trim().to_string(),
            self.timestamp,
            None,
            None,
            None,
        ))
    }
}

fn is_system_message(sender: &str, content: &str) -> bool {
    let system_indicators_en = [
        "Messages and calls are end-to-end encrypted",
        "created group",
        "added",
        "removed",
        "left",
        "changed the subject",
        "changed this group's icon",
        "changed the group description",
        "deleted this group's icon",
        "changed their phone number",
        "joined using this group's invite link",
        "security code changed",
        "You're now an admin",
        "is now an admin",
        "disappeared",
        "turned on disappearing messages",
        "turned off disappearing messages",
    ];

    let system_indicators_ru = [
        "Сообщения и звонки защищены сквозным шифрованием",
        "создал(а) группу",
        "добавил",
        "удалил",
        "вышел",
        "покинул",
        "изменил тему",
        "изменил иконку группы",
        "изменил описание группы",
        "удалил иконку группы",
        "изменил номер телефона",
        "присоединился по ссылке",
        "код безопасности изменён",
        "теперь администратор",
        "включил исчезающие сообщения",
        "выключил исчезающие сообщения",
        "Подробнее",
    ];

    let content_lower = content.to_lowercase();
    let sender_lower = sender.to_lowercase();

    for indicator in &system_indicators_en {
        if content_lower.contains(&indicator.to_lowercase()) {
            return true;
        }
    }

    for indicator in &system_indicators_ru {
        if content.contains(indicator) {
            return true;
        }
    }

    sender.trim().is_empty() || sender_lower.contains("whatsapp") || sender_lower.contains("system")
}

fn parse_timestamp(date_str: &str, time_str: &str, format: DateFormat) -> Option<DateTime<Utc>> {
    let datetime_str = format!("{}, {}", date_str, time_str);

    for parse_format in format.date_parse_formats() {
        if let Ok(naive) = NaiveDateTime::parse_from_str(&datetime_str, parse_format) {
            return Some(naive.and_utc());
        }
    }

    None
}

/// Iterator over WhatsApp messages.
pub struct WhatsAppMessageIterator<R: BufRead> {
    reader: R,
    file_size: u64,
    bytes_read: u64,
    config: StreamingConfig,
    line_buffer: String,
    pending: PendingMessage,
    queued: VecDeque<Message>,
    finished: bool,
    detected_format: Option<DateFormat>,
    format_regex: Option<Regex>,
}

impl<R: BufRead> WhatsAppMessageIterator<R> {
    fn new(mut reader: R, file_size: u64, config: StreamingConfig) -> StreamingResult<Self> {
        // Read first few lines to detect format
        let mut sample_lines = Vec::new();
        let mut sample_bytes = 0u64;

        for _ in 0..20 {
            let mut line = String::new();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            sample_bytes += bytes as u64;
            sample_lines.push(line);
        }

        let detected_format = detect_format(&sample_lines);
        let format_regex = detected_format.map(|f| Regex::new(f.pattern()).unwrap());

        let mut iter = Self {
            reader,
            file_size,
            bytes_read: sample_bytes,
            config,
            line_buffer: String::with_capacity(4096),
            pending: PendingMessage::default(),
            queued: VecDeque::new(),
            finished: false,
            detected_format,
            format_regex,
        };

        // Process sample lines, queuing completed messages
        for line in sample_lines {
            iter.process_line_queuing(&line);
        }

        Ok(iter)
    }

    /// Process line, queuing any completed message before starting new one.
    fn process_line_queuing(&mut self, line: &str) {
        if line.trim().is_empty() {
            return;
        }

        if let (Some(format), Some(regex)) = (self.detected_format, &self.format_regex) {
            if let Some(caps) = regex.captures(line) {
                // New message - queue the pending one first
                if !self.pending.is_empty() {
                    if let Some(msg) = self.pending.take().into_message() {
                        self.queued.push_back(msg);
                    }
                }

                let date_str = caps.get(1).map_or("", |m| m.as_str());
                let time_str = caps.get(2).map_or("", |m| m.as_str());
                let sender = caps.get(3).map_or("", |m| m.as_str().trim());
                let content = caps.get(4).map_or("", |m| m.as_str());

                self.pending.sender = sender.to_string();
                self.pending.content = content.to_string();
                self.pending.timestamp = parse_timestamp(date_str, time_str, format);
                return;
            }
        }

        // Continuation line
        if !self.pending.is_empty() {
            self.pending.content.push('\n');
            self.pending.content.push_str(line.trim_end());
        }
    }

    fn read_line(&mut self) -> std::io::Result<Option<String>> {
        self.line_buffer.clear();
        let bytes = self.reader.read_line(&mut self.line_buffer)?;
        if bytes == 0 {
            return Ok(None);
        }
        self.bytes_read += bytes as u64;
        Ok(Some(self.line_buffer.clone()))
    }
}

fn detect_format(lines: &[String]) -> Option<DateFormat> {
    let formats = [
        DateFormat::US,
        DateFormat::EuDotBracketed,
        DateFormat::EuDotNoBracket,
        DateFormat::EuSlash,
        DateFormat::EuSlashBracketed,
    ];

    let mut scores = [0usize; 5];

    for line in lines {
        for (i, format) in formats.iter().enumerate() {
            if let Ok(regex) = Regex::new(format.pattern()) {
                if regex.is_match(line) {
                    scores[i] += 1;
                }
            }
        }
    }

    let max_score = *scores.iter().max()?;
    if max_score == 0 {
        return None;
    }

    let winner_idx = scores.iter().position(|&s| s == max_score)?;
    Some(formats[winner_idx])
}

impl<R: BufRead + Send> MessageIterator for WhatsAppMessageIterator<R> {
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

impl<R: BufRead + Send> Iterator for WhatsAppMessageIterator<R> {
    type Item = StreamingResult<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        // First, drain queued messages from sample lines
        if let Some(msg) = self.queued.pop_front() {
            return Some(Ok(msg));
        }

        if self.finished && self.pending.is_empty() {
            return None;
        }

        if self.detected_format.is_none() {
            self.finished = true;
            return None;
        }

        loop {
            match self.read_line() {
                Ok(Some(line)) => {
                    if let Some(regex) = &self.format_regex {
                        if regex.is_match(&line) {
                            let to_yield = self.pending.take();
                            self.process_line_queuing(&line);

                            if let Some(msg) = to_yield.into_message() {
                                return Some(Ok(msg));
                            }
                            continue;
                        }
                    }
                    self.process_line_queuing(&line);
                }
                Ok(None) => {
                    self.finished = true;
                    let to_yield = self.pending.take();
                    if let Some(msg) = to_yield.into_message() {
                        return Some(Ok(msg));
                    }
                    return None;
                }
                Err(e) => {
                    if self.config.skip_invalid {
                        continue;
                    }
                    return Some(Err(e.into()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_test_us_format() -> String {
        "[1/15/24, 10:30:00 AM] Alice: Hello everyone!
[1/15/24, 10:31:00 AM] Bob: Hi Alice!
[1/15/24, 10:32:00 AM] Alice: How is everyone doing?
This is a continuation line
[1/15/24, 10:33:00 AM] Charlie: Messages and calls are end-to-end encrypted.
[1/15/24, 10:34:00 AM] Bob: I'm doing great!"
            .to_string()
    }

    fn create_test_eu_format() -> String {
        "[15.01.24, 10:30:00] Alice: Привет всем!
[15.01.24, 10:31:00] Bob: Привет!
[15.01.24, 10:32:00] Alice: Как дела?"
            .to_string()
    }

    #[test]
    fn test_streaming_parser_us_format() {
        let txt = create_test_us_format();
        let cursor = Cursor::new(txt.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let iterator =
            WhatsAppMessageIterator::new(reader, txt.len() as u64, StreamingConfig::default())
                .unwrap();

        let messages: Vec<_> = iterator.filter_map(Result::ok).collect();

        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0].sender, "Alice");
        assert_eq!(messages[0].content, "Hello everyone!");
        assert_eq!(messages[1].sender, "Bob");
        assert!(messages[2].content.contains("continuation"));
    }

    #[test]
    fn test_streaming_parser_eu_format() {
        let txt = create_test_eu_format();
        let cursor = Cursor::new(txt.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let iterator =
            WhatsAppMessageIterator::new(reader, txt.len() as u64, StreamingConfig::default())
                .unwrap();

        let messages: Vec<_> = iterator.filter_map(Result::ok).collect();

        assert_eq!(messages.len(), 3);
        assert!(messages[0].content.contains("Привет"));
    }

    #[test]
    fn test_progress_reporting() {
        let txt = create_test_us_format();
        let cursor = Cursor::new(txt.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let mut iterator =
            WhatsAppMessageIterator::new(reader, txt.len() as u64, StreamingConfig::default())
                .unwrap();

        let _: Vec<_> = iterator.by_ref().collect();

        let progress = iterator.progress().unwrap();
        assert!(progress > 90.0);
    }

    #[test]
    fn test_parser_name() {
        let parser = WhatsAppStreamingParser::new();
        assert_eq!(parser.name(), "WhatsApp (Streaming)");
    }

    #[test]
    fn test_system_message_detection() {
        assert!(is_system_message(
            "Alice",
            "Messages and calls are end-to-end encrypted"
        ));
        assert!(!is_system_message("Alice", "Hello everyone!"));
        assert!(is_system_message("Bob", "added Charlie to the group"));
    }

    #[test]
    fn test_detect_format_us() {
        let lines = vec![
            "[1/15/24, 10:30:45 AM] Alice: Hello".to_string(),
            "[1/15/24, 10:31:00 AM] Bob: Hi there".to_string(),
        ];
        assert_eq!(detect_format(&lines), Some(DateFormat::US));
    }

    #[test]
    fn test_detect_format_eu_dot() {
        let lines = vec![
            "[15.01.24, 10:30:45] Alice: Hello".to_string(),
            "[15.01.24, 10:31:00] Bob: Hi there".to_string(),
        ];
        assert_eq!(detect_format(&lines), Some(DateFormat::EuDotBracketed));
    }

    #[test]
    fn test_multiline_messages() {
        let txt = "[1/15/24, 10:30:00 AM] Alice: Line 1
Line 2
Line 3
[1/15/24, 10:31:00 AM] Bob: Reply";

        let cursor = Cursor::new(txt.as_bytes().to_vec());
        let reader = BufReader::new(cursor);

        let iterator =
            WhatsAppMessageIterator::new(reader, txt.len() as u64, StreamingConfig::default())
                .unwrap();

        let messages: Vec<_> = iterator.filter_map(Result::ok).collect();

        assert_eq!(messages.len(), 2);
        assert!(messages[0].content.contains("Line 1"));
        assert!(messages[0].content.contains("Line 2"));
        assert!(messages[0].content.contains("Line 3"));
    }
}
