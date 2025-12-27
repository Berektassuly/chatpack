//! Property-based tests for chatpack.
//!
//! These tests generate random inputs to find edge cases.

use proptest::prelude::*;

use chatpack::core::{FilterConfig, Message, apply_filters, merge_consecutive};

/// Generate a random Message using fast strategies (no regex!)
fn arb_message() -> impl Strategy<Value = Message> {
    (
        // Fast: select from predefined senders
        prop::sample::select(vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
            "User123".to_string(),
            "Иван".to_string(),
            "Test".to_string(),
        ]),
        // Fast: select from predefined contents
        prop::sample::select(vec![
            "Hello".to_string(),
            "Hi there!".to_string(),
            "How are you?".to_string(),
            "Good morning".to_string(),
            "Test message 123".to_string(),
            "Привет мир".to_string(),
            String::new(),
            "   ".to_string(),
            "Special;chars\"here\nnewline".to_string(),
            "🎉🔥💀 emoji".to_string(),
        ]),
    )
        .prop_map(|(sender, content)| Message {
            sender,
            content,
            timestamp: None,
            id: None,
            reply_to: None,
            edited: None,
        })
}

/// Generate a vector of random messages
fn arb_messages(max_len: usize) -> impl Strategy<Value = Vec<Message>> {
    prop::collection::vec(arb_message(), 0..max_len)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // ============================================
    // MERGE PROPERTIES
    // ============================================

    /// Merge never increases message count
    #[test]
    fn merge_never_increases_count(messages in arb_messages(20)) {
        let original_len = messages.len();
        let merged = merge_consecutive(messages);
        prop_assert!(merged.len() <= original_len);
    }

    /// Empty input produces empty output
    #[test]
    fn merge_empty_is_empty(_dummy in Just(())) {
        let result = merge_consecutive(vec![]);
        prop_assert!(result.is_empty());
    }

    /// Single message stays single
    #[test]
    fn merge_single_stays_single(msg in arb_message()) {
        let result = merge_consecutive(vec![msg]);
        prop_assert_eq!(result.len(), 1);
    }

    /// Merge with same sender produces exactly one message
    #[test]
    fn merge_same_sender_produces_one(n in 1usize..10) {
        let messages: Vec<Message> = (0..n)
            .map(|i| Message {
                sender: "Alice".to_string(),
                content: format!("Message {}", i),
                timestamp: None,
                id: None,
                reply_to: None,
                edited: None,
            })
            .collect();
        let merged = merge_consecutive(messages);
        prop_assert_eq!(merged.len(), 1);
    }

    /// Merge preserves total content
    #[test]
    fn merge_preserves_content_parts(n in 1usize..5) {
        let messages: Vec<Message> = (0..n)
            .map(|i| Message {
                sender: "Alice".to_string(),
                content: format!("part{}", i),
                timestamp: None,
                id: None,
                reply_to: None,
                edited: None,
            })
            .collect();
        let merged = merge_consecutive(messages);
        for i in 0..n {
            let expected = format!("part{}", i);
            prop_assert!(merged[0].content.contains(&expected), "Missing part{}", i);
        }
    }

    // ============================================
    // FILTER PROPERTIES
    // ============================================

    /// Filter never increases message count
    #[test]
    fn filter_never_increases_count(messages in arb_messages(20)) {
        let original_len = messages.len();
        let config = FilterConfig::new(); // no filters = passthrough
        let filtered = apply_filters(messages, &config);
        prop_assert!(filtered.len() <= original_len);
    }

    /// No filter means passthrough
    #[test]
    fn no_filter_is_passthrough(messages in arb_messages(20)) {
        let original_len = messages.len();
        let config = FilterConfig::new();
        let filtered = apply_filters(messages, &config);
        prop_assert_eq!(filtered.len(), original_len);
    }

    /// User filter only keeps matching users (case insensitive)
    #[test]
    fn user_filter_only_keeps_matching(messages in arb_messages(20)) {
        let config = FilterConfig::new().with_user("Alice".to_string());
        let filtered = apply_filters(messages, &config);

        for msg in &filtered {
            prop_assert!(
                msg.sender.eq_ignore_ascii_case("Alice"),
                "Found non-matching sender: {}", msg.sender
            );
        }
    }

    // ============================================
    // ROBUSTNESS PROPERTIES
    // ============================================

    /// Merge never panics on any input
    #[test]
    fn merge_never_panics(messages in arb_messages(30)) {
        let _ = merge_consecutive(messages);
    }

    /// Filter never panics on any input
    #[test]
    fn filter_never_panics(messages in arb_messages(30)) {
        let config = FilterConfig::new().with_user("Test".to_string());
        let _ = apply_filters(messages, &config);
    }

    // ============================================
    // CONTENT EDGE CASES
    // ============================================

    /// Messages with special characters don't break merge
    #[test]
    fn special_chars_dont_break_merge(sender in prop::sample::select(vec!["A", "B", "C"])) {
        let msg = Message {
            sender: sender.to_string(),
            content: "test;with\"special\nchars\ttab".to_string(),
            timestamp: None,
            id: None,
            reply_to: None,
            edited: None,
        };
        let _ = merge_consecutive(vec![msg.clone(), msg]);
    }

    /// Unicode content is handled correctly
    #[test]
    fn unicode_content_preserved(idx in 0usize..5) {
        let contents = ["Привет", "こんにちは", "مرحبا", "🎉🔥💀", "Mixed Тест 日本"];
        let content = contents[idx].to_string();
        let msg = Message {
            sender: "User".to_string(),
            content: content.clone(),
            timestamp: None,
            id: None,
            reply_to: None,
            edited: None,
        };
        let merged = merge_consecutive(vec![msg]);
        prop_assert_eq!(&merged[0].content, &content);
    }

    // ============================================
    // SERDE ROUNDTRIP
    // ============================================

    /// Message serialization roundtrip
    #[test]
    fn message_serde_roundtrip(msg in arb_message()) {
        let json = serde_json::to_string(&msg).expect("serialize");
        let parsed: Message = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(msg.sender, parsed.sender);
        prop_assert_eq!(msg.content, parsed.content);
    }
}

