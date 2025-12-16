//! Storage trait definitions.

use crate::error::StoreError;
use async_trait::async_trait;
use attuned_core::StateSnapshot;

/// Trait for storing and retrieving user state snapshots.
///
/// Implementations must be thread-safe (`Send + Sync`) and support
/// async operations.
#[async_trait]
pub trait StateStore: Send + Sync {
    /// Insert or update the latest state snapshot for a user.
    ///
    /// If a snapshot already exists for the user, it is replaced.
    /// The snapshot is validated before storage.
    async fn upsert_latest(&self, snapshot: StateSnapshot) -> Result<(), StoreError>;

    /// Get the latest state snapshot for a user.
    ///
    /// Returns `None` if no state exists for the user.
    async fn get_latest(&self, user_id: &str) -> Result<Option<StateSnapshot>, StoreError>;

    /// Delete the state for a user.
    ///
    /// This removes all state data for the user (GDPR compliance).
    /// Returns `Ok(())` even if the user did not exist.
    async fn delete(&self, user_id: &str) -> Result<(), StoreError> {
        // Default implementation - stores should override if they need custom logic
        let _ = user_id;
        Ok(())
    }

    /// Get historical snapshots for a user.
    ///
    /// Returns up to `limit` snapshots, ordered by most recent first.
    /// Default implementation returns empty vec (history not supported).
    async fn get_history(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<StateSnapshot>, StoreError> {
        let _ = (user_id, limit);
        Ok(vec![])
    }

    /// Check if the store is healthy and can accept requests.
    ///
    /// Default implementation always returns true.
    async fn health_check(&self) -> Result<bool, StoreError> {
        Ok(true)
    }
}
