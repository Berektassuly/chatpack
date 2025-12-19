//! CSV output writer.

use std::error::Error;
use std::fs::File;

use crate::core::models::{InternalMessage, OutputConfig};

/// Writes messages to CSV with semicolon delimiter.
///
/// # Format
/// - Delimiter: `;`
/// - Columns: Depends on OutputConfig
///   - Basic: `Sender`, `Content`
///   - With timestamps: `Timestamp`, `Sender`, `Content`
///   - With IDs: `ID`, `Sender`, `Content`
///   - With replies: `Sender`, `Content`, `ReplyTo`
/// - Encoding: UTF-8
pub fn write_csv(
    messages: &[InternalMessage],
    output_path: &str,
    config: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(file);

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

    record
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_csv_basic() {
        let messages = vec![
            InternalMessage::new("Alice", "Hello"),
            InternalMessage::new("Bob", "Hi there"),
        ];

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let config = OutputConfig::new();
        write_csv(&messages, path, &config).unwrap();

        let mut content = String::new();
        std::fs::File::open(path)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert!(content.contains("Sender;Content"));
        assert!(content.contains("Alice;Hello"));
        assert!(content.contains("Bob;Hi there"));
    }

    #[test]
    fn test_write_csv_with_timestamps() {
        use chrono::TimeZone;

        let ts = chrono::Utc.with_ymd_and_hms(2024, 6, 15, 12, 30, 0).unwrap();
        let msg = InternalMessage::new("Alice", "Hello").timestamp(ts);

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let config = OutputConfig::new().with_timestamps();
        write_csv(&[msg], path, &config).unwrap();

        let mut content = String::new();
        std::fs::File::open(path)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert!(content.contains("Timestamp;Sender;Content"));
        assert!(content.contains("2024-06-15 12:30:00;Alice;Hello"));
    }
}
