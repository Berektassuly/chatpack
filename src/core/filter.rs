//! Filter messages by date range and sender.
//!
//! This module provides [`FilterConfig`] for defining filter criteria and
//! [`apply_filters`] for filtering message collections.
//!
//! # Filter Types
//!
//! | Filter | Method | Description |
//! |--------|--------|-------------|
//! | Date from | [`with_date_from`](FilterConfig::with_date_from) | Messages on or after date |
//! | Date to | [`with_date_to`](FilterConfig::with_date_to) | Messages on or before date |
//! | Sender | [`with_sender`](FilterConfig::with_sender) | Messages from specific user |
//!
//! # Examples
//!
//! ## Filter by Sender
//!
//! ```
//! use chatpack::core::filter::{FilterConfig, apply_filters};
//! use chatpack::Message;
//!
//! let messages = vec![
//!     Message::new("Alice", "Hello"),
//!     Message::new("Bob", "Hi there"),
//!     Message::new("Alice", "How are you?"),
//! ];
//!
//! // Case-insensitive sender matching
//! let config = FilterConfig::new().with_sender("alice");
//! let filtered = apply_filters(messages, &config);
//!
//! assert_eq!(filtered.len(), 2);
//! ```
//!
//! ## Filter by Date Range
//!
//! ```
//! use chatpack::core::filter::{FilterConfig, apply_filters};
//! use chatpack::Message;
//! use chrono::{TimeZone, Utc};
//!
//! # fn main() -> chatpack::Result<()> {
//! let messages = vec![
//!     Message::new("Alice", "Old").with_timestamp(Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap()),
//!     Message::new("Alice", "New").with_timestamp(Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap()),
//! ];
//!
//! let config = FilterConfig::new()
//!     .with_date_from("2024-06-01")?
//!     .with_date_to("2024-12-31")?;
//!
//! let filtered = apply_filters(messages, &config);
//! assert_eq!(filtered.len(), 1);
//! assert_eq!(filtered[0].content, "New");
//! # Ok(())
//! # }
//! ```
//!
//! # Behavior Notes
//!
//! - Messages without timestamps are **excluded** when date filters are active
//! - Sender matching is case-insensitive for ASCII characters
//! - Multiple filters are combined with AND logic

use chrono::{DateTime, NaiveDate, Utc};

use crate::Message;
use crate::error::ChatpackError;

/// Configuration for filtering messages by date and sender.
///
/// Filters are combined with AND logic: a message must match all active
/// filters to be included in the result.
///
/// # Examples
///
/// ```
/// use chatpack::core::filter::FilterConfig;
///
/// # fn main() -> chatpack::Result<()> {
/// // Filter by sender only
/// let by_sender = FilterConfig::new().with_sender("Alice");
///
/// // Filter by date range
/// let by_date = FilterConfig::new()
///     .with_date_from("2024-01-01")?
///     .with_date_to("2024-12-31")?;
///
/// // Combined filters
/// let combined = FilterConfig::new()
///     .with_sender("Alice")
///     .with_date_from("2024-06-01")?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct FilterConfig {
    /// Include only messages on or after this timestamp.
    pub after: Option<DateTime<Utc>>,

    /// Include only messages on or before this timestamp.
    pub before: Option<DateTime<Utc>>,

    /// Include only messages from this sender (case-insensitive).
    pub from: Option<String>,
}

