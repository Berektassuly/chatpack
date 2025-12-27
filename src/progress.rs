//! Progress reporting types for long-running operations.
//!
//! This module provides a callback-based progress reporting mechanism
//! for library users who want push-based progress updates rather than
//! polling the iterator.
//!
//! # Example
//!
//! ```rust
//! use chatpack::progress::{Progress, ProgressCallback};
//! use std::sync::Arc;
//!
//! // Create a progress callback
//! let callback: ProgressCallback = Arc::new(|progress| {
//!     if let Some(pct) = progress.percentage() {
//!         println!("Progress: {:.1}%", pct);
//!     }
//! });
//!
//! // Use the callback in your processing loop
//! let total_bytes = 1000u64;
//! for i in 0..10usize {
//!     // Simulate processing
//!     let bytes_processed = ((i + 1) * 100) as u64;
//!     callback(Progress::new(bytes_processed, Some(total_bytes), i + 1));
//! }
//! ```
//!
//! For streaming parsers, the iterator returned by `StreamingParser::stream()`
//! provides progress tracking via its `MessageIterator` methods:
//! - `progress()` - Returns percentage (0.0-100.0) if known
//! - `bytes_processed()` - Returns bytes read so far
//! - `total_bytes()` - Returns total file size if known

use std::sync::Arc;

/// Progress information for long-running operations.
///
/// Contains details about the current progress of an operation,
/// including bytes processed, total bytes (if known), and items processed.
#[derive(Debug, Clone, Copy, Default)]
pub struct Progress {
    /// Number of bytes processed so far.
    pub bytes_processed: u64,

    /// Total bytes to process, if known.
    pub total_bytes: Option<u64>,

    /// Number of items (e.g., messages) processed so far.
    pub items_processed: usize,

    /// Total items to process, if known.
    pub total_items: Option<usize>,
}

impl Progress {
    /// Creates a new progress instance.
    pub fn new(bytes_processed: u64, total_bytes: Option<u64>, items_processed: usize) -> Self {
        Self {
            bytes_processed,
            total_bytes,
            items_processed,
            total_items: None,
        }
    }

    /// Creates a progress instance with total items.
    #[must_use]
    pub fn with_items(mut self, total_items: usize) -> Self {
        self.total_items = Some(total_items);
        self
    }

    /// Returns the progress as a percentage (0.0 - 100.0).
    ///
    /// Returns `None` if total bytes is not known.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::progress::Progress;
    ///
    /// let progress = Progress::new(500, Some(1000), 50);
    /// assert_eq!(progress.percentage(), Some(50.0));
    ///
    /// let unknown = Progress::new(500, None, 50);
    /// assert_eq!(unknown.percentage(), None);
    /// ```
    pub fn percentage(&self) -> Option<f64> {
        self.total_bytes.map(|total| {
            if total == 0 {
                100.0
            } else {
                (self.bytes_processed as f64 / total as f64) * 100.0
            }
        })
    }

    /// Returns the items percentage (0.0 - 100.0).
    ///
    /// Returns `None` if total items is not known.
    pub fn items_percentage(&self) -> Option<f64> {
        self.total_items.map(|total| {
            if total == 0 {
                100.0
            } else {
                (self.items_processed as f64 / total as f64) * 100.0
            }
        })
    }

    /// Returns whether the operation is complete.
    ///
    /// An operation is considered complete when bytes_processed equals total_bytes
    /// (if known).
    pub fn is_complete(&self) -> bool {
        self.total_bytes
            .map(|total| self.bytes_processed >= total)
            .unwrap_or(false)
    }

    /// Returns the remaining bytes to process.
    ///
    /// Returns `None` if total bytes is not known.
    pub fn remaining_bytes(&self) -> Option<u64> {
        self.total_bytes.map(|total| total.saturating_sub(self.bytes_processed))
    }
}

/// Callback type for receiving progress updates.
///
/// This is a thread-safe callback that receives [`Progress`] updates
/// during long-running operations.
///
/// # Example
///
/// ```rust
/// use chatpack::progress::{Progress, ProgressCallback};
/// use std::sync::Arc;
///
/// let callback: ProgressCallback = Arc::new(|progress| {
///     println!("Processed {} bytes", progress.bytes_processed);
/// });
///
/// // Call the callback
/// callback(Progress::new(1000, Some(2000), 10));
/// ```
pub type ProgressCallback = Arc<dyn Fn(Progress) + Send + Sync>;

/// Creates a no-op progress callback.
///
/// This is useful when you don't need progress updates but an API
/// requires a callback.
///
/// # Example
///
/// ```rust
/// use chatpack::progress::no_progress;
///
/// let callback = no_progress();
/// callback(chatpack::progress::Progress::default()); // Does nothing
/// ```
pub fn no_progress() -> ProgressCallback {
    Arc::new(|_| {})
}

/// Creates a progress callback that prints to stderr.
///
/// This is useful for CLI applications that want simple progress output.
///
/// # Example
///
/// ```rust
/// use chatpack::progress::stderr_progress;
///
/// let callback = stderr_progress();
/// // Will print "Progress: 50.0%" to stderr
/// callback(chatpack::progress::Progress::new(500, Some(1000), 0));
/// ```
pub fn stderr_progress() -> ProgressCallback {
    Arc::new(|progress| {
        if let Some(pct) = progress.percentage() {
            eprintln!("Progress: {:.1}%", pct);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_percentage() {
        let progress = Progress::new(500, Some(1000), 50);
        assert_eq!(progress.percentage(), Some(50.0));
    }

    #[test]
    fn test_progress_percentage_unknown_total() {
        let progress = Progress::new(500, None, 50);
        assert_eq!(progress.percentage(), None);
    }

    #[test]
    fn test_progress_percentage_zero_total() {
        let progress = Progress::new(0, Some(0), 0);
        assert_eq!(progress.percentage(), Some(100.0));
    }

    #[test]
    fn test_progress_is_complete() {
        let complete = Progress::new(1000, Some(1000), 100);
        assert!(complete.is_complete());

        let incomplete = Progress::new(500, Some(1000), 50);
        assert!(!incomplete.is_complete());

        let unknown = Progress::new(500, None, 50);
        assert!(!unknown.is_complete());
    }

    #[test]
    fn test_progress_remaining_bytes() {
        let progress = Progress::new(300, Some(1000), 30);
        assert_eq!(progress.remaining_bytes(), Some(700));

        let unknown = Progress::new(300, None, 30);
        assert_eq!(unknown.remaining_bytes(), None);
    }

    #[test]
    fn test_progress_with_items() {
        let progress = Progress::new(500, Some(1000), 50).with_items(100);
        assert_eq!(progress.total_items, Some(100));
        assert_eq!(progress.items_percentage(), Some(50.0));
    }

    #[test]
    fn test_no_progress_callback() {
        let callback = no_progress();
        callback(Progress::default()); // Should not panic
    }

    #[test]
    fn test_progress_callback_type() {
        use std::sync::atomic::{AtomicU64, Ordering};

        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = counter.clone();

        let callback: ProgressCallback = Arc::new(move |progress| {
            counter_clone.store(progress.bytes_processed, Ordering::SeqCst);
        });

        callback(Progress::new(42, None, 0));
        assert_eq!(counter.load(Ordering::SeqCst), 42);
    }
}
