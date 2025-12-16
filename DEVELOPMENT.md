# Development Guide

This document helps developers get started with Attuned development.

**Before contributing, please read:**
- [MANIFESTO.md](MANIFESTO.md) - Our philosophical principles and hard constraints
- [SECURITY.md](SECURITY.md) - Security policies

## Prerequisites

- **Rust 1.75+** (tested with 1.92.0)
- **Git**

## Quick Setup

```bash
# Clone the repository
git clone https://github.com/attuned-dev/attuned.git
cd attuned

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run security audit
cargo audit

# Run clippy (linting)
cargo clippy --workspace

# Format code
cargo fmt --all
```

## Project Structure

```
attuned/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── attuned-core/          # Core types, axes, translator, telemetry
│   │   └── src/
│   │       ├── lib.rs         # Public API exports
│   │       ├── types.rs       # Source enum
│   │       ├── axes.rs        # 23 canonical axes with governance
│   │       ├── snapshot.rs    # StateSnapshot + builder
│   │       ├── translator.rs  # Translator trait + RuleTranslator
│   │       ├── error.rs       # Error types
│   │       └── telemetry/     # Observability infrastructure
│   │
│   ├── attuned-store/         # Storage abstraction
│   │   └── src/
│   │       ├── traits.rs      # StateStore trait
│   │       └── memory.rs      # MemoryStore implementation
│   │
│   ├── attuned-qdrant/        # Qdrant backend (stubs)
│   │
│   ├── attuned-http/          # HTTP server with security
│   │   └── src/
│   │       ├── server.rs      # Server setup + routes
│   │       ├── handlers.rs    # Request handlers
│   │       ├── middleware.rs  # Security, auth, rate limiting
│   │       └── config.rs
│   │
│   └── attuned-cli/           # CLI tool
│
├── tasks/                     # Task tracking
├── docs/                      # Additional documentation
├── .cargo/audit.toml          # Security audit config
├── README.md                  # User documentation
├── MANIFESTO.md               # Philosophy & governance
├── SECURITY.md                # Security policies
├── DEVELOPMENT.md             # This file
└── CHANGELOG.md               # Release history
```

## Running the Server

```bash
# Build and run with default settings (127.0.0.1:8080)
cargo run -p attuned-cli -- serve

# Or use the HTTP server directly in code:
cargo run --example server  # (if example exists)
```

## Testing

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p attuned-core

# Run a specific test
cargo test -p attuned-core translator::tests::test_high_cognitive_load

# Run with output
cargo test -- --nocapture
```

### Current Test Coverage

| Crate | Tests | Status |
|-------|-------|--------|
| attuned-core | 36 | All pass |
| attuned-store | 6 | All pass |
| attuned-http | 10 | All pass |
| attuned-qdrant | 0 | Stubs only |
| attuned-cli | 0 | Basic structure |
| **Total** | **52** | **All pass** |

## API Quick Reference

### Core Types

```rust
use attuned_core::{StateSnapshot, Source, RuleTranslator, Translator};

// Create a state snapshot
let snapshot = StateSnapshot::builder()
    .user_id("user_123")
    .source(Source::SelfReport)
    .confidence(1.0)
    .axis("warmth", 0.7)
    .axis("cognitive_load", 0.9)
    .build()?;

// Translate to prompt context
let translator = RuleTranslator::default();
let context = translator.to_prompt_context(&snapshot);
// context.guidelines, context.tone, context.verbosity, context.flags
```

### Storage

```rust
use attuned_store::{StateStore, MemoryStore, MemoryStoreConfig};

let store = MemoryStore::new(MemoryStoreConfig {
    enable_history: true,
    max_history_per_user: 100,
});

store.upsert_latest(snapshot).await?;
let retrieved = store.get_latest("user_123").await?;
```

### HTTP Server

```rust
use attuned_http::{Server, ServerConfig};
use attuned_store::MemoryStore;

let store = MemoryStore::default();
let config = ServerConfig {
    bind_addr: "0.0.0.0:8080".parse()?,
    ..Default::default()
};

let server = Server::new(store, config);
server.run().await?;
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- All public items must have doc comments
- Tests go in the same file as the code (`#[cfg(test)] mod tests`)

## Adding a New Axis

New axes require full governance metadata. See [MANIFESTO.md](MANIFESTO.md) for principles.

1. Create a new `AxisDefinition` constant in `crates/attuned-core/src/axes.rs`:
   ```rust
   pub const NEW_AXIS: AxisDefinition = AxisDefinition {
       name: "new_axis",
       category: AxisCategory::...,
       description: "...",
       low_anchor: "What 0.0 means",
       high_anchor: "What 1.0 means",
       intent: &[
           "Legitimate use case 1",
           "Legitimate use case 2",
       ],
       forbidden_uses: &[
           "Anti-pattern 1",
           "Anti-pattern 2",
       ],
       since: "0.2.0",  // Next version
       deprecated: None,
   };
   ```
2. Add to `CANONICAL_AXES` array
3. Update test count in `test_canonical_axes_count`
4. Add translation rules in `RuleTranslator::to_prompt_context` if needed
5. Update README.md axis documentation

## Implementing Qdrant Backend (TASK-004)

The Qdrant store has stubs in place. To implement:

1. Add `qdrant_client::Qdrant` to `QdrantStore` struct
2. Implement `new()` to connect and ensure collection exists
3. Implement `StateStore` trait methods:
   - `upsert_latest`: Store as point with ID `{user_id}::latest`
   - `get_latest`: Retrieve point by ID
   - `delete`: Remove point
   - `get_history`: Query points with ID pattern `{user_id}::*`

See `crates/attuned-qdrant/src/store.rs` for the skeleton.

## Task Management

This project uses task files for agentic development continuity.

```bash
# List active tasks
ls tasks/TASK-*.md

# Find in-progress tasks
grep -l "In Progress" tasks/TASK-*.md

# View remaining tasks
cat tasks/  # Check remaining_tasks memory or task files
```

See `docs/TASK_MANAGEMENT.md` for the full workflow.