impl FilterConfig {
    /// Creates a new empty filter configuration.
    ///
    /// No filters are active by default; all messages pass through.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the start date filter (inclusive).
    ///
    /// Only messages on or after this date will be included.
    /// Date format: `YYYY-MM-DD`.
    ///
    /// # Errors
    ///
    /// Returns [`ChatpackError::InvalidDate`] if the format is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use chatpack::core::filter::FilterConfig;
    ///
    /// # fn main() -> chatpack::Result<()> {
    /// let config = FilterConfig::new()
    ///     .with_date_from("2024-01-01")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_date_from(mut self, date_str: &str) -> Result<Self, ChatpackError> {
        let dt = parse_date_start(date_str)?;
        self.after = Some(dt);
        Ok(self)
    }

    /// Sets the end date filter (inclusive).
    ///
    /// Only messages on or before this date will be included.
    /// Date format: `YYYY-MM-DD`.
    ///
    /// # Errors
    ///
    /// Returns [`ChatpackError::InvalidDate`] if the format is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use chatpack::core::filter::FilterConfig;
    ///
    /// # fn main() -> chatpack::Result<()> {
    /// let config = FilterConfig::new()
    ///     .with_date_to("2024-12-31")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_date_to(mut self, date_str: &str) -> Result<Self, ChatpackError> {
        let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| ChatpackError::invalid_date(date_str))?;

        // End of the day to include the full day
        let naive_dt = naive.and_hms_opt(23, 59, 59).unwrap();
        let dt = naive_dt.and_utc();
        self.before = Some(dt);
        Ok(self)
    }

    /// Sets the sender filter.
    ///
    /// Only messages from this sender will be included.
    /// Matching is case-insensitive for ASCII characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use chatpack::core::filter::FilterConfig;
    ///
    /// // Matches "Alice", "alice", "ALICE"
    /// let config = FilterConfig::new().with_sender("Alice");
    /// ```
    #[must_use]
    pub fn with_sender(mut self, sender: impl Into<String>) -> Self {
        self.from = Some(sender.into());
        self
    }

    // Legacy method names for backwards compatibility

    /// Sets the start date filter. Alias for [`with_date_from`](Self::with_date_from).
    #[doc(hidden)]
    pub fn after_date(self, date_str: &str) -> Result<Self, ChatpackError> {
        self.with_date_from(date_str)
    }

    /// Sets the end date filter. Alias for [`with_date_to`](Self::with_date_to).
    #[doc(hidden)]
    pub fn before_date(self, date_str: &str) -> Result<Self, ChatpackError> {
        self.with_date_to(date_str)
    }

    /// Sets the sender filter. Alias for [`with_sender`](Self::with_sender).
    #[doc(hidden)]
    #[must_use]
    pub fn with_user(self, user: String) -> Self {
        self.with_sender(user)
    }

    /// Sets the start timestamp directly.
    ///
    /// Use this when you already have a parsed [`DateTime`].
    #[must_use]
    pub fn with_after(mut self, dt: DateTime<Utc>) -> Self {
        self.after = Some(dt);
        self
    }

    /// Sets the end timestamp directly.
    ///
    /// Use this when you already have a parsed [`DateTime`].
    #[must_use]
    pub fn with_before(mut self, dt: DateTime<Utc>) -> Self {
        self.before = Some(dt);
        self
    }

    /// Returns `true` if any filter is active.
    pub fn is_active(&self) -> bool {
        self.after.is_some() || self.before.is_some() || self.from.is_some()
    }

    /// Returns `true` if date filters are active.
    pub fn has_date_filter(&self) -> bool {
        self.after.is_some() || self.before.is_some()
    }

    /// Returns `true` if sender filter is active.
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

/// Filters a collection of messages based on the provided configuration.
///
/// Returns a new vector containing only messages that match all active filters.
/// If no filters are active, returns the original messages unchanged.
///
/// # Filter Behavior
///
/// - **Sender filter**: Case-insensitive ASCII matching
/// - **Date filters**: Messages without timestamps are excluded
/// - **Multiple filters**: Combined with AND logic
///
/// # Examples
///
/// ```
/// use chatpack::core::filter::{FilterConfig, apply_filters};
/// use chatpack::Message;
///
/// let messages = vec![
///     Message::new("Alice", "Hello"),
///     Message::new("Bob", "Hi"),
///     Message::new("Alice", "Goodbye"),
/// ];
///
/// // Filter by sender
/// let config = FilterConfig::new().with_sender("Alice");
/// let filtered = apply_filters(messages, &config);
///
/// assert_eq!(filtered.len(), 2);
/// assert!(filtered.iter().all(|m| m.sender() == "Alice"));
/// ```
///
/// # Performance
///
/// This function consumes the input vector. For streaming use cases,
/// apply filtering inline during iteration instead.
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
        assert!(matches!(result, Err(ChatpackError::InvalidDate { .. })));
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
