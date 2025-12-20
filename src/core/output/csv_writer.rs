//! CSV output writer.

use std::error::Error;
use std::fs::File;

use crate::core::models::{InternalMessage, OutputConfig};

/// Writes messages to CSV with semicolon delimiter.
///
/// # Format
/// - Delimiter: `;`
/// - Columns: Depends on `OutputConfig`
///   - Basic: `Sender`, `Content`
///   - With timestamps: `Timestamp`, `Sender`, `Content`
///   - With IDs: `ID`, `Sender`, `Content`
///   - With replies: `Sender`, `Content`, `ReplyTo`
///   - With edited: `Sender`, `Content`, `Edited`
/// - Encoding: UTF-8
pub fn write_csv(
    messages: &[InternalMessage],
    output_path: &str,
    config: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = csv::WriterBuilder::new().delimiter(b';').from_writer(file);

    // Build header dynamically
    let header = build_header(config);
    writer.write_record(&header)?;

    // Write each message
    for msg in messages {
        let record = build_record(msg, config);
        writer.write_record(&record)?;
    }

    writer.flush()?;
    Ok(())
}

/// Build CSV header based on output configuration.
fn build_header(config: &OutputConfig) -> Vec<&'static str> {
    let mut header = Vec::new();

    if config.include_ids {
        header.push("ID");
    }
    if config.include_timestamps {
        header.push("Timestamp");
    }

    header.push("Sender");
    header.push("Content");

    if config.include_replies {
        header.push("ReplyTo");
    }
    if config.include_edited {
        header.push("Edited");
    }

    header
}

/// Build CSV record for a single message.
fn build_record(msg: &InternalMessage, config: &OutputConfig) -> Vec<String> {
    let mut record = Vec::new();

    if config.include_ids {
        record.push(msg.id.map(|id| id.to_string()).unwrap_or_default());
    }
    if config.include_timestamps {
        record.push(
            msg.timestamp
                .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default(),
        );
    }

    record.push(msg.sender.clone());
    record.push(msg.content.clone());

    if config.include_replies {
        record.push(msg.reply_to.map(|id| id.to_string()).unwrap_or_default());
    }
    if config.include_edited {
        record.push(
            msg.edited
                .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default(),
        );
    }

    record
}
