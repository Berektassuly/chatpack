//! Toxic test data generator for stress testing chatpack.
//!
//! Usage: cargo run --bin gen_test -- [messages] [output]
//! Example: cargo run --bin gen_test -- 100000 heavy_test.json

use rand::Rng;
use rand::seq::SliceRandom;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

const ZALGO_CHARS: &[char] = &[
    '\u{0300}', '\u{0301}', '\u{0302}', '\u{0303}', '\u{0304}', '\u{0305}', '\u{0306}', '\u{0307}',
    '\u{0308}', '\u{0309}', '\u{030A}', '\u{030B}', '\u{030C}', '\u{030D}', '\u{030E}', '\u{030F}',
    '\u{0310}', '\u{0311}', '\u{0312}', '\u{0313}', '\u{0314}', '\u{0315}', '\u{0316}', '\u{0317}',
    '\u{0318}', '\u{0319}', '\u{031A}', '\u{031B}', '\u{031C}', '\u{031D}', '\u{031E}', '\u{031F}',
    '\u{0320}', '\u{0321}', '\u{0322}', '\u{0323}', '\u{0324}', '\u{0325}', '\u{0326}', '\u{0327}',
    '\u{0328}', '\u{0329}', '\u{032A}', '\u{032B}', '\u{032C}', '\u{032D}', '\u{032E}', '\u{032F}',
    '\u{0330}', '\u{0331}', '\u{0332}', '\u{0333}', '\u{0334}', '\u{0335}', '\u{0336}', '\u{0337}',
    '\u{0338}', '\u{0339}', '\u{033A}', '\u{033B}', '\u{033C}', '\u{033D}', '\u{033E}', '\u{033F}',
    '\u{0340}', '\u{0341}',
];

const EMOJIS: &[&str] = &[
    "😀",
    "😂",
    "🤣",
    "😍",
    "🥰",
    "😘",
    "🤔",
    "🙄",
    "😱",
    "🤯",
    "💀",
    "👻",
    "🎃",
    "🤖",
    "👽",
    "🦄",
    "🐉",
    "🌈",
    "⚡",
    "🔥",
    "💩",
    "🖕",
    "👍",
    "❤️",
    "💔",
    "🏳️‍🌈",
    "🇷🇺",
    "🇺🇸",
    "🇰🇿",
    "👨‍👩‍👧‍👦",
    "🧑‍🚀",
    "🏊‍♂️",
    "🤷‍♀️", // Complex emojis
];

const SENDERS: &[&str] = &[
    "Alice",
    "Bob",
    "Иван",
    "Мария",
    "村上",
    "محمد",
    "User;With;Semicolons",
    "User\"With\"Quotes",
    "User\nWith\nNewlines",
    "",
    "   ",
    "🔥FireUser🔥",
    "A̷̧̛̜l̶̨̛͓i̸̧̛̜c̷̨̛͓ȩ̸̛̜", // Zalgo name
];

fn main() {
    let args: Vec<String> = env::args().collect();

    let count: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100_000);

    let output = args.get(2).map(|s| s.as_str()).unwrap_or("heavy_test.json");

    let format = args.get(3).map(|s| s.as_str()).unwrap_or("telegram");

    println!("🧪 Toxic Generator");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   Messages: {}", count);
    println!("   Output:   {}", output);
    println!("   Format:   {}", format);
    println!();

    match format {
        "telegram" | "tg" => generate_telegram(count, output),
        "whatsapp" | "wa" => generate_whatsapp(count, output),
        _ => {
            eprintln!("Unknown format: {}. Use 'telegram' or 'whatsapp'", format);
            std::process::exit(1);
        }
    }
}

fn generate_telegram(count: usize, output: &str) {
    let file = File::create(output).expect("Failed to create output file");
    let mut writer = BufWriter::with_capacity(1024 * 1024, file); // 1MB buffer

    let mut rng = rand::thread_rng();

    writeln!(writer, "{{").unwrap();
    writeln!(writer, "  \"name\": \"Toxic Test Chat\",").unwrap();
    writeln!(writer, "  \"type\": \"personal_chat\",").unwrap();
    writeln!(writer, "  \"messages\": [").unwrap();

    let start = std::time::Instant::now();
    let mut bytes_written: usize = 0;

    for i in 0..count {
        let msg = generate_toxic_message(&mut rng, i);
        let sender = SENDERS.choose(&mut rng).unwrap();
        let timestamp = 1700000000 + (i as i64);

        let json_text = escape_json(&msg);
        let json_sender = escape_json(sender);

        let comma = if i < count - 1 { "," } else { "" };

        let line = format!(
            r#"    {{"id": {}, "type": "message", "date_unixtime": "{}", "from": "{}", "text": "{}"}}{}"#,
            i + 1,
            timestamp,
            json_sender,
            json_text,
            comma
        );

        bytes_written += line.len();
        writeln!(writer, "{}", line).unwrap();

        if (i + 1) % 10000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let mps = (i + 1) as f64 / elapsed;
            let mb = bytes_written as f64 / 1_000_000.0;
            eprint!(
                "\r   Generated {}/{} ({:.1} MB, {:.0} msg/s)",
                i + 1,
                count,
                mb,
                mps
            );
        }
    }

    writeln!(writer, "  ]").unwrap();
    writeln!(writer, "}}").unwrap();

    writer.flush().unwrap();

    let elapsed = start.elapsed();
    let mb = bytes_written as f64 / 1_000_000.0;

    println!("\n\n✅ Done!");
    println!("   Size: {:.2} MB", mb);
    println!("   Time: {:.2}s", elapsed.as_secs_f64());
    println!(
        "   Speed: {:.0} msg/s",
        count as f64 / elapsed.as_secs_f64()
    );
}

