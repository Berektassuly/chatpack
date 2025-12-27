//! Message filtering by date and sender.
//!
//! This module provides filtering capabilities for chat messages:
//! - Filter by date range (after/before)
//! - Filter by sender name (case-insensitive)
//!
//! # Example
//!
//! ```rust
//! use chatpack::core::filter::{FilterConfig, apply_filters};
//! use chatpack::Message;
//!
//! let messages = vec![
//!     Message::new("Alice", "Hello"),
//!     Message::new("Bob", "Hi there"),
//!     Message::new("Alice", "How are you?"),
//! ];
//!
//! // Filter to only Alice's messages
//! let config = FilterConfig::new().with_user("Alice".to_string());
//! let filtered = apply_filters(messages, &config);
//!
//! assert_eq!(filtered.len(), 2);
//! ```

use chrono::{DateTime, NaiveDate, Utc};

use crate::error::ChatpackError;
use crate::Message;

/// Configuration for filtering messages.
///
/// Use the builder pattern to construct filter configurations:
///
/// ```rust
/// use chatpack::core::filter::FilterConfig;
///
/// let config = FilterConfig::new()
///     .after_date("2024-01-01").unwrap()
///     .before_date("2024-12-31").unwrap()
///     .with_user("Alice".to_string());
/// ```
#[derive(Debug, Clone, Default)]
pub struct FilterConfig {
    /// Only include messages after this date
    pub after: Option<DateTime<Utc>>,
    /// Only include messages before this date
    pub before: Option<DateTime<Utc>>,
    /// Only include messages from this sender (case-insensitive)
    pub from: Option<String>,
}

impl FilterConfig {
    /// Creates a new empty filter configuration.
    ///
    /// No filters are active by default.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse date string in YYYY-MM-DD format and set as "after" filter.
    ///
    /// The time is set to 00:00:00 UTC, so the specified date is included.
    ///
    /// # Errors
    ///
    /// Returns [`ChatpackError::InvalidDate`] if the date string
    /// doesn't match YYYY-MM-DD format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::core::filter::FilterConfig;
    ///
    /// let config = FilterConfig::new()
    ///     .after_date("2024-01-01")
    ///     .unwrap();
    /// ```
    pub fn after_date(mut self, date_str: &str) -> Result<Self, ChatpackError> {
        let dt = parse_date_start(date_str)?;
        self.after = Some(dt);
        Ok(self)
    }

    /// Parse date string in YYYY-MM-DD format and set as "before" filter.
    ///
    /// The time is set to 23:59:59 UTC to include the entire specified day.
    ///
    /// # Errors
    ///
    /// Returns [`ChatpackError::InvalidDate`] if the date string
    /// doesn't match YYYY-MM-DD format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::core::filter::FilterConfig;
    ///
    /// let config = FilterConfig::new()
    ///     .before_date("2024-12-31")
    ///     .unwrap();
    /// ```
    pub fn before_date(mut self, date_str: &str) -> Result<Self, ChatpackError> {
        let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| ChatpackError::invalid_date(date_str))?;

        // End of the day to include the full day
        let naive_dt = naive.and_hms_opt(23, 59, 59).unwrap();
        let dt = naive_dt.and_utc();
        self.before = Some(dt);
        Ok(self)
    }

    /// Set a `DateTime<Utc>` directly as the "after" filter.
    /// Use this when you already have a parsed `DateTime`.
    #[must_use]
    pub fn with_after(mut self, dt: DateTime<Utc>) -> Self {
        self.after = Some(dt);
        self
    }

    /// Set a `DateTime<Utc>` directly as the "before" filter.
    ///
    /// Use this when you already have a parsed `DateTime`.
    #[must_use]
    pub fn with_before(mut self, dt: DateTime<Utc>) -> Self {
        self.before = Some(dt);
        self
    }

    /// Set the sender filter.
    ///
    /// Filtering is case-insensitive for ASCII characters.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::core::filter::FilterConfig;
    ///
    /// // Both "Alice" and "alice" will match
    /// let config = FilterConfig::new()
    ///     .with_user("Alice".to_string());
    /// ```
    #[must_use]
    pub fn with_user(mut self, user: String) -> Self {
        self.from = Some(user);
        self
    }

    /// Check if any filter is active.
    ///
    /// Returns `true` if at least one of after, before, or from is set.
    pub fn is_active(&self) -> bool {
        self.after.is_some() || self.before.is_some() || self.from.is_some()
    }

    /// Check if date filters are active.
    pub fn has_date_filter(&self) -> bool {
        self.after.is_some() || self.before.is_some()
    }

    /// Check if sender filter is active.
    pub fn has_user_filter(&self) -> bool {
        self.from.is_some()
    }
}

