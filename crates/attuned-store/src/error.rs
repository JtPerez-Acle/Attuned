//! Storage error types.

use thiserror::Error;

/// Errors that can occur during storage operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StoreError {
    /// User was not found in the store.
    #[error("user not found: {user_id}")]
    UserNotFound {
        /// The user ID that was not found.
        user_id: String,
    },

    /// Internal storage error.
    #[error("storage error: {message}")]
    Internal {
        /// Error message.
        message: String,
        /// Underlying error source.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Connection error (for remote stores).
    #[error("connection error: {message}")]
    Connection {
        /// Error message.
        message: String,
        /// Underlying error source.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Validation error.
    #[error("validation error: {0}")]
    Validation(#[from] attuned_core::ValidationError),
}

impl StoreError {
    /// Create a new internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new internal error with a source.
    pub fn internal_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a new connection error.
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            source: None,
        }
    }
}
