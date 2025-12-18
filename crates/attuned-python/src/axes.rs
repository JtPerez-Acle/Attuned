//! Python bindings for axis definitions.

use attuned_core::axes::{AxisCategory, AxisDefinition};
use pyo3::prelude::*;

/// Semantic category for grouping related axes.
///
/// Categories:
///     - COGNITIVE: Mental processing and decision-making capacity
///     - EMOTIONAL: Emotional state and needs
///     - SOCIAL: Interpersonal interaction preferences
///     - PREFERENCES: Communication and format preferences
///     - CONTROL: Agency and autonomy preferences
///     - SAFETY: Risk and privacy concerns
#[pyclass(name = "AxisCategory", eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum PyAxisCategory {
    /// Mental processing and decision-making capacity.
    Cognitive = 0,
    /// Emotional state and needs.
    Emotional = 1,
    /// Interpersonal interaction preferences.
    Social = 2,
    /// Communication and format preferences.
    Preferences = 3,
    /// Agency and autonomy preferences.
    Control = 4,
    /// Risk and privacy concerns.
    Safety = 5,
}

#[pymethods]
impl PyAxisCategory {
    fn __str__(&self) -> &'static str {
        match self {
            PyAxisCategory::Cognitive => "cognitive",
            PyAxisCategory::Emotional => "emotional",
            PyAxisCategory::Social => "social",
            PyAxisCategory::Preferences => "preferences",
            PyAxisCategory::Control => "control",
            PyAxisCategory::Safety => "safety",
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "AxisCategory.{}",
            match self {
                PyAxisCategory::Cognitive => "Cognitive",
                PyAxisCategory::Emotional => "Emotional",
                PyAxisCategory::Social => "Social",
                PyAxisCategory::Preferences => "Preferences",
                PyAxisCategory::Control => "Control",
                PyAxisCategory::Safety => "Safety",
            }
        )
    }
}

impl From<AxisCategory> for PyAxisCategory {
    fn from(cat: AxisCategory) -> Self {
        match cat {
            AxisCategory::Cognitive => PyAxisCategory::Cognitive,
            AxisCategory::Emotional => PyAxisCategory::Emotional,
            AxisCategory::Social => PyAxisCategory::Social,
            AxisCategory::Preferences => PyAxisCategory::Preferences,
            AxisCategory::Control => PyAxisCategory::Control,
            AxisCategory::Safety => PyAxisCategory::Safety,
        }
    }
}

/// Complete definition of a single axis with governance metadata.
///
/// Each axis has not just what it *is*, but what it's *for* and what it
/// must *never* be used for. This turns philosophy into enforceable data.
///
/// Attributes:
///     name: Canonical name (immutable after v1.0)
///     category: Semantic category for grouping
///     description: Human-readable description
///     low_anchor: What a low value (near 0.0) represents
///     high_anchor: What a high value (near 1.0) represents
///     intent: Intended use cases for this axis
///     forbidden_uses: Explicitly forbidden uses (anti-patterns)
///     since: Version when this axis was introduced
///     deprecated: Deprecation information, if applicable
///
/// Example:
///     >>> axis = attuned.get_axis("cognitive_load")
///     >>> print(axis.name)  # "cognitive_load"
///     >>> print(axis.description)
///     >>> for use in axis.intent:
///     ...     print(f"  Intended: {use}")
///     >>> for forbidden in axis.forbidden_uses:
///     ...     print(f"  FORBIDDEN: {forbidden}")
#[pyclass(name = "AxisDefinition")]
#[derive(Clone)]
pub struct PyAxisDefinition {
    name: String,
    category: PyAxisCategory,
    description: String,
    low_anchor: String,
    high_anchor: String,
    intent: Vec<String>,
    forbidden_uses: Vec<String>,
    since: String,
    deprecated_since: Option<String>,
    deprecated_reason: Option<String>,
    deprecated_replacement: Option<String>,
}

