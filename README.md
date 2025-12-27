# chatpack

**Token-efficient chat export processing for LLM and RAG pipelines.**

[![CI](https://github.com/berektassuly/chatpack/actions/workflows/ci.yml/badge.svg)](https://github.com/berektassuly/chatpack/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/berektassuly/chatpack/branch/main/graph/badge.svg)](https://codecov.io/gh/berektassuly/chatpack)
[![Crates.io](https://img.shields.io/crates/v/chatpack.svg)](https://crates.io/crates/chatpack)
[![docs.rs](https://docs.rs/chatpack/badge.svg)](https://docs.rs/chatpack)
[![Downloads](https://img.shields.io/crates/d/chatpack.svg)](https://crates.io/crates/chatpack)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

[Website](https://chatpack.berektassuly.com) |
[API Docs](https://docs.rs/chatpack) |
[Benchmarks](docs/BENCHMARKS.md) |
[Export Guide](docs/EXPORT_GUIDE.md)

---

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Feature Flags](#feature-flags)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

Chatpack parses chat exports from Telegram, WhatsApp, Instagram, and Discord, converting them into token-efficient formats for LLM analysis and RAG ingestion.

Raw chat exports waste 80%+ of tokens on JSON structure, metadata, and formatting. Chatpack removes this noise, achieving **13x compression** (92% token reduction) with CSV output.

```
┌─────────────────┐                   ┌─────────────────┐
│ Telegram JSON   │                   │ Clean CSV       │
│ WhatsApp TXT    │ ──▶ chatpack ──▶ │ 13x compression │
│ Instagram JSON  │                   │ LLM-ready       │
│ Discord Export  │                   │ RAG-optimized   │
└─────────────────┘                   └─────────────────┘
```

### Token Compression Results

| Format | Input | Output | Compression |
|--------|-------|--------|-------------|
| **CSV** | 11.2M tokens | 850K tokens | **13x (92%)** |
| JSONL | 11.2M tokens | 1.0M tokens | 11x (91%) |
| JSON | 11.2M tokens | 1.3M tokens | 8x (88%) |

---

## Installation

```bash
cargo add chatpack
```

Or add to Cargo.toml:

```toml
[dependencies]
chatpack = "0.5"
```

See [Feature Flags](#feature-flags) for minimal installations.

---

## Quick Start

```rust
use chatpack::prelude::*;
use chatpack::parser::{Platform, create_parser};

fn main() -> Result<()> {
    // Parse Telegram export
    let parser = create_parser(Platform::Telegram);
    let messages = parser.parse("export.json".as_ref())?;

    // Merge consecutive messages from same sender
    let merged = merge_consecutive(messages);

    // Write as CSV (13x compression)
    write_csv(&merged, "output.csv", &OutputConfig::new())?;

    println!("Processed {} messages", merged.len());
    Ok(())
}
```

---

## Documentation

| Guide | Description |
|-------|-------------|
| [Export Guide](docs/EXPORT_GUIDE.md) | How to export from Telegram, WhatsApp, Instagram, Discord |
| [Benchmarks](docs/BENCHMARKS.md) | Performance metrics and compression stats |
| [API Docs](https://docs.rs/chatpack) | Structs, traits, and functions reference |

---

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `full` | All parsers + outputs + streaming | Yes |
| `telegram` | Telegram JSON parser | Yes |
| `whatsapp` | WhatsApp TXT parser | Yes |
| `instagram` | Instagram JSON parser | Yes |
| `discord` | Discord multi-format parser | Yes |
| `csv-output` | CSV output writer | Yes |
| `json-output` | JSON/JSONL output writers | Yes |
| `streaming` | O(1) memory streaming parsers | Yes |
| `async` | Tokio-based async parsers | No |

Minimal installation example:

```toml
chatpack = { version = "0.5", default-features = false, features = ["telegram", "csv-output"] }
```

---

## Minimum Supported Rust Version

Rust 2024 edition (rustc 1.85+). MSRV is tested in CI.

---

## Contributing

Contributions welcome! Please:

1. Check existing issues
2. Open an issue before large changes
3. Run `cargo test && cargo clippy` before submitting PRs

---

## License

MIT License. See [LICENSE](LICENSE) for details.

---

[Try chatpack online](https://chatpack.berektassuly.com) - no installation required!