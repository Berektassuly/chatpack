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
//! use chatpack::core::{
//!     InternalMessage, OutputConfig, FilterConfig,
//!     merge_consecutive, apply_filters,
//!     write_csv, write_json, write_jsonl,
//! };
//! ```

pub mod filter;
pub mod models;
pub mod output;
pub mod processor;

// Re-export main types for convenience
pub use filter::{FilterConfig, FilterError, apply_filters};
pub use models::{InternalMessage, OutputConfig};
pub use output::{write_csv, write_json, write_jsonl};
pub use processor::{ProcessingStats, merge_consecutive};
