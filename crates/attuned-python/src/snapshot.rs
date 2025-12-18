//! Python bindings for StateSnapshot and related types.

use attuned_core::{Source, StateSnapshot};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::collections::HashMap;

/// Source of a state snapshot - how the data was obtained.
///
/// - SELF_REPORT: User explicitly provided this state (highest trust)
/// - INFERRED: State was inferred from behavior or context
/// - MIXED: Combination of self-report and inference
#[pyclass(name = "Source", eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum PySource {
    /// User explicitly provided this state (highest trust).
    SelfReport = 0,
    /// State was inferred from behavior or context.
    Inferred = 1,
    /// Combination of self-report and inference.
    Mixed = 2,
}

#[pymethods]
impl PySource {
    /// Create a Source from a string value.
    ///
    /// Args:
    ///     value: One of "self_report", "inferred", or "mixed"
    ///
    /// Returns:
    ///     The corresponding Source variant
    #[staticmethod]
    fn from_str(value: &str) -> PyResult<Self> {
        match value {
            "self_report" => Ok(PySource::SelfReport),
            "inferred" => Ok(PySource::Inferred),
            "mixed" => Ok(PySource::Mixed),
            _ => Err(PyValueError::new_err(format!(
                "Invalid source: '{}'. Expected 'self_report', 'inferred', or 'mixed'",
                value
            ))),
        }
    }

    fn __str__(&self) -> &'static str {
        match self {
            PySource::SelfReport => "self_report",
            PySource::Inferred => "inferred",
            PySource::Mixed => "mixed",
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Source.{}",
            match self {
                PySource::SelfReport => "SelfReport",
                PySource::Inferred => "Inferred",
                PySource::Mixed => "Mixed",
            }
        )
    }
}

impl From<Source> for PySource {
    fn from(source: Source) -> Self {
        match source {
            Source::SelfReport => PySource::SelfReport,
            Source::Inferred => PySource::Inferred,
            Source::Mixed => PySource::Mixed,
            // Handle future variants gracefully
            _ => PySource::Mixed,
        }
    }
}

impl From<PySource> for Source {
    fn from(source: PySource) -> Self {
        match source {
            PySource::SelfReport => Source::SelfReport,
            PySource::Inferred => Source::Inferred,
            PySource::Mixed => Source::Mixed,
        }
    }
}

/// A snapshot of user state at a point in time.
///
/// State is represented as values along named axes, where each axis
/// is a value in [0.0, 1.0] with defined semantics.
///
/// Example:
///     >>> snapshot = StateSnapshot.builder() \\
///     ...     .user_id("user_123") \\
///     ...     .source(Source.SelfReport) \\
///     ...     .axis("warmth", 0.7) \\
///     ...     .axis("cognitive_load", 0.9) \\
///     ...     .build()
///     >>> print(snapshot.get_axis("warmth"))
///     0.7
#[pyclass(name = "StateSnapshot")]
#[derive(Clone)]
pub struct PyStateSnapshot {
    pub(crate) inner: StateSnapshot,
}

#[pymethods]
impl PyStateSnapshot {
    /// Create a new builder for constructing a StateSnapshot.
    ///
    /// Returns:
    ///     A StateSnapshotBuilder instance
    #[staticmethod]
    fn builder() -> PyStateSnapshotBuilder {
        PyStateSnapshotBuilder::new()
    }

    /// Get the user ID.
    #[getter]
    fn user_id(&self) -> &str {
        &self.inner.user_id
    }

    /// Get the timestamp (Unix milliseconds).
    #[getter]
    fn updated_at_unix_ms(&self) -> i64 {
        self.inner.updated_at_unix_ms
    }

    /// Get the source of this state.
    #[getter]
    fn source(&self) -> PySource {
        PySource::from(self.inner.source.clone())
    }

    /// Get the confidence level.
    #[getter]
    fn confidence(&self) -> f32 {
        self.inner.confidence
    }

    /// Get all axes as a dictionary.
    #[getter]
    fn axes(&self) -> HashMap<String, f32> {
        self.inner
            .axes
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    /// Get an axis value, returning the default (0.5) if not present.
    ///
    /// Args:
    ///     name: The axis name
    ///
    /// Returns:
    ///     The axis value, or 0.5 if not set
    fn get_axis(&self, name: &str) -> f32 {
        self.inner.get_axis(name)
    }

    /// Get an axis value if present.
    ///
    /// Args:
    ///     name: The axis name
    ///
    /// Returns:
    ///     The axis value, or None if not set
    fn get_axis_opt(&self, name: &str) -> Option<f32> {
        self.inner.get_axis_opt(name)
    }

    /// Serialize to JSON string.
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    /// Deserialize from JSON string.
    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        let inner: StateSnapshot = serde_json::from_str(json)
            .map_err(|e| PyValueError::new_err(format!("Deserialization error: {}", e)))?;
        Ok(PyStateSnapshot { inner })
    }

