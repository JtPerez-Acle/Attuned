//! Python bindings for translator types.

use crate::snapshot::PyStateSnapshot;
use attuned_core::{PromptContext, RuleTranslator, Thresholds, Translator, Verbosity};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Output verbosity level for LLM responses.
///
/// - LOW: Brief, concise responses
/// - MEDIUM: Balanced detail level (default)
/// - HIGH: Comprehensive, detailed responses
#[pyclass(name = "Verbosity", eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum PyVerbosity {
    /// Brief, concise responses.
    Low = 0,
    /// Balanced detail level.
    Medium = 1,
    /// Comprehensive, detailed responses.
    High = 2,
}

#[pymethods]
impl PyVerbosity {
    /// Create a Verbosity from a string value.
    #[staticmethod]
    fn from_str(value: &str) -> PyResult<Self> {
        match value.to_lowercase().as_str() {
            "low" => Ok(PyVerbosity::Low),
            "medium" => Ok(PyVerbosity::Medium),
            "high" => Ok(PyVerbosity::High),
            _ => Err(PyValueError::new_err(format!(
                "Invalid verbosity: '{}'. Expected 'low', 'medium', or 'high'",
                value
            ))),
        }
    }

    fn __str__(&self) -> &'static str {
        match self {
            PyVerbosity::Low => "low",
            PyVerbosity::Medium => "medium",
            PyVerbosity::High => "high",
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Verbosity.{}",
            match self {
                PyVerbosity::Low => "Low",
                PyVerbosity::Medium => "Medium",
                PyVerbosity::High => "High",
            }
        )
    }
}

impl From<Verbosity> for PyVerbosity {
    fn from(v: Verbosity) -> Self {
        match v {
            Verbosity::Low => PyVerbosity::Low,
            Verbosity::Medium => PyVerbosity::Medium,
            Verbosity::High => PyVerbosity::High,
        }
    }
}

impl From<PyVerbosity> for Verbosity {
    fn from(v: PyVerbosity) -> Self {
        match v {
            PyVerbosity::Low => Verbosity::Low,
            PyVerbosity::Medium => Verbosity::Medium,
            PyVerbosity::High => Verbosity::High,
        }
    }
}

/// Context produced by translating user state.
///
/// This is the output that should be injected into LLM system prompts
/// to condition interaction style.
///
/// Attributes:
///     guidelines: Behavioral guidelines for the LLM
///     tone: Suggested tone (e.g., "calm-neutral", "warm-formal")
///     verbosity: Desired response verbosity
///     flags: Active flags for special conditions
///
/// Example:
///     >>> translator = RuleTranslator()
///     >>> context = translator.to_prompt_context(snapshot)
///     >>> for guideline in context.guidelines:
///     ...     print(f"- {guideline}")
///     >>> print(f"Tone: {context.tone}")
#[pyclass(name = "PromptContext")]
#[derive(Clone)]
pub struct PyPromptContext {
    pub(crate) inner: PromptContext,
}

#[pymethods]
impl PyPromptContext {
    /// Behavioral guidelines for the LLM.
    #[getter]
    fn guidelines(&self) -> Vec<String> {
        self.inner.guidelines.clone()
    }

    /// Suggested tone (e.g., "calm-neutral", "warm-formal").
    #[getter]
    fn tone(&self) -> &str {
        &self.inner.tone
    }

    /// Desired response verbosity.
    #[getter]
    fn verbosity(&self) -> PyVerbosity {
        PyVerbosity::from(self.inner.verbosity.clone())
    }

    /// Active flags for special conditions.
    #[getter]
    fn flags(&self) -> Vec<String> {
        self.inner.flags.clone()
    }

    /// Format the context as a system prompt section.
    ///
    /// Returns:
    ///     A formatted string suitable for inclusion in LLM system prompts
    fn format_for_prompt(&self) -> String {
        let mut parts = vec![];

        if !self.inner.guidelines.is_empty() {
            parts.push("## Interaction Guidelines".to_string());
            for g in &self.inner.guidelines {
                parts.push(format!("- {}", g));
            }
            parts.push(String::new());
        }

        parts.push(format!("Tone: {}", self.inner.tone));
        parts.push(format!(
            "Verbosity: {}",
            match self.inner.verbosity {
                Verbosity::Low => "brief",
                Verbosity::Medium => "balanced",
                Verbosity::High => "comprehensive",
            }
        ));

        if !self.inner.flags.is_empty() {
            parts.push(format!("Active flags: {}", self.inner.flags.join(", ")));
        }

        parts.join("\n")
    }

