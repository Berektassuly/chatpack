//! Core processing logic for chatpack.
//!
//! This module contains:
//! - [`models`] - Data structures for messages and configuration
//! - [`filter`] - Message filtering by date and sender
//! - [`processor`] - Message merging and statistics
//! - [`output`] - Format writers (CSV, JSON, JSONL)
//!
//! # Quick Start
//!
//! ```rust
//! # #[cfg(all(feature = "csv-output", feature = "json-output"))]
//! # fn example() {
//! use chatpack::core::{
//!     InternalMessage, OutputConfig, FilterConfig,
//!     merge_consecutive, apply_filters,
//!     write_csv, write_json, write_jsonl,
//! };
//! # }
//! ```

pub mod filter;
pub mod models;
pub mod output;
pub mod processor;

// Re-export main types for convenience
pub use filter::{FilterConfig, apply_filters};
#[allow(deprecated)]
pub use models::InternalMessage;
pub use models::OutputConfig;

// Re-export Message from the new location
pub use crate::Message;

// Conditionally re-export output writers
#[cfg(feature = "csv-output")]
pub use output::{to_csv, write_csv};
#[cfg(feature = "json-output")]
pub use output::{to_json, to_jsonl, write_json, write_jsonl};

pub use processor::{ProcessingStats, merge_consecutive};
