//! Error types for Attuned.

use thiserror::Error;

/// Top-level error type for Attuned operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AttunedError {
    /// Validation error for input data.
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Storage-related error.
    #[error("storage error: {0}")]
    Storage(String),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Validation errors for state data.
#[derive(Debug, Error, PartialEq)]
#[non_exhaustive]
pub enum ValidationError {
    /// Axis value is outside the valid range [0.0, 1.0].
    #[error("axis '{axis}' has value {value} outside valid range [0.0, 1.0]")]
    AxisOutOfRange {
        /// The axis name.
        axis: String,
        /// The invalid value.
        value: f32,
    },

    /// Axis name contains invalid characters.
    #[error("axis name '{axis}' contains invalid characters (must be lowercase alphanumeric with underscores)")]
    InvalidAxisName {
        /// The invalid axis name.
        axis: String,
    },

    /// User ID is empty.
    #[error("user_id cannot be empty")]
    EmptyUserId,

    /// User ID exceeds maximum length.
    #[error("user_id exceeds maximum length of {max} characters")]
    UserIdTooLong {
        /// Maximum allowed length.
        max: usize,
    },

    /// User ID contains invalid characters.
    #[error("user_id contains invalid characters (must be alphanumeric, underscore, or hyphen)")]
    InvalidUserIdChars,

    /// Confidence value is outside valid range.
    #[error("confidence {value} is outside valid range [0.0, 1.0]")]
    ConfidenceOutOfRange {
        /// The invalid confidence value.
        value: f32,
    },

    /// Missing required field.
    #[error("missing required field: {field}")]
    MissingField {
        /// The missing field name.
        field: String,
    },
}