fn generate_whatsapp(count: usize, output: &str) {
    let file = File::create(output).expect("Failed to create output file");
    let mut writer = BufWriter::with_capacity(1024 * 1024, file);

    let mut rng = rand::thread_rng();
    let start = std::time::Instant::now();
    let mut bytes_written: usize = 0;

    // Mix of date formats
    let formats = [
        |i: usize| format!("[{}/15/24, 10:{}:00 AM]", (i % 12) + 1, i % 60), // US
        |i: usize| format!("[15.{:02}.24, 10:{}:00]", (i % 12) + 1, i % 60), // EU dot
        |i: usize| format!("26.10.2025, {}:{:02}", 10 + (i % 14), i % 60),   // RU no bracket
        |i: usize| format!("{}/01/2024, {}:{:02}", (i % 28) + 1, i % 24, i % 60), // EU slash
    ];

    for i in 0..count {
        let msg = generate_toxic_message(&mut rng, i);
        let sender = SENDERS.choose(&mut rng).unwrap();

        // Escape newlines for WhatsApp format (continuation lines)
        let msg_escaped = msg.replace('\n', " ");

        let format_fn = formats[i % formats.len()];
        let timestamp = format_fn(i);

        let line = format!("{} - {}: {}\n", timestamp, sender, msg_escaped);
        bytes_written += line.len();
        writer.write_all(line.as_bytes()).unwrap();

        // Occasionally insert garbage lines to test robustness
        if i % 1000 == 500 {
            let garbage = generate_garbage_line(&mut rng);
            writer.write_all(garbage.as_bytes()).unwrap();
            bytes_written += garbage.len();
        }

        if (i + 1) % 10000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let mps = (i + 1) as f64 / elapsed;
            let mb = bytes_written as f64 / 1_000_000.0;
            eprint!(
                "\r   Generated {}/{} ({:.1} MB, {:.0} msg/s)",
                i + 1,
                count,
                mb,
                mps
            );
        }
    }

    writer.flush().unwrap();

    let elapsed = start.elapsed();
    let mb = bytes_written as f64 / 1_000_000.0;

    println!("\n\n✅ Done!");
    println!("   Size: {:.2} MB", mb);
    println!("   Time: {:.2}s", elapsed.as_secs_f64());
    println!(
        "   Speed: {:.0} msg/s",
        count as f64 / elapsed.as_secs_f64()
    );
}

fn generate_toxic_message(rng: &mut impl Rng, index: usize) -> String {
    match index % 20 {
        // Normal messages
        0..=5 => format!("Normal message #{} with some text", index),

        // Messages with special chars
        6 => format!("Message with semicolons; here; and; there; index={}", index),
        7 => format!("Message with \"quotes\" and 'apostrophes' #{}", index),
        8 => format!("Message with\nnewlines\nand\ttabs #{}", index),

        // Emoji spam
        9 => {
            let emojis: String = (0..50)
                .map(|_| *EMOJIS.choose(rng).unwrap())
                .collect::<Vec<_>>()
                .join("");
            format!("Emoji spam: {} #{}", emojis, index)
        }

        // Zalgo text
        10 => generate_zalgo("This is zalgo text", rng),

        // Giant message (100KB+)
        11 => {
            let base = format!("Giant message #{}: ", index);
            let padding: String = (0..100_000).map(|_| 'X').collect();
            base + &padding
        }

        // Unicode edge cases
        12 => format!("Кириллица: Привет мир! #{}", index),
        13 => format!("日本語: こんにちは #{}", index),
        14 => format!("العربية: مرحبا #{}", index),
        15 => format!("Mixed: Hello Привет 你好 🌍 #{}", index),

        // Empty-ish
        16 => String::new(),
        17 => "   ".to_string(),
        18 => "\n\n\n".to_string(),

        // Control characters
        19 => format!("Control chars: \x00\x01\x02\x03 #{}", index),

        _ => format!("Fallback message #{}", index),
    }
}

fn generate_zalgo(text: &str, rng: &mut impl Rng) -> String {
    let mut result = String::new();
    for c in text.chars() {
        result.push(c);
        // Add 1-10 random combining characters
        let zalgo_count = rng.gen_range(1..=10);
        for _ in 0..zalgo_count {
            let zalgo = ZALGO_CHARS[rng.gen_range(0..ZALGO_CHARS.len())];
            result.push(zalgo);
        }
    }
    result
}

fn generate_garbage_line(rng: &mut impl Rng) -> String {
    match rng.gen_range(0..5) {
        0 => "This line has no timestamp or sender format\n".to_string(),
        1 => "[Invalid date format here] - : message\n".to_string(),
        2 => "-------------------------------------------\n".to_string(),
        3 => "\n".to_string(), // Empty line
        4 => "☠️💀👻 Random emoji line 👻💀☠️\n".to_string(),
        _ => "garbage\n".to_string(),
    }
}

fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}
