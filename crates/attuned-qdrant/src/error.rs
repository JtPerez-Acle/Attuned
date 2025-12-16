//! Qdrant-specific error types.

use thiserror::Error;

/// Errors specific to the Qdrant backend.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum QdrantError {
    /// Failed to connect to Qdrant.
    #[error("failed to connect to Qdrant: {0}")]
    Connection(String),

    /// Qdrant operation failed.
    #[error("Qdrant operation failed: {0}")]
    Operation(String),

    /// Collection does not exist.
    #[error("collection '{0}' does not exist")]
    CollectionNotFound(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
