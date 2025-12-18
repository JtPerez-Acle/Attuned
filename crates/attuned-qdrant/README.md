# attuned-qdrant

Qdrant vector database backend for [Attuned](https://github.com/JtPerez-Acle/Attuned).

## Overview

This crate provides a `StateStore` implementation using [Qdrant](https://qdrant.tech/) for production deployments requiring:

- Semantic similarity search across user states
- Horizontal scaling
- Persistent storage with vector indexing

## Quick Start

```rust
use attuned_qdrant::{QdrantStore, QdrantConfig};
use attuned_store::StateStore;
use attuned_core::{StateSnapshot, Source};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Qdrant
    let config = QdrantConfig {
        url: "http://localhost:6334".to_string(),
        collection_name: "attuned_states".to_string(),
        ..Default::default()
    };
    let store = QdrantStore::new(config).await?;

    // Use like any StateStore
    let state = StateSnapshot::builder()
        .user_id("user_123")
        .source(Source::SelfReport)
        .axis("cognitive_load", 0.8)
        .build()?;

    store.upsert_latest(state).await?;

    Ok(())
}
```

## Configuration

```rust
pub struct QdrantConfig {
    pub url: String,              // Qdrant server URL
    pub collection_name: String,  // Collection for states
    pub api_key: Option<String>,  // Optional API key
    pub vector_size: u64,         // Default: 23 (canonical axes)
}
```

## Running Qdrant

```bash
# Docker
docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant

# Or use Qdrant Cloud
# https://cloud.qdrant.io
```

## License

Apache-2.0
