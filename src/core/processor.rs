//! Message processing utilities.

use super::models::InternalMessage;

/// Merges consecutive messages from the same sender into single entries.
/// This reduces token count when feeding to LLMs.
///
/// # Example
/// Input:  [("Alice", "Hi"), ("Alice", "How are you?"), ("Bob", "Fine")]
/// Output: [("Alice", "Hi\nHow are you?"), ("Bob", "Fine")]
///
/// # Note
/// When merging, the first message's metadata (timestamp, id, reply_to, edited)
/// is preserved. This represents the start of the conversation block.
pub fn merge_consecutive(messages: Vec<InternalMessage>) -> Vec<InternalMessage> {
    let mut merged: Vec<InternalMessage> = Vec::new();

    for msg in messages {
        match merged.last_mut() {
            Some(last) if last.sender == msg.sender => {
                last.content.push('\n');
                last.content.push_str(&msg.content);
            }
            _ => {
                merged.push(msg);
            }
        }
    }

    merged
}

/// Statistics about the processing result.
#[derive(Debug)]
pub struct ProcessingStats {
    pub original_count: usize,
    pub merged_count: usize,
    pub filtered_count: Option<usize>,
}

impl ProcessingStats {
    pub fn new(original: usize, merged: usize) -> Self {
        Self {
            original_count: original,
            merged_count: merged,
            filtered_count: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_filtered(mut self, filtered: usize) -> Self {
        self.filtered_count = Some(filtered);
        self
    }

    /// Calculate compression ratio as percentage.
    pub fn compression_ratio(&self) -> f64 {
        let base = self.filtered_count.unwrap_or(self.original_count);
        if base == 0 {
            return 0.0;
        }
        (1.0 - (self.merged_count as f64 / base as f64)) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_consecutive() {
        let messages = vec![
            InternalMessage::new("Alice", "Hi"),
            InternalMessage::new("Alice", "How are you?"),
            InternalMessage::new("Bob", "Fine"),
            InternalMessage::new("Bob", "Thanks"),
            InternalMessage::new("Alice", "Great!"),
        ];

        let merged = merge_consecutive(messages);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].sender, "Alice");
        assert_eq!(merged[0].content, "Hi\nHow are you?");
        assert_eq!(merged[1].sender, "Bob");
        assert_eq!(merged[1].content, "Fine\nThanks");
        assert_eq!(merged[2].sender, "Alice");
        assert_eq!(merged[2].content, "Great!");
    }

    #[test]
    fn test_merge_empty() {
        let messages: Vec<InternalMessage> = vec![];
        let merged = merge_consecutive(messages);
        assert!(merged.is_empty());
    }

    #[test]
    fn test_merge_single() {
        let messages = vec![InternalMessage::new("Alice", "Hi")];
        let merged = merge_consecutive(messages);
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn test_compression_ratio() {
        let stats = ProcessingStats::new(100, 50);
        assert!((stats.compression_ratio() - 50.0).abs() < 0.01);
    }
}
