//! Additional tests for CLI module to improve coverage

use chatpack::cli::{Source, OutputFormat};
use std::str::FromStr;

#[test]
fn test_source_from_str_all_variants() {
    // Standard names
    assert!(Source::from_str("telegram").is_ok());
    assert!(Source::from_str("whatsapp").is_ok());
    assert!(Source::from_str("instagram").is_ok());
    
    // Aliases
    assert!(Source::from_str("tg").is_ok());
    assert!(Source::from_str("wa").is_ok());
    assert!(Source::from_str("ig").is_ok());
    
    // Case variations
    assert!(Source::from_str("TELEGRAM").is_ok());
    assert!(Source::from_str("Telegram").is_ok());
    assert!(Source::from_str("TG").is_ok());
    assert!(Source::from_str("WA").is_ok());
    assert!(Source::from_str("IG").is_ok());
}

#[test]
fn test_source_from_str_errors() {
    assert!(Source::from_str("").is_err());
    assert!(Source::from_str("unknown").is_err());
    assert!(Source::from_str("signal").is_err());
    assert!(Source::from_str("discord").is_err());
}

#[test]
fn test_output_format_from_str_all_variants() {
    assert!(OutputFormat::from_str("json").is_ok());
    assert!(OutputFormat::from_str("jsonl").is_ok());
    assert!(OutputFormat::from_str("csv").is_ok());
    
    // Case variations
    assert!(OutputFormat::from_str("JSON").is_ok());
    assert!(OutputFormat::from_str("JSONL").is_ok());
    assert!(OutputFormat::from_str("CSV").is_ok());
}

#[test]
fn test_output_format_from_str_errors() {
    assert!(OutputFormat::from_str("").is_err());
    assert!(OutputFormat::from_str("xml").is_err());
    assert!(OutputFormat::from_str("yaml").is_err());
    assert!(OutputFormat::from_str("txt").is_err());
}

#[test]
fn test_source_equality() {
    assert_eq!(Source::Telegram, Source::Telegram);
    assert_ne!(Source::Telegram, Source::WhatsApp);
    assert_ne!(Source::WhatsApp, Source::Instagram);
}

#[test]
fn test_output_format_equality() {
    assert_eq!(OutputFormat::Json, OutputFormat::Json);
    assert_ne!(OutputFormat::Json, OutputFormat::Jsonl);
    assert_ne!(OutputFormat::Jsonl, OutputFormat::Csv);
}

#[test]
fn test_source_copy() {
    let source = Source::Telegram;
    let copied = source;  // Copy, not clone
    assert_eq!(source, copied);
}

#[test]
fn test_output_format_copy() {
    let format = OutputFormat::Json;
    let copied = format;  // Copy, not clone
    assert_eq!(format, copied);
}

#[test]
fn test_source_debug() {
    let debug = format!("{:?}", Source::Telegram);
    assert!(debug.contains("Telegram"));
}

#[test]
fn test_output_format_debug() {
    let debug = format!("{:?}", OutputFormat::Json);
    assert!(debug.contains("Json"));
}

#[test]
fn test_source_hash() {
    use std::collections::HashSet;
    
    let mut set = HashSet::new();
    set.insert(Source::Telegram);
    set.insert(Source::WhatsApp);
    set.insert(Source::Telegram); // duplicate
    
    assert_eq!(set.len(), 2);
}

#[test]
fn test_output_format_hash() {
    use std::collections::HashSet;
    
    let mut set = HashSet::new();
    set.insert(OutputFormat::Json);
    set.insert(OutputFormat::Jsonl);
    set.insert(OutputFormat::Json); // duplicate
    
    assert_eq!(set.len(), 2);
}