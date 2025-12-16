//! # attuned-store
//!
//! Storage traits and in-memory backend for Attuned.
//!
//! This crate provides:
//! - [`StateStore`] trait defining the storage contract
//! - [`MemoryStore`] in-memory implementation for single-process apps
//!
//! ## Example
//!
//! ```rust
//! use attuned_store::{StateStore, MemoryStore, MemoryStoreConfig};
//! use attuned_core::{StateSnapshot, Source};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let store = MemoryStore::new(MemoryStoreConfig::default());
//!
//!     let snapshot = StateSnapshot::builder()
//!         .user_id("user_123")
//!         .source(Source::SelfReport)
//!         .axis("warmth", 0.7)
//!         .build()?;
//!
//!     store.upsert_latest(snapshot).await?;
//!
//!     let retrieved = store.get_latest("user_123").await?;
//!     assert!(retrieved.is_some());
//!
//!     Ok(())
//! }
//! ```

#![deny(missing_docs)]

mod error;
mod memory;
mod traits;

pub use error::StoreError;
pub use memory::{MemoryStore, MemoryStoreConfig};
pub use traits::StateStore;
