//! Output format types for the chatpack library.
//!
//! This module provides library-first format types that don't depend on CLI
//! frameworks. These types are suitable for use in library code, WASM builds,
//! and other contexts where CLI dependencies are not desired.
//!
//! # Example
//!
//! ```rust
//! # #[cfg(all(feature = "csv-output", feature = "json-output"))]
//! # fn example() -> chatpack::Result<()> {
//! use chatpack::format::{OutputFormat, write_to_format};
//! use chatpack::core::models::OutputConfig;
//! use chatpack::Message;
//!
//! let messages = vec![
//!     Message::new("Alice", "Hello!"),
//!     Message::new("Bob", "Hi there!"),
//! ];
//!
//! // Write using format enum
//! write_to_format(&messages, "output.csv", OutputFormat::Csv, &OutputConfig::new())?;
//!
//! // Or use format detection from extension
//! let format = OutputFormat::from_path("output.jsonl")?;
//! assert_eq!(format, OutputFormat::Jsonl);
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};

use crate::Message;
use crate::core::models::OutputConfig;
use crate::error::ChatpackError;

/// Output format for chat exports.
///
/// Different formats serve different purposes:
/// - [`Csv`](OutputFormat::Csv) - Best for LLM context (13x token compression)
/// - [`Json`](OutputFormat::Json) - Structured array, good for APIs
/// - [`Jsonl`](OutputFormat::Jsonl) - One JSON per line, ideal for RAG/ML pipelines
///
/// # Example
///
/// ```rust
/// use chatpack::format::OutputFormat;
/// use std::str::FromStr;
///
/// let format = OutputFormat::from_str("jsonl").unwrap();
/// assert_eq!(format, OutputFormat::Jsonl);
/// assert_eq!(format.extension(), "jsonl");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OutputFormat {
    /// CSV with semicolon delimiter (default, best for LLMs)
    ///
    /// Produces the most token-efficient output, with up to 13x compression
    /// compared to raw chat exports.
    #[default]
    Csv,

    /// JSON array of messages
    ///
    /// Standard JSON format, suitable for APIs and structured processing.
    Json,

    /// JSON Lines - one JSON object per line
    ///
    /// Ideal for streaming, RAG pipelines, and ML applications.
    /// Also known as NDJSON.
    Jsonl,
}

impl OutputFormat {
    /// Returns the file extension for this format (without dot).
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::format::OutputFormat;
    ///
    /// assert_eq!(OutputFormat::Csv.extension(), "csv");
    /// assert_eq!(OutputFormat::Json.extension(), "json");
    /// assert_eq!(OutputFormat::Jsonl.extension(), "jsonl");
    /// ```
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Csv => "csv",
            OutputFormat::Json => "json",
            OutputFormat::Jsonl => "jsonl",
        }
    }

    /// Returns all supported format names.
    pub fn all_names() -> &'static [&'static str] {
        &["csv", "json", "jsonl", "ndjson"]
    }

    /// Returns all available formats.
    pub fn all() -> &'static [OutputFormat] {
        &[OutputFormat::Csv, OutputFormat::Json, OutputFormat::Jsonl]
    }

    /// Returns the MIME type for this format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::format::OutputFormat;
    ///
    /// assert_eq!(OutputFormat::Json.mime_type(), "application/json");
    /// ```
    pub fn mime_type(&self) -> &'static str {
        match self {
            OutputFormat::Csv => "text/csv",
            OutputFormat::Json => "application/json",
            OutputFormat::Jsonl => "application/x-ndjson",
        }
    }

    /// Detects format from a file path based on extension.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chatpack::format::OutputFormat;
    ///
    /// let format = OutputFormat::from_path("output.jsonl").unwrap();
    /// assert_eq!(format, OutputFormat::Jsonl);
    /// ```
    pub fn from_path(path: &str) -> Result<Self, ChatpackError> {
        let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();

        match ext.as_str() {
            "csv" => Ok(OutputFormat::Csv),
            "json" => Ok(OutputFormat::Json),
            "jsonl" | "ndjson" => Ok(OutputFormat::Jsonl),
            _ => Err(ChatpackError::InvalidFormat {
                format: "output",
                message: format!(
                    "Unknown file extension: '.{}'. Expected one of: csv, json, jsonl",
                    ext
                ),
            }),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Csv => write!(f, "CSV"),
            OutputFormat::Json => write!(f, "JSON"),
            OutputFormat::Jsonl => write!(f, "JSONL"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(OutputFormat::Csv),
            "json" => Ok(OutputFormat::Json),
            "jsonl" | "ndjson" => Ok(OutputFormat::Jsonl),
            _ => Err(format!(
                "Unknown format: '{}'. Expected one of: {}",
                s,
                OutputFormat::all_names().join(", ")
            )),
        }
    }
}

