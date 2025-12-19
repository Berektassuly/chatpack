//! WhatsApp TXT export parser.

use std::error::Error;

use super::ChatParser;
use crate::core::InternalMessage;

/// Parser for WhatsApp TXT exports.
///
/// WhatsApp exports chats as plain text with the following format:
/// ```
/// [DD/MM/YYYY, HH:MM:SS] Sender Name: Message text
/// [DD/MM/YYYY, HH:MM:SS] Sender Name: Another message
/// ```
///
/// # TODO
/// - Handle multiline messages
/// - Handle media placeholders (e.g., "<Media omitted>")
/// - Handle system messages (e.g., "Sender joined using this group's invite link")
pub struct WhatsAppParser;

impl WhatsAppParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WhatsAppParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatParser for WhatsAppParser {
    fn name(&self) -> &'static str {
        "WhatsApp"
    }

    fn parse(&self, file_path: &str) -> Result<Vec<InternalMessage>, Box<dyn Error>> {
        Err(format!("WhatsApp parser not yet implemented. File: {}", file_path).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_name() {
        let parser = WhatsAppParser::new();
        assert_eq!(parser.name(), "WhatsApp");
    }
}
