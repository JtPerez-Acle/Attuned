//! # attuned-qdrant
//!
//! Qdrant storage backend for Attuned.
//!
//! This crate provides a persistent, distributed storage backend using Qdrant
//! vector database. Qdrant is used as a snapshot store (not for semantic search).
//!
//! ## Features
//!
//! - Persistent state storage across restarts
//! - Optional history snapshot retention
//! - Connection pooling and retry logic
//! - Full observability integration
//!
//! ## Example
//!
//! ```rust,ignore
//! use attuned_qdrant::{QdrantStore, QdrantStoreConfig};
//! use attuned_store::StateStore;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = QdrantStoreConfig {
//!         url: "http://localhost:6334".to_string(),
//!         collection_name: "attuned_state".to_string(),
//!         ..Default::default()
//!     };
//!
//!     let store = QdrantStore::new(config).await?;
//!     // Use store via StateStore trait...
//!     Ok(())
//! }
//! ```

#![deny(missing_docs)]

// TODO: Implement Qdrant backend (TASK-004)

mod config;
mod error;
mod store;

pub use config::QdrantStoreConfig;
pub use error::QdrantError;
pub use store::QdrantStore;