/// Writes messages to a file in the specified format.
///
/// This is a convenience function that selects the appropriate writer
/// based on the format enum.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(all(feature = "csv-output", feature = "json-output"))]
/// # fn example() -> chatpack::Result<()> {
/// use chatpack::format::{OutputFormat, write_to_format};
/// use chatpack::core::models::OutputConfig;
/// use chatpack::Message;
///
/// let messages = vec![Message::new("Alice", "Hello!")];
/// let config = OutputConfig::new().with_timestamps();
///
/// write_to_format(&messages, "output.csv", OutputFormat::Csv, &config)?;
/// write_to_format(&messages, "output.json", OutputFormat::Json, &config)?;
/// write_to_format(&messages, "output.jsonl", OutputFormat::Jsonl, &config)?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The required feature for the format is not enabled
/// - The file cannot be written
#[allow(unused_variables)]
pub fn write_to_format(
    messages: &[Message],
    path: &str,
    format: OutputFormat,
    config: &OutputConfig,
) -> Result<(), ChatpackError> {
    match format {
        #[cfg(feature = "csv-output")]
        OutputFormat::Csv => crate::core::output::write_csv(messages, path, config),
        #[cfg(feature = "json-output")]
        OutputFormat::Json => crate::core::output::write_json(messages, path, config),
        #[cfg(feature = "json-output")]
        OutputFormat::Jsonl => crate::core::output::write_jsonl(messages, path, config),
        #[allow(unreachable_patterns)]
        _ => Err(ChatpackError::InvalidFormat {
            format: "output",
            message: format!(
                "Output format {:?} requires the '{}' feature to be enabled",
                format,
                match format {
                    OutputFormat::Csv => "csv-output",
                    OutputFormat::Json | OutputFormat::Jsonl => "json-output",
                }
            ),
        }),
    }
}

