//! HTTP error types.

use thiserror::Error;

/// HTTP server errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum HttpError {
    /// Failed to bind to address.
    #[error("failed to bind to {addr}: {message}")]
    Bind {
        /// The address that failed to bind.
        addr: String,
        /// Error message.
        message: String,
    },

    /// Request handling error.
    #[error("request error: {0}")]
    Request(String),

    /// Store error.
    #[error("store error: {0}")]
    Store(#[from] attuned_store::StoreError),
}
