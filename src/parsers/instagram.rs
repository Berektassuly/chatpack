//! Instagram JSON export parser.

use std::error::Error;

use super::ChatParser;
use crate::core::InternalMessage;

/// Parser for Instagram JSON exports.
///
/// Instagram exports messages as JSON (from "Download Your Data" feature).
/// The structure typically looks like:
/// ```json
/// {
///   "participants": [{"name": "User1"}, {"name": "User2"}],
///   "messages": [
///     {
///       "sender_name": "User1",
///       "timestamp_ms": 1234567890000,
///       "content": "Hello!",
///       "type": "Generic"
///     }
///   ]
/// }
/// ```
pub struct InstagramParser;

impl InstagramParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InstagramParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatParser for InstagramParser {
    fn name(&self) -> &'static str {
        "Instagram"
    }

    fn parse(&self, file_path: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>> {
        Err(format!("Instagram parser not yet implemented. File: {}", file_path).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_name() {
        let parser = InstagramParser::new();
        assert_eq!(parser.name(), "Instagram");
    }
}