/// Converts messages to a string in the specified format.
///
/// This is useful for WASM environments or when you need the output
/// as a string rather than writing to a file.
///
/// # Example
///
/// ```rust
/// # #[cfg(all(feature = "csv-output", feature = "json-output"))]
/// # fn example() -> chatpack::Result<()> {
/// use chatpack::format::{OutputFormat, to_format_string};
/// use chatpack::core::models::OutputConfig;
/// use chatpack::Message;
///
/// let messages = vec![Message::new("Alice", "Hello!")];
/// let csv = to_format_string(&messages, OutputFormat::Csv, &OutputConfig::new())?;
/// # Ok(())
/// # }
/// ```
#[allow(unused_variables)]
pub fn to_format_string(
    messages: &[Message],
    format: OutputFormat,
    config: &OutputConfig,
) -> Result<String, ChatpackError> {
    match format {
        #[cfg(feature = "csv-output")]
        OutputFormat::Csv => crate::core::output::to_csv(messages, config),
        #[cfg(feature = "json-output")]
        OutputFormat::Json => crate::core::output::to_json(messages, config),
        #[cfg(feature = "json-output")]
        OutputFormat::Jsonl => crate::core::output::to_jsonl(messages, config),
        #[allow(unreachable_patterns)]
        _ => Err(ChatpackError::InvalidFormat {
            format: "output",
            message: format!(
                "Output format {:?} requires the '{}' feature to be enabled",
                format,
                match format {
                    OutputFormat::Csv => "csv-output",
                    OutputFormat::Json | OutputFormat::Jsonl => "json-output",
                }
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // =========================================================================
    // FromStr tests
    // =========================================================================

    #[test]
    fn test_format_from_str() {
        assert_eq!(OutputFormat::from_str("csv").unwrap(), OutputFormat::Csv);
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(
            OutputFormat::from_str("jsonl").unwrap(),
            OutputFormat::Jsonl
        );
        assert_eq!(
            OutputFormat::from_str("ndjson").unwrap(),
            OutputFormat::Jsonl
        );
        assert_eq!(OutputFormat::from_str("CSV").unwrap(), OutputFormat::Csv);
        assert_eq!(OutputFormat::from_str("JSON").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("JSONL").unwrap(), OutputFormat::Jsonl);
        assert_eq!(OutputFormat::from_str("NDJSON").unwrap(), OutputFormat::Jsonl);
    }

    #[test]
    fn test_format_from_str_errors() {
        let err = OutputFormat::from_str("unknown").unwrap_err();
        assert!(err.contains("Unknown format"));
        assert!(err.contains("unknown"));

        let err = OutputFormat::from_str("").unwrap_err();
        assert!(err.contains("Unknown format"));

        let err = OutputFormat::from_str("xml").unwrap_err();
        assert!(err.contains("Unknown format"));
    }

    // =========================================================================
    // Display tests
    // =========================================================================

    #[test]
    fn test_format_display() {
        assert_eq!(OutputFormat::Csv.to_string(), "CSV");
        assert_eq!(OutputFormat::Json.to_string(), "JSON");
        assert_eq!(OutputFormat::Jsonl.to_string(), "JSONL");
    }

    // =========================================================================
    // Extension tests
    // =========================================================================

    #[test]
    fn test_format_extension() {
        assert_eq!(OutputFormat::Csv.extension(), "csv");
        assert_eq!(OutputFormat::Json.extension(), "json");
        assert_eq!(OutputFormat::Jsonl.extension(), "jsonl");
    }

    // =========================================================================
    // MIME type tests
    // =========================================================================

    #[test]
    fn test_format_mime_type() {
        assert_eq!(OutputFormat::Csv.mime_type(), "text/csv");
        assert_eq!(OutputFormat::Json.mime_type(), "application/json");
        assert_eq!(OutputFormat::Jsonl.mime_type(), "application/x-ndjson");
    }

    // =========================================================================
    // from_path tests
    // =========================================================================

    #[test]
    fn test_format_from_path() {
        assert_eq!(
            OutputFormat::from_path("output.csv").unwrap(),
            OutputFormat::Csv
        );
        assert_eq!(
            OutputFormat::from_path("output.json").unwrap(),
            OutputFormat::Json
        );
        assert_eq!(
            OutputFormat::from_path("output.jsonl").unwrap(),
            OutputFormat::Jsonl
        );
        assert_eq!(
            OutputFormat::from_path("output.ndjson").unwrap(),
            OutputFormat::Jsonl
        );
        assert_eq!(
            OutputFormat::from_path("/path/to/file.JSON").unwrap(),
            OutputFormat::Json
        );
        assert_eq!(
            OutputFormat::from_path("file.CSV").unwrap(),
            OutputFormat::Csv
        );
    }

    #[test]
    fn test_format_from_path_errors() {
        let err = OutputFormat::from_path("output.txt");
        assert!(err.is_err());

        let err = OutputFormat::from_path("noextension");
        assert!(err.is_err());

        let err = OutputFormat::from_path("file.xml");
        assert!(err.is_err());
    }

    // =========================================================================
    // all() and all_names() tests
    // =========================================================================

    #[test]
    fn test_format_all() {
        let all = OutputFormat::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&OutputFormat::Csv));
        assert!(all.contains(&OutputFormat::Json));
        assert!(all.contains(&OutputFormat::Jsonl));
    }

    #[test]
    fn test_format_all_names() {
        let names = OutputFormat::all_names();
        assert!(names.contains(&"csv"));
        assert!(names.contains(&"json"));
        assert!(names.contains(&"jsonl"));
        assert!(names.contains(&"ndjson"));
    }

    // =========================================================================
    // Default trait tests
    // =========================================================================

    #[test]
    fn test_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Csv);
    }

    // =========================================================================
    // Serde tests
    // =========================================================================

    #[test]
    fn test_format_serde() {
        let format = OutputFormat::Jsonl;
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, "\"jsonl\"");

        let parsed: OutputFormat = serde_json::from_str("\"csv\"").unwrap();
        assert_eq!(parsed, OutputFormat::Csv);
    }

    #[test]
    fn test_format_serde_all_variants() {
        for format in OutputFormat::all() {
            let json = serde_json::to_string(format).expect("serialize failed");
            let parsed: OutputFormat = serde_json::from_str(&json).expect("deserialize failed");
            assert_eq!(parsed, *format);
        }
    }

    // =========================================================================
    // Traits tests (Clone, Copy, Eq, Hash)
    // =========================================================================

    #[test]
    fn test_format_clone_copy() {
        let f1 = OutputFormat::Csv;
        let f2 = f1; // Copy
        let f3 = f1.clone();
        assert_eq!(f1, f2);
        assert_eq!(f1, f3);
    }

    #[test]
    fn test_format_eq_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(OutputFormat::Csv);
        set.insert(OutputFormat::Json);
        set.insert(OutputFormat::Csv); // Duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_format_debug() {
        let debug = format!("{:?}", OutputFormat::Csv);
        assert!(debug.contains("Csv"));
    }

    // =========================================================================
    // to_format_string tests
    // =========================================================================

    #[cfg(all(feature = "csv-output", feature = "json-output"))]
    #[test]
    fn test_to_format_string() {
        let messages = vec![Message::new("Alice", "Hello!")];
        let config = OutputConfig::new();

        let csv = to_format_string(&messages, OutputFormat::Csv, &config).expect("csv failed");
        assert!(csv.contains("Alice"));
        assert!(csv.contains("Hello"));

        let json = to_format_string(&messages, OutputFormat::Json, &config).expect("json failed");
        assert!(json.contains("Alice"));
        assert!(json.contains("Hello"));

        let jsonl =
            to_format_string(&messages, OutputFormat::Jsonl, &config).expect("jsonl failed");
        assert!(jsonl.contains("Alice"));
        assert!(jsonl.contains("Hello"));
    }
}
