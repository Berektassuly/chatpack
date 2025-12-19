//! Message filtering by date and sender.

use chrono::{DateTime, NaiveDate, Utc};

use super::models::InternalMessage;

/// Configuration for filtering messages.
#[derive(Debug, Clone, Default)]
pub struct FilterConfig {
    /// Only include messages after this date
    pub after: Option<DateTime<Utc>>,
    /// Only include messages before this date
    pub before: Option<DateTime<Utc>>,
    /// Only include messages from this sender
    pub from: Option<String>,
}

impl FilterConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse date string in YYYY-MM-DD format and set as "after" filter.
    /// The time is set to 00:00:00 UTC.
    pub fn after_date(mut self, date_str: &str) -> Result<Self, FilterError> {
        let dt = parse_date(date_str)?;
        self.after = Some(dt);
        Ok(self)
    }

    /// Parse date string in YYYY-MM-DD format and set as "before" filter.
    /// The time is set to 23:59:59 UTC to include the entire day.
    pub fn before_date(mut self, date_str: &str) -> Result<Self, FilterError> {
        let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| FilterError::InvalidDateFormat(date_str.to_string()))?;

        // End of the day
        let naive_dt = naive.and_hms_opt(23, 59, 59).unwrap();
        let dt = naive_dt.and_utc();
        self.before = Some(dt);
        Ok(self)
    }

    /// Set the sender filter.
    pub fn with_user(mut self, user: String) -> Self {
        self.from = Some(user);
        self
    }

    /// Check if any filter is active.
    pub fn is_active(&self) -> bool {
        self.after.is_some() || self.before.is_some() || self.from.is_some()
    }
}

/// Errors that can occur during filtering.
#[derive(Debug)]
pub enum FilterError {
    /// Invalid date format (expected YYYY-MM-DD)
    InvalidDateFormat(String),
}

impl std::fmt::Display for FilterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterError::InvalidDateFormat(s) => {
                write!(f, "Invalid date format: '{}'. Expected YYYY-MM-DD", s)
            }
        }
    }
}

impl std::error::Error for FilterError {}

/// Parse a date string in YYYY-MM-DD format to DateTime<Utc>.
fn parse_date(date_str: &str) -> Result<DateTime<Utc>, FilterError> {
    let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| FilterError::InvalidDateFormat(date_str.to_string()))?;

    // Start of the day
    let naive_dt = naive.and_hms_opt(0, 0, 0).unwrap();
    Ok(naive_dt.and_utc())
}

/// Apply filters to a vector of messages.
///
/// Note: Messages without timestamps are excluded when date filters are active.
/// This ensures consistent behavior - if you're filtering by date, you probably
/// want messages with known dates.
pub fn apply_filters(
    messages: Vec<InternalMessage>,
    config: &FilterConfig,
) -> Vec<InternalMessage> {
    if !config.is_active() {
        return messages;
    }

    messages
        .into_iter()
        .filter(|msg| {
            // Filter by sender
            if let Some(ref from) = config.from
                && !msg.sender.eq_ignore_ascii_case(from)
            {
                return false;
            }

            // Filter by date (only if message has timestamp)
            if config.after.is_some() || config.before.is_some() {
                match msg.timestamp {
                    Some(ts) => {
                        if config.after.is_some_and(|after| ts < after) {
                            return false;
                        }
                        if config.before.is_some_and(|before| ts > before) {
                            return false;
                        }
                    }
                    None => {
                        // No timestamp - exclude from date-filtered results
                        return false;
                    }
                }
            }

            true
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(sender: &str, content: &str, ts: Option<&str>) -> InternalMessage {
        let mut msg = InternalMessage::new(sender, content);
        if let Some(ts_str) = ts {
            let naive = NaiveDate::parse_from_str(ts_str, "%Y-%m-%d").unwrap();
            msg.timestamp = Some(naive.and_hms_opt(12, 0, 0).unwrap().and_utc());
        }
        msg
    }

    #[test]
    fn test_filter_by_sender() {
        let messages = vec![
            make_msg("Alice", "Hello", None),
            make_msg("Bob", "Hi", None),
            make_msg("alice", "Bye", None), // lowercase
        ];

        let config = FilterConfig::new().with_user("Alice".to_string());
        let filtered = apply_filters(messages, &config);

        assert_eq!(filtered.len(), 2);
        assert!(
            filtered
                .iter()
                .all(|m| m.sender.eq_ignore_ascii_case("Alice"))
        );
    }

    #[test]
    fn test_filter_by_date_after() {
        let messages = vec![
            make_msg("Alice", "Old", Some("2024-01-01")),
            make_msg("Alice", "New", Some("2024-06-15")),
        ];

        let config = FilterConfig::new().after_date("2024-06-01").unwrap();
        let filtered = apply_filters(messages, &config);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].content, "New");
    }

    #[test]
    fn test_filter_by_date_before() {
        let messages = vec![
            make_msg("Alice", "Old", Some("2024-01-01")),
            make_msg("Alice", "New", Some("2024-06-15")),
        ];

        let config = FilterConfig::new().before_date("2024-03-01").unwrap();
        let filtered = apply_filters(messages, &config);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].content, "Old");
    }

    #[test]
    fn test_no_timestamp_excluded_when_date_filter() {
        let messages = vec![
            make_msg("Alice", "With date", Some("2024-06-15")),
            make_msg("Alice", "No date", None),
        ];

        let config = FilterConfig::new().after_date("2024-01-01").unwrap();
        let filtered = apply_filters(messages, &config);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].content, "With date");
    }

    #[test]
    fn test_invalid_date_format() {
        let result = FilterConfig::new().after_date("01-01-2024");
        assert!(result.is_err());
    }
}