    fn __repr__(&self) -> String {
        format!(
            "StateSnapshot(user_id='{}', source={}, axes={{{}}})",
            &self.inner.user_id[..self.inner.user_id.len().min(8)],
            match self.inner.source {
                Source::SelfReport => "SelfReport",
                Source::Inferred => "Inferred",
                Source::Mixed => "Mixed",
                _ => "Unknown",
            },
            self.inner.axes.len()
        )
    }
}

impl From<StateSnapshot> for PyStateSnapshot {
    fn from(inner: StateSnapshot) -> Self {
        PyStateSnapshot { inner }
    }
}

/// Builder for constructing StateSnapshot instances.
///
/// Example:
///     >>> builder = StateSnapshot.builder()
///     >>> builder = builder.user_id("user_123")
///     >>> builder = builder.axis("warmth", 0.8)
///     >>> snapshot = builder.build()
#[pyclass(name = "StateSnapshotBuilder")]
#[derive(Clone)]
pub struct PyStateSnapshotBuilder {
    user_id: Option<String>,
    updated_at_unix_ms: Option<i64>,
    source: PySource,
    confidence: f32,
    axes: HashMap<String, f32>,
}

#[pymethods]
impl PyStateSnapshotBuilder {
    /// Create a new builder with defaults.
    #[new]
    fn new() -> Self {
        PyStateSnapshotBuilder {
            user_id: None,
            updated_at_unix_ms: None,
            source: PySource::SelfReport,
            confidence: 1.0,
            axes: HashMap::new(),
        }
    }

    /// Set the user ID.
    ///
    /// Args:
    ///     user_id: Unique identifier for the user
    ///
    /// Returns:
    ///     The builder for chaining
    fn user_id(mut slf: PyRefMut<'_, Self>, user_id: String) -> PyRefMut<'_, Self> {
        slf.user_id = Some(user_id);
        slf
    }

    /// Set the timestamp (Unix milliseconds).
    ///
    /// Args:
    ///     unix_ms: Unix timestamp in milliseconds
    ///
    /// Returns:
    ///     The builder for chaining
    fn updated_at(mut slf: PyRefMut<'_, Self>, unix_ms: i64) -> PyRefMut<'_, Self> {
        slf.updated_at_unix_ms = Some(unix_ms);
        slf
    }

    /// Set the source of this state.
    ///
    /// Args:
    ///     source: How the state was obtained
    ///
    /// Returns:
    ///     The builder for chaining
    fn source(mut slf: PyRefMut<'_, Self>, source: PySource) -> PyRefMut<'_, Self> {
        slf.source = source;
        slf
    }

    /// Set the confidence level.
    ///
    /// Args:
    ///     confidence: Confidence in accuracy [0.0, 1.0]
    ///
    /// Returns:
    ///     The builder for chaining
    fn confidence(mut slf: PyRefMut<'_, Self>, confidence: f32) -> PyRefMut<'_, Self> {
        slf.confidence = confidence;
        slf
    }

    /// Add an axis value.
    ///
    /// Args:
    ///     name: The axis name (must be a canonical axis)
    ///     value: The axis value [0.0, 1.0]
    ///
    /// Returns:
    ///     The builder for chaining
    fn axis(mut slf: PyRefMut<'_, Self>, name: String, value: f32) -> PyRefMut<'_, Self> {
        slf.axes.insert(name, value);
        slf
    }

    /// Add multiple axis values from a dictionary.
    ///
    /// Args:
    ///     axes: Dictionary of axis name to value
    ///
    /// Returns:
    ///     The builder for chaining
    fn axes(mut slf: PyRefMut<'_, Self>, axes: HashMap<String, f32>) -> PyRefMut<'_, Self> {
        slf.axes.extend(axes);
        slf
    }

    /// Build the StateSnapshot, validating all fields.
    ///
    /// Returns:
    ///     A validated StateSnapshot
    ///
    /// Raises:
    ///     ValueError: If validation fails (missing user_id, invalid axis values, etc.)
    fn build(&self) -> PyResult<PyStateSnapshot> {
        let user_id = self
            .user_id
            .clone()
            .ok_or_else(|| PyValueError::new_err("user_id is required"))?;

        let mut builder = StateSnapshot::builder()
            .user_id(user_id)
            .source(Source::from(self.source.clone()))
            .confidence(self.confidence);

        if let Some(ts) = self.updated_at_unix_ms {
            builder = builder.updated_at(ts);
        }

        for (name, value) in &self.axes {
            builder = builder.axis(name.clone(), *value);
        }

        builder
            .build()
            .map(PyStateSnapshot::from)
            .map_err(|e| PyValueError::new_err(format!("Validation error: {}", e)))
    }

    fn __repr__(&self) -> String {
        format!(
            "StateSnapshotBuilder(user_id={:?}, axes={})",
            self.user_id,
            self.axes.len()
        )
    }
}
