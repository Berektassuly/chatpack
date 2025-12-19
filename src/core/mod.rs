//! Core processing logic.

pub mod filter;
pub mod models;
pub mod output;
pub mod processor;

pub use filter::{FilterConfig, apply_filters};
pub use models::{InternalMessage, OutputConfig};
pub use output::{write_csv, write_json, write_jsonl};
pub use processor::{ProcessingStats, merge_consecutive};
