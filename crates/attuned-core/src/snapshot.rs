//! State snapshot representation.

use crate::axes::is_valid_axis_name;
use crate::error::ValidationError;
use crate::types::Source;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Maximum allowed length for user IDs.
pub const MAX_USER_ID_LENGTH: usize = 256;

/// A normalized axis value in the range [0.0, 1.0].
pub type AxisValue = f32;

/// A snapshot of user state at a point in time.
///
/// State is represented as values along named axes, where each axis
/// is a value in [0.0, 1.0] with defined semantics.
///
/// # PII Redaction
///
/// The `Debug` implementation redacts the `user_id` field to prevent
/// accidental logging of personally identifiable information. Only
/// the first 4 characters are shown, followed by `...`.
#[derive(Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Unique identifier for the user.
    pub user_id: String,

    /// Unix timestamp in milliseconds when this snapshot was created/updated.
    pub updated_at_unix_ms: i64,

    /// How this state was obtained.
    pub source: Source,

    /// Confidence in the accuracy of this snapshot [0.0, 1.0].
    pub confidence: f32,

    /// Axis values as a map from axis name to value.
    /// Values must be in [0.0, 1.0].
    pub axes: BTreeMap<String, AxisValue>,
}

/// Redact a user ID for safe logging.
///
/// Shows first 4 characters followed by "..." to allow correlation
/// while protecting full user identity.
fn redact_user_id(user_id: &str) -> String {
    if user_id.len() <= 4 {
        // Very short IDs are fully redacted
        "[redacted]".to_string()
    } else {
        format!("{}...", &user_id[..4])
    }
}

impl fmt::Debug for StateSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateSnapshot")
            .field("user_id", &redact_user_id(&self.user_id))
            .field("updated_at_unix_ms", &self.updated_at_unix_ms)
            .field("source", &self.source)
            .field("confidence", &self.confidence)
            .field("axes", &self.axes)
            .finish()
    }
}

impl StateSnapshot {
    /// Create a new builder for constructing a StateSnapshot.
    pub fn builder() -> StateSnapshotBuilder {
        StateSnapshotBuilder::new()
    }

    /// Validate the snapshot, returning errors if invalid.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate user_id
        validate_user_id(&self.user_id)?;

        // Validate confidence
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(ValidationError::ConfidenceOutOfRange {
                value: self.confidence,
            });
        }

        // Validate axes
        for (name, value) in &self.axes {
            if !is_valid_axis_name(name) {
                return Err(ValidationError::InvalidAxisName { axis: name.clone() });
            }
            if !(0.0..=1.0).contains(value) {
                return Err(ValidationError::AxisOutOfRange {
                    axis: name.clone(),
                    value: *value,
                });
            }
        }

        Ok(())
    }

    /// Get an axis value, returning the default (0.5) if not present.
    pub fn get_axis(&self, name: &str) -> AxisValue {
        *self.axes.get(name).unwrap_or(&0.5)
    }

    /// Get an axis value if present.
    pub fn get_axis_opt(&self, name: &str) -> Option<AxisValue> {
        self.axes.get(name).copied()
    }
}

impl Default for StateSnapshot {
    fn default() -> Self {
        Self {
            user_id: String::new(),
            updated_at_unix_ms: chrono::Utc::now().timestamp_millis(),
            source: Source::default(),
            confidence: 1.0,
            axes: BTreeMap::new(),
        }
    }
}

/// Builder for constructing StateSnapshot instances.
#[derive(Default)]
pub struct StateSnapshotBuilder {
    user_id: Option<String>,
    updated_at_unix_ms: Option<i64>,
    source: Source,
    confidence: f32,
    axes: BTreeMap<String, AxisValue>,
}

impl StateSnapshotBuilder {
    /// Create a new builder with defaults.
    pub fn new() -> Self {
        Self {
            user_id: None,
            updated_at_unix_ms: None,
            source: Source::SelfReport,
            confidence: 1.0,
            axes: BTreeMap::new(),
        }
    }

    /// Set the user ID.
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the timestamp (Unix milliseconds).
    pub fn updated_at(mut self, unix_ms: i64) -> Self {
        self.updated_at_unix_ms = Some(unix_ms);
        self
    }