    /// Serialize to JSON string.
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    /// Deserialize from JSON string.
    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        let inner: PromptContext = serde_json::from_str(json)
            .map_err(|e| PyValueError::new_err(format!("Deserialization error: {}", e)))?;
        Ok(PyPromptContext { inner })
    }

    fn __repr__(&self) -> String {
        format!(
            "PromptContext(tone='{}', verbosity={}, guidelines={}, flags={})",
            self.inner.tone,
            match self.inner.verbosity {
                Verbosity::Low => "Low",
                Verbosity::Medium => "Medium",
                Verbosity::High => "High",
            },
            self.inner.guidelines.len(),
            self.inner.flags.len()
        )
    }
}

impl From<PromptContext> for PyPromptContext {
    fn from(inner: PromptContext) -> Self {
        PyPromptContext { inner }
    }
}

/// Threshold configuration for rule-based translation.
///
/// Attributes:
///     hi: Values above this are considered "high" (default: 0.7)
///     lo: Values below this are considered "low" (default: 0.3)
#[pyclass(name = "Thresholds")]
#[derive(Clone, Default)]
pub struct PyThresholds {
    pub(crate) inner: Thresholds,
}

#[pymethods]
impl PyThresholds {
    /// Create new thresholds with custom values.
    ///
    /// Args:
    ///     hi: High threshold (default: 0.7)
    ///     lo: Low threshold (default: 0.3)
    #[new]
    #[pyo3(signature = (hi=0.7, lo=0.3))]
    fn new(hi: f32, lo: f32) -> PyResult<Self> {
        if hi <= lo {
            return Err(PyValueError::new_err("hi must be greater than lo"));
        }
        if !(0.0..=1.0).contains(&hi) || !(0.0..=1.0).contains(&lo) {
            return Err(PyValueError::new_err("thresholds must be in [0.0, 1.0]"));
        }
        Ok(PyThresholds {
            inner: Thresholds { hi, lo },
        })
    }

    /// High threshold - values above this are considered "high".
    #[getter]
    fn hi(&self) -> f32 {
        self.inner.hi
    }

    /// Low threshold - values below this are considered "low".
    #[getter]
    fn lo(&self) -> f32 {
        self.inner.lo
    }

    fn __repr__(&self) -> String {
        format!("Thresholds(hi={}, lo={})", self.inner.hi, self.inner.lo)
    }
}

/// Rule-based translator that converts state to context using threshold rules.
///
/// This is the reference implementation that provides full transparency into
/// how state values affect generated guidelines.
///
/// Example:
///     >>> translator = RuleTranslator()
///     >>> context = translator.to_prompt_context(snapshot)
///     >>> print(context.format_for_prompt())
///
/// With custom thresholds:
///     >>> translator = RuleTranslator(thresholds=Thresholds(hi=0.8, lo=0.2))
#[pyclass(name = "RuleTranslator")]
#[derive(Clone)]
pub struct PyRuleTranslator {
    inner: RuleTranslator,
}

#[pymethods]
impl PyRuleTranslator {
    /// Create a new RuleTranslator.
    ///
    /// Args:
    ///     thresholds: Optional custom thresholds (default: hi=0.7, lo=0.3)
    #[new]
    #[pyo3(signature = (thresholds=None))]
    fn new(thresholds: Option<PyThresholds>) -> Self {
        let inner = match thresholds {
            Some(t) => RuleTranslator::new(t.inner),
            None => RuleTranslator::default(),
        };
        PyRuleTranslator { inner }
    }

    /// Get the current thresholds.
    #[getter]
    fn thresholds(&self) -> PyThresholds {
        PyThresholds {
            inner: self.inner.thresholds.clone(),
        }
    }

    /// Translate a state snapshot into prompt context.
    ///
    /// Args:
    ///     snapshot: The user state to translate
    ///
    /// Returns:
    ///     A PromptContext with guidelines, tone, verbosity, and flags
    ///
    /// Example:
    ///     >>> snapshot = StateSnapshot.builder() \\
    ///     ...     .user_id("user_123") \\
    ///     ...     .axis("cognitive_load", 0.9) \\
    ///     ...     .axis("warmth", 0.8) \\
    ///     ...     .build()
    ///     >>> context = translator.to_prompt_context(snapshot)
    ///     >>> print(context.tone)  # "warm-casual"
    fn to_prompt_context(&self, snapshot: &PyStateSnapshot) -> PyPromptContext {
        let context = self.inner.to_prompt_context(&snapshot.inner);
        PyPromptContext::from(context)
    }

    fn __repr__(&self) -> String {
        format!(
            "RuleTranslator(thresholds=Thresholds(hi={}, lo={}))",
            self.inner.thresholds.hi, self.inner.thresholds.lo
        )
    }
}
