//! Shared WhatsApp parsing utilities.
//!
//! This module contains types and functions shared between the standard
//! and streaming WhatsApp parsers.

use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;

/// Detected date format variants for WhatsApp exports.
///
/// WhatsApp exports vary by locale and platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFormat {
    /// US format: M/D/YY or M/D/YYYY with optional AM/PM
    /// Example: [1/15/24, 10:30:45 AM]
    US,
    /// EU format with dots in brackets: DD.MM.YY or DD.MM.YYYY
    /// Example: [15.01.24, 10:30:45]
    EuDotBracketed,
    /// EU format with dots, no brackets: DD.MM.YYYY
    /// Example: 26.10.2025, 20:40 - Sender: Message
    EuDotNoBracket,
    /// EU format with slashes, no brackets: DD/MM/YYYY
    /// Example: 15/01/2024, 10:30 -
    EuSlash,
    /// Bracketed EU with slashes
    /// Example: [15/01/2024, 10:30:45]
    EuSlashBracketed,
}

impl DateFormat {
    /// Returns regex pattern for this date format.
    pub fn pattern(self) -> &'static str {
        match self {
            // [1/15/24, 10:30:45 AM] Sender: Message
            DateFormat::US => {
                r"^\[(\d{1,2}/\d{1,2}/\d{2,4}),\s(\d{1,2}:\d{2}(?::\d{2})?(?:\s?[APap][Mm])?)\]\s([^:]+):\s?(.*)"
            }
            // [15.01.24, 10:30:45] Sender: Message
            DateFormat::EuDotBracketed => {
                r"^\[(\d{2}\.\d{2}\.\d{2,4}),\s(\d{2}:\d{2}(?::\d{2})?)\]\s([^:]+):\s?(.*)"
            }
            // 26.10.2025, 20:40 - Sender: Message
            DateFormat::EuDotNoBracket => {
                r"^(\d{2}\.\d{2}\.\d{2,4}),\s(\d{2}:\d{2}(?::\d{2})?)\s-\s([^:]+):\s?(.*)"
            }
            // 15/01/2024, 10:30 - Sender: Message
            DateFormat::EuSlash => {
                r"^(\d{2}/\d{2}/\d{2,4}),\s(\d{2}:\d{2}(?::\d{2})?)\s-\s([^:]+):\s?(.*)"
            }
            // [15/01/2024, 10:30:45] Sender: Message
            DateFormat::EuSlashBracketed => {
                r"^\[(\d{2}/\d{2}/\d{2,4}),\s(\d{2}:\d{2}(?::\d{2})?)\]\s([^:]+):\s?(.*)"
            }
        }
    }

    /// Returns date parsing format strings for chrono.
    pub fn date_parse_formats(self) -> &'static [&'static str] {
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

    /// Returns all format variants.
    pub fn all() -> &'static [DateFormat] {
        &[
            DateFormat::US,
            DateFormat::EuDotBracketed,
            DateFormat::EuDotNoBracket,
            DateFormat::EuSlash,
            DateFormat::EuSlashBracketed,
        ]
    }
}

/// Parse timestamp from date and time strings.
pub fn parse_whatsapp_timestamp(
    date_str: &str,
    time_str: &str,
    format: DateFormat,
) -> Option<DateTime<Utc>> {
    let datetime_str = format!("{date_str}, {time_str}");

    for parse_format in format.date_parse_formats() {
        if let Ok(naive) = NaiveDateTime::parse_from_str(&datetime_str, parse_format) {
            return Some(naive.and_utc());
        }
    }

    None
}

/// Check if a line is a system message (no actual sender).
///
/// System messages include: group created, user added/left, encryption notice, etc.
pub fn is_whatsapp_system_message(sender: &str, content: &str) -> bool {
    // English system indicators
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

    // Russian system indicators
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

    // Check English indicators
    for indicator in &system_indicators_en {
        if content_lower.contains(&indicator.to_lowercase()) {
            return true;
        }
    }

    // Check Russian indicators (case-sensitive for Cyrillic)
    for indicator in &system_indicators_ru {
        if content.contains(indicator) {
            return true;
        }
    }

    // Check if sender is empty or system-like
    sender.trim().is_empty() || sender_lower.contains("whatsapp") || sender_lower.contains("system")
}

/// Detection result for format auto-detection.
struct FormatDetector {
    format: DateFormat,
    regex: Regex,
}

