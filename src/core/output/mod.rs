//! Export messages to CSV, JSON, and JSONL formats.
//!
//! This module provides format writers optimized for different use cases.
//! Each format has both file-writing and string-generating variants.
//!
//! # Format Comparison
//!
//! | Format | Function | Feature | Best For |
//! |--------|----------|---------|----------|
//! | CSV | [`write_csv`] / [`to_csv`] | `csv-output` | LLM context (13x compression) |
//! | JSON | [`write_json`] / [`to_json`] | `json-output` | APIs, structured data |
//! | JSONL | [`write_jsonl`] / [`to_jsonl`] | `json-output` | RAG pipelines, streaming |
//!
//! # Examples
//!
//! ## Write to Files
//!
//! ```no_run
//! # #[cfg(all(feature = "csv-output", feature = "json-output"))]
//! # fn main() -> chatpack::Result<()> {
//! use chatpack::prelude::*;
//!
//! let messages = vec![
//!     Message::new("Alice", "Hello!"),
//!     Message::new("Bob", "Hi there!"),
//! ];
//! let config = OutputConfig::new().with_timestamps();
//!
//! write_csv(&messages, "output.csv", &config)?;
//! write_json(&messages, "output.json", &config)?;
//! write_jsonl(&messages, "output.jsonl", &config)?;
//! # Ok(())
//! # }
//! # #[cfg(not(all(feature = "csv-output", feature = "json-output")))]
//! # fn main() {}
//! ```
//!
//! ## Generate Strings (WASM-friendly)
//!
//! ```
//! # #[cfg(feature = "csv-output")]
//! # fn main() -> chatpack::Result<()> {
//! use chatpack::prelude::*;
//!
//! let messages = vec![Message::new("Alice", "Hello!")];
//! let csv = to_csv(&messages, &OutputConfig::new())?;
//!
//! assert!(csv.contains("Alice"));
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "csv-output"))]
//! # fn main() {}
//! ```
//!
//! # Feature Flags
//!
//! - `csv-output`: Enables CSV functions ([`write_csv`], [`to_csv`])
//! - `json-output`: Enables JSON functions ([`write_json`], [`to_json`], [`write_jsonl`], [`to_jsonl`])

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
