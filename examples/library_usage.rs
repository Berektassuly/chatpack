//! Example: Using chatpack as a library
//!
//! This example demonstrates how to use chatpack in your own projects.
//!
//! Run with: cargo run --example library_usage

use chatpack::prelude::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== chatpack Library Usage Examples ===\n");

    // Example 1: Create messages manually
    println!("1. Creating messages manually:");
    let messages = vec![
        Message::new("Alice", "Hello!"),
        Message::new("Alice", "How are you?"),
        Message::new("Bob", "I'm fine, thanks!"),
        Message::new("Bob", "And you?"),
        Message::new("Alice", "Great!"),
    ];

    for msg in &messages {
        println!("   {}: {}", msg.sender, msg.content);
    }

    // Example 2: Merge consecutive messages
    println!("\n2. After merging consecutive messages:");
    let merged = merge_consecutive(messages);

    for msg in &merged {
        println!("   {}: {}", msg.sender, msg.content.replace('\n', " | "));
    }
    println!("   (Merged from 5 to {} messages)", merged.len());

    // Example 3: Filter by sender
    println!("\n3. Filtering by sender (Alice only):");
    let alice_messages = vec![
        Message::new("Alice", "Message 1"),
        Message::new("Bob", "Message 2"),
        Message::new("Alice", "Message 3"),
    ];

    let config = FilterConfig::new().with_user("Alice".to_string());
    let filtered = apply_filters(alice_messages, &config);

    for msg in &filtered {
        println!("   {}: {}", msg.sender, msg.content);
    }

    // Example 4: Using builder pattern with metadata
    println!("\n4. Creating messages with metadata:");
    let msg = Message::new("Charlie", "Important message")
        .with_id(12345)
        .with_timestamp(chrono::Utc::now());

    println!("   Sender: {}", msg.sender);
    println!("   Content: {}", msg.content);
    println!("   ID: {:?}", msg.id);
    println!("   Timestamp: {:?}", msg.timestamp);

    // Example 5: Output configuration
    println!("\n5. Output configuration options:");
    let minimal = OutputConfig::new();
    let full = OutputConfig::all();

    println!(
        "   Minimal config - timestamps: {}",
        minimal.include_timestamps
    );
    println!("   Full config - timestamps: {}", full.include_timestamps);
    println!("   Full config - ids: {}", full.include_ids);

    // Example 6: Processing statistics
    println!("\n6. Processing statistics:");
    let stats = ProcessingStats::new(100, 65);
    println!("   Original: {} messages", stats.original_count);
    println!("   Merged: {} messages", stats.merged_count);
    println!("   Compression: {:.1}%", stats.compression_ratio());
    println!("   Messages saved: {}", stats.messages_saved());

    // Example 7: Serialization
    println!("\n7. JSON serialization:");
    let msg = Message::new("Dave", "Serializable message").with_id(999);
    let json = serde_json::to_string_pretty(&msg)?;
    println!("{}", json);

    // Example 8: Deserialization
    println!("\n8. JSON deserialization:");
    let json_str = r#"{"sender":"Eve","content":"From JSON","id":123}"#;
    let parsed: Message = serde_json::from_str(json_str)?;
    println!("   Parsed: {} said '{}'", parsed.sender, parsed.content);

    println!("\n=== Examples complete! ===");
    Ok(())
}
