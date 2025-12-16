//! Qdrant StateStore implementation.

use crate::config::QdrantStoreConfig;
use crate::error::QdrantError;
use async_trait::async_trait;
use attuned_core::StateSnapshot;
use attuned_store::{StateStore, StoreError};

/// Qdrant-backed state store.
///
/// Stores state snapshots in a Qdrant collection for persistence
/// across application restarts.
pub struct QdrantStore {
    #[allow(dead_code)]
    config: QdrantStoreConfig,
    // TODO: Add qdrant_client::Qdrant client
}

impl QdrantStore {
    /// Create a new Qdrant store with the given configuration.
    ///
    /// This will connect to Qdrant and ensure the collection exists.
    pub async fn new(config: QdrantStoreConfig) -> Result<Self, QdrantError> {
        // TODO: Implement connection and collection setup
        Ok(Self { config })
    }

    /// Check if the Qdrant server is healthy.
    pub async fn health_check(&self) -> Result<bool, QdrantError> {
        // TODO: Implement health check
        Ok(true)
    }
}

#[async_trait]
impl StateStore for QdrantStore {
    async fn upsert_latest(&self, _snapshot: StateSnapshot) -> Result<(), StoreError> {
        // TODO: Implement (TASK-004)
        todo!("Qdrant upsert not yet implemented")
    }

    async fn get_latest(&self, _user_id: &str) -> Result<Option<StateSnapshot>, StoreError> {
        // TODO: Implement (TASK-004)
        todo!("Qdrant get not yet implemented")
    }

    async fn delete(&self, _user_id: &str) -> Result<(), StoreError> {
        // TODO: Implement (TASK-004)
        todo!("Qdrant delete not yet implemented")
    }

    async fn get_history(
        &self,
        _user_id: &str,
        _limit: usize,
    ) -> Result<Vec<StateSnapshot>, StoreError> {
        // TODO: Implement (TASK-004)
        todo!("Qdrant history not yet implemented")
    }
}