impl FormatDetector {
    fn new(format: DateFormat) -> Self {
        Self {
            format,
            regex: Regex::new(format.pattern()).unwrap(),
        }
    }

    fn matches(&self, line: &str) -> bool {
        self.regex.is_match(line)
    }
}

/// Auto-detect date format by analyzing sample lines.
///
/// Analyzes the provided lines and returns the most likely format.
/// Returns `None` if no format matches any lines.
pub fn detect_whatsapp_format(lines: &[&str]) -> Option<DateFormat> {
    let detectors: Vec<FormatDetector> = DateFormat::all()
        .iter()
        .map(|&f| FormatDetector::new(f))
        .collect();

    let mut scores = vec![0usize; detectors.len()];

    for line in lines {
        for (i, detector) in detectors.iter().enumerate() {
            if detector.matches(line) {
                scores[i] += 1;
            }
        }
    }

    // Find the winner (highest score)
    let max_score = *scores.iter().max()?;
    if max_score == 0 {
        return None;
    }

    let winner_idx = scores.iter().position(|&s| s == max_score)?;
    Some(detectors[winner_idx].format)
}

/// Auto-detect date format from owned strings (for streaming).
pub fn detect_whatsapp_format_owned(lines: &[String]) -> Option<DateFormat> {
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    detect_whatsapp_format(&refs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_us() {
        let lines = vec![
            "[1/15/24, 10:30:45 AM] Alice: Hello",
            "[1/15/24, 10:31:00 AM] Bob: Hi there",
        ];
        assert_eq!(detect_whatsapp_format(&lines), Some(DateFormat::US));
    }

    #[test]
    fn test_detect_format_eu_dot_bracketed() {
        let lines = vec![
            "[15.01.24, 10:30:45] Alice: Hello",
            "[15.01.24, 10:31:00] Bob: Hi there",
        ];
        assert_eq!(
            detect_whatsapp_format(&lines),
            Some(DateFormat::EuDotBracketed)
        );
    }

    #[test]
    fn test_detect_format_eu_dot_no_bracket() {
        let lines = vec![
            "26.10.2025, 20:40 - Alice: Hello",
            "26.10.2025, 20:41 - Bob: Hi there",
        ];
        assert_eq!(
            detect_whatsapp_format(&lines),
            Some(DateFormat::EuDotNoBracket)
        );
    }

    #[test]
    fn test_detect_format_eu_slash() {
        let lines = vec![
            "15/01/2024, 10:30 - Alice: Hello",
            "15/01/2024, 10:31 - Bob: Hi there",
        ];
        assert_eq!(detect_whatsapp_format(&lines), Some(DateFormat::EuSlash));
    }

    #[test]
    fn test_is_system_message_english() {
        assert!(is_whatsapp_system_message(
            "Alice",
            "Messages and calls are end-to-end encrypted"
        ));
        assert!(is_whatsapp_system_message(
            "Bob",
            "added Charlie to the group"
        ));
        assert!(is_whatsapp_system_message("Alice", "left"));
        assert!(!is_whatsapp_system_message("Alice", "Hello everyone!"));
        assert!(!is_whatsapp_system_message("Bob", "<Media omitted>"));
    }

    #[test]
    fn test_is_system_message_russian() {
        assert!(is_whatsapp_system_message(
            "Система",
            "Сообщения и звонки защищены сквозным шифрованием"
        ));
        assert!(is_whatsapp_system_message("Bob", "Подробнее"));
        assert!(!is_whatsapp_system_message("Муха", "Добрый вечер"));
    }

    #[test]
    fn test_parse_timestamp_us() {
        let ts = parse_whatsapp_timestamp("1/15/24", "10:30:45 AM", DateFormat::US);
        assert!(ts.is_some());
    }

    #[test]
    fn test_parse_timestamp_eu_dot() {
        let ts = parse_whatsapp_timestamp("15.01.24", "10:30:45", DateFormat::EuDotBracketed);
        assert!(ts.is_some());

        let ts2 = parse_whatsapp_timestamp("26.10.2025", "20:40", DateFormat::EuDotNoBracket);
        assert!(ts2.is_some());
    }

    #[test]
    fn test_empty_sender_is_system() {
        assert!(is_whatsapp_system_message("", "Some message"));
        assert!(is_whatsapp_system_message("   ", "Some message"));
    }
}
