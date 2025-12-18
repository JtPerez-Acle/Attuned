# attuned-store

Storage traits and in-memory backend for [Attuned](https://github.com/JtPerez-Acle/Attuned).

## Overview

This crate provides the `StateStore` trait for persisting user state snapshots, along with a production-ready in-memory implementation.

## Quick Start

```rust
use attuned_store::{StateStore, MemoryStore};
use attuned_core::{StateSnapshot, Source};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create in-memory store
    let store = MemoryStore::new();

    // Create and store a state snapshot
    let state = StateSnapshot::builder()
        .user_id("user_123")
        .source(Source::SelfReport)
        .axis("cognitive_load", 0.8)
        .build()?;

    store.upsert_latest(state).await?;

    // Retrieve latest state
    if let Some(state) = store.get_latest("user_123").await? {
        println!("Cognitive load: {}", state.get_axis("cognitive_load"));
    }

    Ok(())
}
```

## StateStore Trait

```rust
#[async_trait]
pub trait StateStore: Send + Sync {
    async fn upsert_latest(&self, state: StateSnapshot) -> Result<(), StoreError>;
    async fn get_latest(&self, user_id: &str) -> Result<Option<StateSnapshot>, StoreError>;
    async fn delete(&self, user_id: &str) -> Result<bool, StoreError>;
}
```

## Implementations

| Backend | Crate | Description |
|---------|-------|-------------|
| In-Memory | `attuned-store` | Thread-safe HashMap, good for development |
| Qdrant | `attuned-qdrant` | Vector database for production semantic search |

## License

Apache-2.0