#[pymethods]
impl PyAxisDefinition {
    /// Canonical axis name (immutable after v1.0).
    #[getter]
    fn name(&self) -> &str {
        &self.name
    }

    /// Semantic category for grouping.
    #[getter]
    fn category(&self) -> PyAxisCategory {
        self.category.clone()
    }

    /// Human-readable description of what this axis measures.
    #[getter]
    fn description(&self) -> &str {
        &self.description
    }

    /// What a low value (near 0.0) represents.
    #[getter]
    fn low_anchor(&self) -> &str {
        &self.low_anchor
    }

    /// What a high value (near 1.0) represents.
    #[getter]
    fn high_anchor(&self) -> &str {
        &self.high_anchor
    }

    /// Intended use cases for this axis.
    #[getter]
    fn intent(&self) -> Vec<String> {
        self.intent.clone()
    }

    /// Explicitly forbidden uses of this axis.
    ///
    /// These are anti-patterns that violate user agency or trust.
    /// Systems using Attuned MUST NOT use axis values for these purposes.
    #[getter]
    fn forbidden_uses(&self) -> Vec<String> {
        self.forbidden_uses.clone()
    }

    /// Version when this axis was introduced.
    #[getter]
    fn since(&self) -> &str {
        &self.since
    }

    /// Whether this axis is deprecated.
    #[getter]
    fn is_deprecated(&self) -> bool {
        self.deprecated_since.is_some()
    }

    /// Deprecation version, if deprecated.
    #[getter]
    fn deprecated_since(&self) -> Option<&str> {
        self.deprecated_since.as_deref()
    }

    /// Deprecation reason, if deprecated.
    #[getter]
    fn deprecated_reason(&self) -> Option<&str> {
        self.deprecated_reason.as_deref()
    }

    /// Replacement axis name, if deprecated.
    #[getter]
    fn deprecated_replacement(&self) -> Option<&str> {
        self.deprecated_replacement.as_deref()
    }

    /// Format the axis definition as a readable summary.
    fn format_summary(&self) -> String {
        let mut lines = vec![
            format!("# {}", self.name),
            format!("Category: {}", self.category.__str__()),
            String::new(),
            self.description.clone(),
            String::new(),
            format!("Low (0.0): {}", self.low_anchor),
            format!("High (1.0): {}", self.high_anchor),
        ];

        if !self.intent.is_empty() {
            lines.push(String::new());
            lines.push("## Intended Uses".to_string());
            for use_case in &self.intent {
                lines.push(format!("- {}", use_case));
            }
        }

        if !self.forbidden_uses.is_empty() {
            lines.push(String::new());
            lines.push("## FORBIDDEN Uses".to_string());
            for forbidden in &self.forbidden_uses {
                lines.push(format!("- {}", forbidden));
            }
        }

        lines.join("\n")
    }

    fn __repr__(&self) -> String {
        format!(
            "AxisDefinition(name='{}', category={}, intent={}, forbidden={})",
            self.name,
            self.category.__str__(),
            self.intent.len(),
            self.forbidden_uses.len()
        )
    }
}

impl From<&AxisDefinition> for PyAxisDefinition {
    fn from(axis: &AxisDefinition) -> Self {
        PyAxisDefinition {
            name: axis.name.to_string(),
            category: PyAxisCategory::from(axis.category),
            description: axis.description.to_string(),
            low_anchor: axis.low_anchor.to_string(),
            high_anchor: axis.high_anchor.to_string(),
            intent: axis.intent.iter().map(|s| s.to_string()).collect(),
            forbidden_uses: axis.forbidden_uses.iter().map(|s| s.to_string()).collect(),
            since: axis.since.to_string(),
            deprecated_since: axis.deprecated.as_ref().map(|d| d.since.to_string()),
            deprecated_reason: axis.deprecated.as_ref().map(|d| d.reason.to_string()),
            deprecated_replacement: axis
                .deprecated
                .as_ref()
                .and_then(|d| d.replacement.map(|s| s.to_string())),
        }
    }
}
