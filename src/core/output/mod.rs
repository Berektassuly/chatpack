//! Output format writers.
//!
//! This module provides writers for different output formats:
//! - [`write_csv`] / [`to_csv`] - CSV with semicolon delimiter (best for LLMs) - requires `csv-output` feature
//! - [`write_json`] / [`to_json`] - JSON array of messages - requires `json-output` feature
//! - [`write_jsonl`] / [`to_jsonl`] - JSON Lines (one JSON per line, best for RAG) - requires `json-output` feature
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
//! # #[cfg(all(feature = "csv-output", feature = "json-output"))]
//! # fn main() -> chatpack::Result<()> {
//! use chatpack::core::output::{write_csv, write_json, write_jsonl, to_csv};
//! use chatpack::core::models::OutputConfig;
//! use chatpack::Message;
//!
//! let messages = vec![
//!     Message::new("Alice", "Hello!"),
//!     Message::new("Bob", "Hi there!"),
//! ];
//!
//! let config = OutputConfig::new().with_timestamps();
//!
//! // Write to files
//! write_csv(&messages, "output.csv", &config)?;
//! write_json(&messages, "output.json", &config)?;
//! write_jsonl(&messages, "output.jsonl", &config)?;
//!
//! // Or get as strings (useful for WASM)
//! let csv_string = to_csv(&messages, &config)?;
//! # Ok(())
//! # }
//! # #[cfg(not(all(feature = "csv-output", feature = "json-output")))]
//! # fn main() {}
//! ```

#[cfg(feature = "csv-output")]
mod csv_writer;
#[cfg(feature = "json-output")]
mod json_writer;
#[cfg(feature = "json-output")]
mod jsonl_writer;

#[cfg(feature = "csv-output")]
pub use csv_writer::{to_csv, write_csv};
#[cfg(feature = "json-output")]
pub use json_writer::{to_json, write_json};
#[cfg(feature = "json-output")]
pub use jsonl_writer::{to_jsonl, write_jsonl};
