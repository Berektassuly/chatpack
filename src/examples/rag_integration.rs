//! Example: How chatrag would use chatpack
//!
//! This demonstrates how a RAG system would integrate with chatpack.
//! This is pseudocode showing the integration pattern.

use chatpack::prelude::*;

/// Example chunk structure for RAG
#[derive(Debug)]
struct ChatChunk {
    id: String,
    text: String,
    sender: String,
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
    message_ids: Vec<u64>,
}

/// Example: Time-window based chunking strategy
fn chunk_by_time_window(
    messages: &[InternalMessage],
    window_minutes: i64,
) -> Vec<ChatChunk> {
    let mut chunks = Vec::new();
    let mut current_chunk: Option<ChatChunk> = None;

    for msg in messages {
        let should_start_new = match (&current_chunk, msg.timestamp) {
            (Some(chunk), Some(msg_ts)) => {
                match chunk.timestamp {
                    Some(chunk_ts) => {
                        let diff = (msg_ts - chunk_ts).num_minutes();
                        diff > window_minutes
                    }
                    None => true,
                }
            }
            (None, _) => true,
            _ => false,
        };

        if should_start_new {
            if let Some(chunk) = current_chunk.take() {
                chunks.push(chunk);
            }
            current_chunk = Some(ChatChunk {
                id: format!("chunk_{}", chunks.len()),
                text: format!("{}: {}", msg.sender, msg.content),
                sender: msg.sender.clone(),
                timestamp: msg.timestamp,
                message_ids: msg.id.into_iter().collect(),
            });
        } else if let Some(ref mut chunk) = current_chunk {
            chunk.text.push_str(&format!("\n{}: {}", msg.sender, msg.content));
            if let Some(id) = msg.id {
                chunk.message_ids.push(id);
            }
        }
    }

    if let Some(chunk) = current_chunk {
        chunks.push(chunk);
    }

    chunks
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== RAG Integration Example ===\n");

    // Step 1: Parse chat using chatpack
    println!("Step 1: Parse chat export");
    
    // In real usage, this would be:
    // let parser = create_parser(Source::Telegram);
    // let messages = parser.parse("chat.json")?;
    
    // For demo, create sample messages
    use chrono::{TimeZone, Utc};
    
    let base_time = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
    
    let messages = vec![
        InternalMessage::new("Alice", "Hey, have you seen the new project specs?")
            .with_timestamp(base_time)
            .with_id(1),
        InternalMessage::new("Bob", "Yes! They look good")
            .with_timestamp(base_time + chrono::Duration::minutes(1))
            .with_id(2),
        InternalMessage::new("Bob", "But I have some concerns about the timeline")
            .with_timestamp(base_time + chrono::Duration::minutes(2))
            .with_id(3),
        InternalMessage::new("Alice", "Let's discuss in the meeting")
            .with_timestamp(base_time + chrono::Duration::minutes(3))
            .with_id(4),
        // Gap of 30 minutes
        InternalMessage::new("Alice", "Meeting notes: decided to extend deadline")
            .with_timestamp(base_time + chrono::Duration::minutes(35))
            .with_id(5),
        InternalMessage::new("Bob", "Great, I'll update the schedule")
            .with_timestamp(base_time + chrono::Duration::minutes(36))
            .with_id(6),
    ];

    println!("   Loaded {} messages", messages.len());

    // Step 2: Optionally merge consecutive messages
    println!("\nStep 2: Merge consecutive messages");
    let merged = merge_consecutive(messages);
    println!("   Merged to {} messages", merged.len());

    // Step 3: Apply filters if needed
    println!("\nStep 3: Apply filters (none in this example)");
    let config = FilterConfig::new();
    let filtered = apply_filters(merged, &config);
    println!("   {} messages after filtering", filtered.len());

    // Step 4: Chunk for RAG
    println!("\nStep 4: Chunk by time window (5 minutes)");
    let chunks = chunk_by_time_window(&filtered, 5);
    
    for chunk in &chunks {
        println!("\n   Chunk {}:", chunk.id);
        println!("   Timestamp: {:?}", chunk.timestamp);
        println!("   Message IDs: {:?}", chunk.message_ids);
        println!("   Text preview: {}...", 
            chunk.text.chars().take(50).collect::<String>());
    }

    // Step 5: Would embed and store (pseudocode)
    println!("\nStep 5: Embed and store (pseudocode)");
    println!("   // let embeddings = embedder.embed_batch(&texts).await?;");
    println!("   // store.upsert_batch(&chunks, &embeddings).await?;");

    // Step 6: Search (pseudocode)
    println!("\nStep 6: Search (pseudocode)");
    println!("   // let results = store.search(\"project deadline\", 5).await?;");
    println!("   // let context = results.iter().map(|r| r.text).collect();");
    println!("   // let answer = llm.ask(query, context).await?;");

    println!("\n=== Integration complete! ===");
    println!("\nKey chatpack types used:");
    println!("   - InternalMessage: Universal message format");
    println!("   - create_parser(Source): Get appropriate parser");
    println!("   - merge_consecutive(): Reduce message count");
    println!("   - apply_filters(): Filter by date/sender");
    println!("   - ProcessingStats: Track compression");

    Ok(())
}