    /// Set the source of this state.
    pub fn source(mut self, source: Source) -> Self {
        self.source = source;
        self
    }

    /// Set the confidence level.
    pub fn confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    /// Add an axis value.
    pub fn axis(mut self, name: impl Into<String>, value: AxisValue) -> Self {
        self.axes.insert(name.into(), value);
        self
    }

    /// Add multiple axis values from an iterator.
    pub fn axes(mut self, axes: impl IntoIterator<Item = (String, AxisValue)>) -> Self {
        self.axes.extend(axes);
        self
    }

    /// Build the StateSnapshot, validating all fields.
    pub fn build(self) -> Result<StateSnapshot, ValidationError> {
        let user_id = self.user_id.ok_or(ValidationError::MissingField {
            field: "user_id".to_string(),
        })?;

        let snapshot = StateSnapshot {
            user_id,
            updated_at_unix_ms: self
                .updated_at_unix_ms
                .unwrap_or_else(|| chrono::Utc::now().timestamp_millis()),
            source: self.source,
            confidence: self.confidence,
            axes: self.axes,
        };

        snapshot.validate()?;
        Ok(snapshot)
    }
}

/// Validate a user ID.
pub fn validate_user_id(user_id: &str) -> Result<(), ValidationError> {
    if user_id.is_empty() {
        return Err(ValidationError::EmptyUserId);
    }

    if user_id.len() > MAX_USER_ID_LENGTH {
        return Err(ValidationError::UserIdTooLong {
            max: MAX_USER_ID_LENGTH,
        });
    }

    if !user_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ValidationError::InvalidUserIdChars);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let snapshot = StateSnapshot::builder()
            .user_id("user_123")
            .axis("warmth", 0.7)
            .build()
            .unwrap();

        assert_eq!(snapshot.user_id, "user_123");
        assert_eq!(snapshot.get_axis("warmth"), 0.7);
        assert_eq!(snapshot.get_axis("unknown"), 0.5); // default
    }

    #[test]
    fn test_validation_axis_out_of_range() {
        let result = StateSnapshot::builder()
            .user_id("user_123")
            .axis("warmth", 1.5)
            .build();

        assert!(matches!(
            result,
            Err(ValidationError::AxisOutOfRange { axis, value })
            if axis == "warmth" && value == 1.5
        ));
    }

    #[test]
    fn test_validation_invalid_user_id() {
        let result = StateSnapshot::builder().user_id("user with spaces").build();

        assert!(matches!(result, Err(ValidationError::InvalidUserIdChars)));
    }

    #[test]
    fn test_validation_empty_user_id() {
        let result = StateSnapshot::builder().user_id("").build();

        assert!(matches!(result, Err(ValidationError::EmptyUserId)));
    }

    #[test]
    fn test_serialization() {
        let snapshot = StateSnapshot::builder()
            .user_id("u_123")
            .source(Source::SelfReport)
            .confidence(1.0)
            .axis("warmth", 0.6)
            .axis("formality", 0.3)
            .build()
            .unwrap();

        let json = serde_json::to_string(&snapshot).unwrap();
        let parsed: StateSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.user_id, snapshot.user_id);
        assert_eq!(parsed.get_axis("warmth"), 0.6);
    }

    #[test]
    fn test_debug_redacts_user_id() {
        let snapshot = StateSnapshot::builder()
            .user_id("user_123456789")
            .axis("warmth", 0.5)
            .build()
            .unwrap();

        let debug_output = format!("{:?}", snapshot);

        // Should contain redacted user_id
        assert!(debug_output.contains("user..."));
        // Should NOT contain the full user_id
        assert!(!debug_output.contains("user_123456789"));
    }

    #[test]
    fn test_debug_redacts_short_user_id() {
        let snapshot = StateSnapshot::builder()
            .user_id("ab12")
            .axis("warmth", 0.5)
            .build()
            .unwrap();

        let debug_output = format!("{:?}", snapshot);

        // Very short IDs should be fully redacted
        assert!(debug_output.contains("[redacted]"));
        assert!(!debug_output.contains("ab12"));
    }

    #[test]
    fn test_redact_user_id_function() {
        // Long user ID shows first 4 chars
        assert_eq!(redact_user_id("user_12345"), "user...");
        assert_eq!(redact_user_id("abcde"), "abcd...");

        // Short user IDs are fully redacted
        assert_eq!(redact_user_id("abc"), "[redacted]");
        assert_eq!(redact_user_id("abcd"), "[redacted]");
        assert_eq!(redact_user_id(""), "[redacted]");
    }

    // Property-based tests
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Strategy for generating valid user IDs
        fn valid_user_id() -> impl Strategy<Value = String> {
            "[a-zA-Z0-9_-]{1,64}".prop_filter("non-empty", |s| !s.is_empty())
        }

        // Strategy for generating valid axis values [0.0, 1.0]
        fn valid_axis_value() -> impl Strategy<Value = f32> {
            0.0f32..=1.0f32
        }

        // Strategy for generating valid axis names (must start with letter, can contain letters/digits/underscores, can't end with underscore)
        fn valid_axis_name() -> impl Strategy<Value = String> {
            "[a-z][a-z0-9_]{0,30}[a-z0-9]?"
                .prop_filter("must not end with underscore", |s| !s.ends_with('_'))
        }

        proptest! {
            #[test]
            fn prop_valid_axis_values_accepted(value in valid_axis_value()) {
                let result = StateSnapshot::builder()
                    .user_id("test_user")
                    .axis("test_axis", value)
                    .build();

                prop_assert!(result.is_ok());
                let snapshot = result.unwrap();
                let stored = snapshot.get_axis("test_axis");
                prop_assert!((stored - value).abs() < f32::EPSILON || stored == 0.5);
            }

            #[test]
            fn prop_invalid_axis_values_rejected(value in prop::num::f32::ANY) {
                prop_assume!(!(0.0..=1.0).contains(&value));
                prop_assume!(!value.is_nan());

                let result = StateSnapshot::builder()
                    .user_id("test_user")
                    .axis("test_axis", value)
                    .build();

                prop_assert!(result.is_err());
            }

            #[test]
            fn prop_valid_user_ids_accepted(user_id in valid_user_id()) {
                let result = StateSnapshot::builder()
                    .user_id(&user_id)
                    .build();

                prop_assert!(result.is_ok());
            }

            #[test]
            fn prop_snapshot_roundtrip_serialization(
                user_id in valid_user_id(),
                warmth in valid_axis_value(),
                formality in valid_axis_value(),
                confidence in valid_axis_value(),
            ) {
                let snapshot = StateSnapshot::builder()
                    .user_id(&user_id)
                    .confidence(confidence)
                    .axis("warmth", warmth)
                    .axis("formality", formality)
                    .build()
                    .unwrap();

                let json = serde_json::to_string(&snapshot).unwrap();
                let parsed: StateSnapshot = serde_json::from_str(&json).unwrap();

                prop_assert_eq!(&parsed.user_id, &snapshot.user_id);
                prop_assert!((parsed.confidence - snapshot.confidence).abs() < f32::EPSILON);
                prop_assert!((parsed.get_axis("warmth") - snapshot.get_axis("warmth")).abs() < f32::EPSILON);
            }

            #[test]
            fn prop_multiple_axes_preserved(
                axes in prop::collection::btree_map(
                    valid_axis_name(),
                    valid_axis_value(),
                    0..20
                )
            ) {
                let mut builder = StateSnapshot::builder().user_id("test_user");

                for (name, value) in &axes {
                    builder = builder.axis(name, *value);
                }

                let snapshot = builder.build().unwrap();

                for (name, expected_value) in &axes {
                    let actual = snapshot.get_axis(name);
                    prop_assert!(
                        (actual - expected_value).abs() < f32::EPSILON,
                        "Axis {} expected {} but got {}", name, expected_value, actual
                    );
                }
            }

            #[test]
            fn prop_get_axis_returns_default_for_unknown(
                axis_name in valid_axis_name()
            ) {
                let snapshot = StateSnapshot::builder()
                    .user_id("test_user")
                    .build()
                    .unwrap();

                let value = snapshot.get_axis(&axis_name);
                prop_assert_eq!(value, 0.5, "Unknown axis should return default 0.5");
            }
        }
    }
}