/// Parse a date string in YYYY-MM-DD format to `DateTime`<Utc> at start of day.
fn parse_date_start(date_str: &str) -> Result<DateTime<Utc>, ChatpackError> {
    let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| ChatpackError::invalid_date(date_str))?;

    // Start of the day
    let naive_dt = naive.and_hms_opt(0, 0, 0).unwrap();
    Ok(naive_dt.and_utc())
}

/// Apply filters to a vector of messages.
///
/// # Behavior
///
/// - Messages matching all active filters are kept
/// - Sender matching is case-insensitive (ASCII)
/// - Messages without timestamps are **excluded** when date filters are active
///
/// # Performance
///
/// This function consumes the input vector and returns a new filtered vector.
/// For large datasets, consider streaming approaches.
///
/// # Example
///
/// ```rust
/// use chatpack::core::filter::{FilterConfig, apply_filters};
/// use chatpack::Message;
///
/// let messages = vec![
///     Message::new("Alice", "Hello"),
///     Message::new("Bob", "Hi"),
/// ];
///
/// let config = FilterConfig::new().with_user("Alice".to_string());
/// let filtered = apply_filters(messages, &config);
///
/// assert_eq!(filtered.len(), 1);
/// assert_eq!(filtered[0].sender(), "Alice");
/// ```
pub fn apply_filters(messages: Vec<Message>, config: &FilterConfig) -> Vec<Message> {
    if !config.is_active() {
        return messages;
    }

    messages
        .into_iter()
        .filter(|msg| {
            // Filter by sender (case-insensitive)
            if let Some(ref from) = config.from {
                if !msg.sender.eq_ignore_ascii_case(from) {
                    return false;
                }
            }

            // Filter by date (only if message has timestamp)
            if config.has_date_filter() {
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
    use chrono::TimeZone;

    fn make_msg(sender: &str, content: &str, ts: Option<&str>) -> Message {
        let mut msg = Message::new(sender, content);
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
        assert!(matches!(
            result,
            Err(ChatpackError::InvalidDate { .. })
        ));
    }

    #[test]
    fn test_combined_filters() {
        let messages = vec![
            make_msg("Alice", "Old Alice", Some("2024-01-01")),
            make_msg("Alice", "New Alice", Some("2024-06-15")),
            make_msg("Bob", "New Bob", Some("2024-06-15")),
        ];

        let config = FilterConfig::new()
            .after_date("2024-06-01")
            .unwrap()
            .with_user("Alice".to_string());

        let filtered = apply_filters(messages, &config);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].sender, "Alice");
        assert_eq!(filtered[0].content, "New Alice");
    }

    #[test]
    fn test_with_datetime_directly() {
        let dt = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap();
        let config = FilterConfig::new().with_after(dt);
        assert_eq!(config.after, Some(dt));
    }

    #[test]
    fn test_is_active() {
        assert!(!FilterConfig::new().is_active());
        assert!(FilterConfig::new().with_user("Alice".into()).is_active());
        assert!(
            FilterConfig::new()
                .after_date("2024-01-01")
                .unwrap()
                .is_active()
        );
    }
}
