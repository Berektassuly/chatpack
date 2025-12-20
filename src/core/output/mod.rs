//! Output format writers.
//!
//! This module provides writers for different output formats:
//! - [`write_csv`] - CSV with semicolon delimiter (best for LLMs)
//! - [`write_json`] - JSON array of messages
//! - [`write_jsonl`] - JSON Lines (one JSON per line, best for RAG)
//!
//! # Choosing a Format
//!
//! | Format | Use Case | Token Efficiency |
//! |--------|----------|-----------------|
//! | CSV | ChatGPT/Claude context | ⭐⭐⭐ Best (13x compression) |
//! | JSON | Structured data, APIs | ⭐ Good |
//! | JSONL | RAG pipelines, streaming | ⭐⭐ Better |
//!
//! # Example
//!
//! ```rust,no_run
//! use chatpack::core::output::{write_csv, write_json, write_jsonl};
//! use chatpack::core::models::{InternalMessage, OutputConfig};
//!
//! let messages = vec![
//!     InternalMessage::new("Alice", "Hello!"),
//!     InternalMessage::new("Bob", "Hi there!"),
//! ];
//!
//! let config = OutputConfig::new().with_timestamps();
//!
//! // Write to different formats
//! write_csv(&messages, "output.csv", &config).unwrap();
//! write_json(&messages, "output.json", &config).unwrap();
//! write_jsonl(&messages, "output.jsonl", &config).unwrap();
//! ```

mod csv_writer;
mod json_writer;
mod jsonl_writer;

pub use csv_writer::write_csv;
pub use json_writer::write_json;
pub use jsonl_writer::write_jsonl;
