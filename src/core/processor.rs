use std::error::Error;
use std::fs::File;

use super::models::InternalMessage;

/// Merges consecutive messages from the same sender into single entries.
/// This reduces token count when feeding to LLMs.
///
/// # Example
/// Input:  [("Alice", "Hi"), ("Alice", "How are you?"), ("Bob", "Fine")]
/// Output: [("Alice", "Hi\nHow are you?"), ("Bob", "Fine")]
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

/// Writes messages to CSV with semicolon delimiter.
///
/// # Format
/// - Delimiter: `;`
/// - Columns: `Sender`, `Content`
/// - Encoding: UTF-8
pub fn write_csv(messages: &[InternalMessage], output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(file);

    writer.write_record(["Sender", "Content"])?;

    for msg in messages {
        writer.write_record([&msg.sender, &msg.content])?;
    }

    writer.flush()?;
    Ok(())
}

/// Statistics about the processing result
#[derive(Debug)]
pub struct ProcessingStats {
    pub original_count: usize,
    pub merged_count: usize,
}

impl ProcessingStats {
    pub fn compression_ratio(&self) -> f64 {
        if self.original_count == 0 {
            return 0.0;
        }
        (1.0 - (self.merged_count as f64 / self.original_count as f64)) * 100.0
    }
}

/// Full processing pipeline: merge + write CSV
pub fn process_and_write(
    messages: Vec<InternalMessage>,
    output_path: &str,
) -> Result<ProcessingStats, Box<dyn Error>> {
    let original_count = messages.len();
    let merged = merge_consecutive(messages);
    let merged_count = merged.len();

    write_csv(&merged, output_path)?;

    Ok(ProcessingStats {
        original_count,
        merged_count,
    })
}
