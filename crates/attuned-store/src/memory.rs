//! In-memory state store implementation.

use crate::error::StoreError;
use crate::traits::StateStore;
use async_trait::async_trait;
use attuned_core::{ComponentHealth, HealthCheck, StateSnapshot};
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::Arc;

/// Configuration for the in-memory store.
#[derive(Clone, Debug)]
pub struct MemoryStoreConfig {
    /// Whether to keep historical snapshots.
    pub enable_history: bool,
    /// Maximum number of historical snapshots per user.
    pub max_history_per_user: usize,
}

impl Default for MemoryStoreConfig {
    fn default() -> Self {
        Self {
            enable_history: false,
            max_history_per_user: 100,
        }
    }
}

/// Thread-safe in-memory state store.
///
/// Uses [`DashMap`] for lock-free concurrent access.
/// Suitable for single-process applications and testing.
#[derive(Clone)]
pub struct MemoryStore {
    latest: Arc<DashMap<String, StateSnapshot>>,
    history: Option<Arc<DashMap<String, VecDeque<StateSnapshot>>>>,
    config: MemoryStoreConfig,
}

impl MemoryStore {
    /// Create a new in-memory store with the given configuration.
    pub fn new(config: MemoryStoreConfig) -> Self {
        let history = if config.enable_history {
            Some(Arc::new(DashMap::new()))
        } else {
            None
        };

        Self {
            latest: Arc::new(DashMap::new()),
            history,
            config,
        }
    }

    /// Get the number of users with stored state.
    pub fn len(&self) -> usize {
        self.latest.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.latest.is_empty()
    }

    /// Clear all stored state.
    pub fn clear(&self) {
        self.latest.clear();
        if let Some(ref history) = self.history {
            history.clear();
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new(MemoryStoreConfig::default())
    }
}

#[async_trait]
impl StateStore for MemoryStore {
    #[tracing::instrument(skip(self, snapshot), fields(user_id = %snapshot.user_id))]
    async fn upsert_latest(&self, snapshot: StateSnapshot) -> Result<(), StoreError> {
        // Validate the snapshot
        snapshot.validate()?;

        let user_id = snapshot.user_id.clone();

        // Store in history if enabled
        if let Some(ref history) = self.history {
            let mut entry = history.entry(user_id.clone()).or_insert_with(VecDeque::new);
            entry.push_front(snapshot.clone());

            // Trim to max history size
            while entry.len() > self.config.max_history_per_user {
                entry.pop_back();
            }
        }

        // Store as latest
        self.latest.insert(user_id, snapshot);

        tracing::debug!("upserted state snapshot");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(user_id = %user_id))]
    async fn get_latest(&self, user_id: &str) -> Result<Option<StateSnapshot>, StoreError> {
        let result = self.latest.get(user_id).map(|r| r.value().clone());
        tracing::debug!(found = result.is_some(), "retrieved state snapshot");
        Ok(result)
    }

    #[tracing::instrument(skip(self), fields(user_id = %user_id))]
    async fn delete(&self, user_id: &str) -> Result<(), StoreError> {
        self.latest.remove(user_id);
        if let Some(ref history) = self.history {
            history.remove(user_id);
        }
        tracing::debug!("deleted user state");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(user_id = %user_id, limit = %limit))]
    async fn get_history(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<StateSnapshot>, StoreError> {
        let result = match &self.history {
            Some(history) => history
                .get(user_id)
                .map(|entry| entry.iter().take(limit).cloned().collect())
                .unwrap_or_default(),
            None => vec![],
        };
        tracing::debug!(count = result.len(), "retrieved history");
        Ok(result)
    }

    async fn health_check(&self) -> Result<bool, StoreError> {
        Ok(true)
    }
}

#[async_trait]
impl HealthCheck for MemoryStore {
    async fn check(&self) -> ComponentHealth {
        ComponentHealth::healthy("memory_store")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use attuned_core::Source;

    fn test_snapshot(user_id: &str) -> StateSnapshot {
        StateSnapshot::builder()
            .user_id(user_id)
            .source(Source::SelfReport)
            .axis("warmth", 0.7)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_upsert_and_get() {
        let store = MemoryStore::default();
        let snapshot = test_snapshot("user_1");

        store.upsert_latest(snapshot.clone()).await.unwrap();

        let retrieved = store.get_latest("user_1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_id, "user_1");
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let store = MemoryStore::default();
        let result = store.get_latest("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete() {
        let store = MemoryStore::default();
        store.upsert_latest(test_snapshot("user_1")).await.unwrap();

        store.delete("user_1").await.unwrap();

        assert!(store.get_latest("user_1").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_history() {
        let config = MemoryStoreConfig {
            enable_history: true,
            max_history_per_user: 5,
        };
        let store = MemoryStore::new(config);

        // Insert multiple snapshots
        for i in 0..3 {
            let mut snapshot = test_snapshot("user_1");
            snapshot.axes.insert("warmth".to_string(), i as f32 / 10.0);
            store.upsert_latest(snapshot).await.unwrap();
        }

        let history = store.get_history("user_1", 10).await.unwrap();
        assert_eq!(history.len(), 3);
    }

    #[tokio::test]
    async fn test_history_limit() {
        let config = MemoryStoreConfig {
            enable_history: true,
            max_history_per_user: 3,
        };
        let store = MemoryStore::new(config);

        // Insert more than max
        for i in 0..5 {
            let mut snapshot = test_snapshot("user_1");
            snapshot.axes.insert("warmth".to_string(), i as f32 / 10.0);
            store.upsert_latest(snapshot).await.unwrap();
        }

        let history = store.get_history("user_1", 10).await.unwrap();
        assert_eq!(history.len(), 3); // Limited to max
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let store = MemoryStore::default();
        let store = Arc::new(store);

        let handles: Vec<_> = (0..100)
            .map(|i| {
                let store = store.clone();
                tokio::spawn(async move {
                    let snapshot = test_snapshot(&format!("user_{}", i));
                    store.upsert_latest(snapshot).await
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        assert_eq!(store.len(), 100);
    }
}
