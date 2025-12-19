//! Core processing logic.

pub mod filter;
pub mod models;
pub mod output;
pub mod processor;

pub use filter::{apply_filters, FilterConfig, FilterError};
pub use models::{InternalMessage, OutputConfig};
pub use output::write_csv;
pub use processor::{merge_consecutive, ProcessingStats};