// ============================================
// NON-PROPTEST EDGE CASE TESTS
// ============================================

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn merge_consecutive_same_sender() {
        let messages = vec![
            Message {
                sender: "Alice".into(),
                content: "Hello".into(),
                timestamp: None,
                id: None,
                reply_to: None,
                edited: None,
            },
            Message {
                sender: "Alice".into(),
                content: "World".into(),
                timestamp: None,
                id: None,
                reply_to: None,
                edited: None,
            },
        ];

        let merged = merge_consecutive(messages);
        assert_eq!(merged.len(), 1);
        assert!(merged[0].content.contains("Hello"));
        assert!(merged[0].content.contains("World"));
    }

    #[test]
    fn merge_alternating_senders() {
        let messages = vec![
            Message {
                sender: "Alice".into(),
                content: "Hi".into(),
                timestamp: None,
                id: None,
                reply_to: None,
                edited: None,
            },
            Message {
                sender: "Bob".into(),
                content: "Hey".into(),
                timestamp: None,
                id: None,
                reply_to: None,
                edited: None,
            },
            Message {
                sender: "Alice".into(),
                content: "Bye".into(),
                timestamp: None,
                id: None,
                reply_to: None,
                edited: None,
            },
        ];

        let merged = merge_consecutive(messages);
        assert_eq!(merged.len(), 3); // No merge - different senders
    }

    #[test]
    fn filter_empty_messages() {
        let messages: Vec<Message> = vec![];
        let config = FilterConfig::new().with_user("Anyone".into());
        let filtered = apply_filters(messages, &config);
        assert!(filtered.is_empty());
    }

    #[test]
    fn merge_with_empty_content() {
        let messages = vec![
            Message {
                sender: "Alice".into(),
                content: String::new(),
                timestamp: None,
                id: None,
                reply_to: None,
                edited: None,
            },
            Message {
                sender: "Alice".into(),
                content: "Real message".into(),
                timestamp: None,
                id: None,
                reply_to: None,
                edited: None,
            },
        ];

        let merged = merge_consecutive(messages);
        assert_eq!(merged.len(), 1);
    }
}
