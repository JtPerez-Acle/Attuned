//! Python bindings for inference types.

use attuned_infer::{
    AxisEstimate, InferenceEngine, InferenceSource, InferredState, LinguisticExtractor,
    LinguisticFeatures,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::collections::HashMap;

/// Source of an axis inference with full provenance.
///
/// This enum explains exactly how an estimate was derived, supporting
/// full auditability of the inference system.
///
/// Variants:
///     - SelfReport: User explicitly provided this value
///     - Linguistic: Inferred from text features
///     - Delta: Inferred from deviation from baseline
///     - Combined: Multiple sources combined
///     - Decayed: Confidence has decayed over time
///     - Prior: Default/prior value
#[pyclass(name = "InferenceSource")]
#[derive(Clone)]
pub struct PyInferenceSource {
    pub(crate) inner: InferenceSource,
}

#[pymethods]
impl PyInferenceSource {
    /// Check if this is a self-report source.
    fn is_self_report(&self) -> bool {
        self.inner.is_self_report()
    }

    /// Check if this is an inferred source (not self-report).
    fn is_inferred(&self) -> bool {
        self.inner.is_inferred()
    }

    /// Get a human-readable summary of this source.
    fn summary(&self) -> String {
        self.inner.summary()
    }

    /// Get the source type as a string.
    #[getter]
    fn source_type(&self) -> &'static str {
        match &self.inner {
            InferenceSource::SelfReport => "self_report",
            InferenceSource::Linguistic { .. } => "linguistic",
            InferenceSource::Delta { .. } => "delta",
            InferenceSource::Combined { .. } => "combined",
            InferenceSource::Decayed { .. } => "decayed",
            InferenceSource::Prior { .. } => "prior",
        }
    }

    /// Get linguistic features used (if source is Linguistic).
    #[getter]
    fn features_used(&self) -> Option<Vec<String>> {
        match &self.inner {
            InferenceSource::Linguistic { features_used, .. } => Some(features_used.clone()),
            _ => None,
        }
    }

    /// Get feature values (if source is Linguistic).
    #[getter]
    fn feature_values(&self) -> Option<HashMap<String, f32>> {
        match &self.inner {
            InferenceSource::Linguistic { feature_values, .. } => Some(feature_values.clone()),
            _ => None,
        }
    }

    /// Serialize to JSON string.
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    fn __repr__(&self) -> String {
        format!(
            "InferenceSource(type='{}', summary='{}')",
            self.source_type(),
            self.inner.summary()
        )
    }

    fn __str__(&self) -> String {
        self.inner.summary()
    }
}

impl From<InferenceSource> for PyInferenceSource {
    fn from(inner: InferenceSource) -> Self {
        PyInferenceSource { inner }
    }
}

/// A single axis estimate with full metadata.
///
/// Attributes:
///     axis: The axis name (canonical axis)
///     value: Estimated value in [0.0, 1.0]
///     confidence: Confidence in this estimate (0.0-1.0)
///     variance: Statistical variance (lower = more certain)
///     source: How this estimate was derived
///     timestamp: When this estimate was computed (ISO 8601)
///
/// Example:
///     >>> engine = InferenceEngine()
///     >>> state = engine.infer("I need this ASAP!!!")
///     >>> for estimate in state.all():
///     ...     print(f"{estimate.axis}: {estimate.value:.2f}")
///     ...     print(f"  Confidence: {estimate.confidence:.2f}")
///     ...     print(f"  Source: {estimate.source}")
#[pyclass(name = "AxisEstimate")]
#[derive(Clone)]
pub struct PyAxisEstimate {
    pub(crate) inner: AxisEstimate,
}

#[pymethods]
impl PyAxisEstimate {
    /// The axis name (canonical axis).
    #[getter]
    fn axis(&self) -> &str {
        &self.inner.axis
    }

    /// Estimated value in [0.0, 1.0].
    #[getter]
    fn value(&self) -> f32 {
        self.inner.value
    }

    /// Confidence in this estimate (0.0-1.0).
    ///
    /// - 1.0 for self-report
    /// - ≤0.7 for inference (capped)
    #[getter]
    fn confidence(&self) -> f32 {
        self.inner.confidence
    }

    /// Statistical variance (lower = more certain).
    #[getter]
    fn variance(&self) -> f32 {
        self.inner.variance
    }

    /// How this estimate was derived.
    #[getter]
    fn source(&self) -> PyInferenceSource {
        PyInferenceSource::from(self.inner.source.clone())
    }

    /// When this estimate was computed (ISO 8601 string).
    #[getter]
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    /// Serialize to JSON string.
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    fn __repr__(&self) -> String {
        format!(
            "AxisEstimate(axis='{}', value={:.3}, confidence={:.3}, source='{}')",
            self.inner.axis,
            self.inner.value,
            self.inner.confidence,
            self.inner.source.summary()
        )
    }
}

impl From<AxisEstimate> for PyAxisEstimate {
    fn from(inner: AxisEstimate) -> Self {
        PyAxisEstimate { inner }
    }
}

/// Complete inferred state across multiple axes.
///
/// This is the result of running inference on text. It contains
/// estimates for all axes that could be inferred with sufficient
/// confidence.
///
/// Example:
///     >>> engine = InferenceEngine()
///     >>> state = engine.infer("Please help me, I'm really stressed!")
///     >>>
///     >>> # Get a specific axis
///     >>> if (estimate := state.get("anxiety_level")):
///     ...     print(f"Anxiety: {estimate.value:.2f}")
///     >>>
///     >>> # Iterate all estimates
///     >>> for estimate in state.all():
///     ...     print(f"{estimate.axis}: {estimate.value:.2f}")
#[pyclass(name = "InferredState")]
#[derive(Clone)]
pub struct PyInferredState {
    pub(crate) inner: InferredState,
}

#[pymethods]
impl PyInferredState {
    /// Create empty state.
    #[new]
    fn new() -> Self {
        PyInferredState {
            inner: InferredState::new(),
        }
    }

    /// Get estimate for a specific axis.
    ///
    /// Args:
    ///     axis: The axis name to look up
    ///
    /// Returns:
    ///     AxisEstimate if found, None otherwise
    fn get(&self, axis: &str) -> Option<PyAxisEstimate> {
        self.inner
            .get(axis)
            .map(|e| PyAxisEstimate::from(e.clone()))
    }

    /// Get all estimates.
    ///
    /// Returns:
    ///     List of all AxisEstimate objects
    fn all(&self) -> Vec<PyAxisEstimate> {
        self.inner
            .all()
            .map(|e| PyAxisEstimate::from(e.clone()))
            .collect()
    }

    /// Get all axis names with estimates.
    ///
    /// Returns:
    ///     List of axis names
    fn axes(&self) -> Vec<String> {
        self.inner.axes().map(|s| s.to_string()).collect()
    }

    /// Number of axes with estimates.
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if no estimates.
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Apply an override from self-report.
    ///
    /// This sets the axis to the self-reported value with confidence 1.0,
    /// regardless of any existing inference.
    ///
    /// Args:
    ///     axis: The axis name
    ///     value: The self-reported value [0.0, 1.0]
    fn override_with_self_report(&mut self, axis: &str, value: f32) -> PyResult<()> {
        if !(0.0..=1.0).contains(&value) {
            return Err(PyValueError::new_err("value must be in [0.0, 1.0]"));
        }
        self.inner.override_with_self_report(axis, value);
        Ok(())
    }

    /// Convert to a dict of axis -> value.
    ///
    /// Returns:
    ///     Dict mapping axis names to estimated values
    fn to_dict(&self) -> HashMap<String, f32> {
        self.inner
            .all()
            .map(|e| (e.axis.clone(), e.value))
            .collect()
    }

    /// Convert to a dict including confidence.
    ///
    /// Returns:
    ///     Dict mapping axis names to {value, confidence, source} dicts
    fn to_dict_detailed(&self) -> HashMap<String, HashMap<String, PyObject>> {
        Python::with_gil(|py| {
            self.inner
                .all()
                .map(|e| {
                    let mut details: HashMap<String, PyObject> = HashMap::new();
                    details.insert(
                        "value".into(),
                        e.value.into_pyobject(py).unwrap().into_any().unbind(),
                    );
                    details.insert(
                        "confidence".into(),
                        e.confidence.into_pyobject(py).unwrap().into_any().unbind(),
                    );
                    details.insert(
                        "source".into(),
                        e.source
                            .summary()
                            .into_pyobject(py)
                            .unwrap()
                            .into_any()
                            .unbind(),
                    );
                    (e.axis.clone(), details)
                })
                .collect()
        })
    }

    /// Serialize to JSON string.
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    /// Deserialize from JSON string.
    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        let inner: InferredState = serde_json::from_str(json)
            .map_err(|e| PyValueError::new_err(format!("Deserialization error: {}", e)))?;
        Ok(PyInferredState { inner })
    }

    fn __repr__(&self) -> String {
        let axes: Vec<_> = self.inner.axes().collect();
        format!("InferredState(axes={})", axes.len())
    }

    fn __str__(&self) -> String {
        let items: Vec<String> = self
            .inner
            .all()
            .map(|e| format!("{}={:.2}", e.axis, e.value))
            .collect();
        format!("InferredState({})", items.join(", "))
    }
}

impl From<InferredState> for PyInferredState {
    fn from(inner: InferredState) -> Self {
        PyInferredState { inner }
    }
}

/// Linguistic features extracted from text.
///
/// These are the raw features used by the inference engine.
/// Useful for debugging, transparency, and custom inference logic.
///
/// Example:
///     >>> engine = InferenceEngine()
///     >>> features = engine.extract_features("I really need this ASAP!!!")
///     >>> print(f"Urgency words: {features.urgency_word_count}")
///     >>> print(f"Exclamation ratio: {features.exclamation_ratio:.2f}")
///     >>> print(f"Urgency score: {features.urgency_score():.2f}")
#[pyclass(name = "LinguisticFeatures")]
#[derive(Clone)]
pub struct PyLinguisticFeatures {
    pub(crate) inner: LinguisticFeatures,
}

#[pymethods]
impl PyLinguisticFeatures {
    // === Raw metrics ===

    /// Total character count.
    #[getter]
    fn char_count(&self) -> usize {
        self.inner.char_count
    }

    /// Total word count.
    #[getter]
    fn word_count(&self) -> usize {
        self.inner.word_count
    }

    /// Number of sentences.
    #[getter]
    fn sentence_count(&self) -> usize {
        self.inner.sentence_count
    }

    // === Complexity indicators ===

    /// Average word length in characters.
    #[getter]
    fn avg_word_length(&self) -> f32 {
        self.inner.avg_word_length
    }

    /// Average sentence length in words.
    #[getter]
    fn avg_sentence_length(&self) -> f32 {
        self.inner.avg_sentence_length
    }

    /// Ratio of long words (>6 chars) to total words.
    #[getter]
    fn long_word_ratio(&self) -> f32 {
        self.inner.long_word_ratio
    }

    /// Flesch-Kincaid grade level approximation.
    #[getter]
    fn reading_grade_level(&self) -> f32 {
        self.inner.reading_grade_level
    }

    // === Emotional indicators ===

    /// Ratio of exclamation marks to sentences.
    #[getter]
    fn exclamation_ratio(&self) -> f32 {
        self.inner.exclamation_ratio
    }

    /// Ratio of question marks to sentences.
    #[getter]
    fn question_ratio(&self) -> f32 {
        self.inner.question_ratio
    }

    /// Count of ALL CAPS words.
    #[getter]
    fn caps_word_count(&self) -> usize {
        self.inner.caps_word_count
    }

    /// Ratio of caps words to total words.
    #[getter]
    fn caps_ratio(&self) -> f32 {
        self.inner.caps_ratio
    }

    // === Hedge/uncertainty markers ===

    /// Count of hedge words ("maybe", "perhaps", "I think", etc.).
    #[getter]
    fn hedge_count(&self) -> usize {
        self.inner.hedge_count
    }

    /// Hedge words per sentence.
    #[getter]
    fn hedge_density(&self) -> f32 {
        self.inner.hedge_density
    }

    /// Count of certainty markers ("definitely", "absolutely", etc.).
    #[getter]
    fn certainty_count(&self) -> usize {
        self.inner.certainty_count
    }

    // === Anxiety/stress indicators ===

    /// Count of negative emotion words.
    #[getter]
    fn negative_emotion_count(&self) -> usize {
        self.inner.negative_emotion_count
    }

    /// Negative emotion words per sentence.
    #[getter]
    fn negative_emotion_density(&self) -> f32 {
        self.inner.negative_emotion_density
    }

    /// Count of absolutist words ("always", "never", etc.).
    #[getter]
    fn absolutist_count(&self) -> usize {
        self.inner.absolutist_count
    }

    /// Absolutist words per sentence.
    #[getter]
    fn absolutist_density(&self) -> f32 {
        self.inner.absolutist_density
    }

    // === Formality indicators ===

    /// Ratio of contractions to potential contraction sites.
    #[getter]
    fn contraction_ratio(&self) -> f32 {
        self.inner.contraction_ratio
    }

    /// Count of politeness markers ("please", "thank you", etc.).
    #[getter]
    fn politeness_count(&self) -> usize {
        self.inner.politeness_count
    }

    /// First-person pronoun ratio ("I", "me", "my").
    #[getter]
    fn first_person_ratio(&self) -> f32 {
        self.inner.first_person_ratio
    }

    // === Urgency indicators ===

    /// Count of urgency words ("urgent", "asap", "immediately").
    #[getter]
    fn urgency_word_count(&self) -> usize {
        self.inner.urgency_word_count
    }

    /// Count of imperative sentence starters.
    #[getter]
    fn imperative_count(&self) -> usize {
        self.inner.imperative_count
    }

    // === Computed scores ===

    /// Get a normalized complexity score [0, 1].
    fn complexity_score(&self) -> f32 {
        self.inner.complexity_score()
    }

    /// Get a normalized emotional intensity score [0, 1].
    fn emotional_intensity(&self) -> f32 {
        self.inner.emotional_intensity()
    }

    /// Get a normalized uncertainty score [0, 1].
    fn uncertainty_score(&self) -> f32 {
        self.inner.uncertainty_score()
    }

    /// Get a normalized anxiety/stress score [0, 1].
    ///
    /// Research-validated score combining negative emotions,
    /// first-person pronouns, and uncertainty markers.
    fn anxiety_score(&self) -> f32 {
        self.inner.anxiety_score()
    }

    /// Get a normalized urgency score [0, 1].
    fn urgency_score(&self) -> f32 {
        self.inner.urgency_score()
    }

    /// Get a normalized formality score [0, 1].
    ///
    /// Higher = more formal.
    fn formality_score(&self) -> f32 {
        self.inner.formality_score()
    }

    /// Convert to a dict of all features.
    fn to_dict(&self) -> HashMap<String, PyObject> {
        Python::with_gil(|py| {
            let mut d: HashMap<String, PyObject> = HashMap::new();
            d.insert(
                "char_count".into(),
                self.inner
                    .char_count
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "word_count".into(),
                self.inner
                    .word_count
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "sentence_count".into(),
                self.inner
                    .sentence_count
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "avg_word_length".into(),
                self.inner
                    .avg_word_length
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "avg_sentence_length".into(),
                self.inner
                    .avg_sentence_length
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "exclamation_ratio".into(),
                self.inner
                    .exclamation_ratio
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "question_ratio".into(),
                self.inner
                    .question_ratio
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "hedge_count".into(),
                self.inner
                    .hedge_count
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "hedge_density".into(),
                self.inner
                    .hedge_density
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "urgency_word_count".into(),
                self.inner
                    .urgency_word_count
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            // Computed scores
            d.insert(
                "complexity_score".into(),
                self.inner
                    .complexity_score()
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "emotional_intensity".into(),
                self.inner
                    .emotional_intensity()
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "urgency_score".into(),
                self.inner
                    .urgency_score()
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "formality_score".into(),
                self.inner
                    .formality_score()
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d.insert(
                "anxiety_score".into(),
                self.inner
                    .anxiety_score()
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind(),
            );
            d
        })
    }

    /// Serialize to JSON string.
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    fn __repr__(&self) -> String {
        format!(
            "LinguisticFeatures(words={}, sentences={}, urgency={:.2}, formality={:.2})",
            self.inner.word_count,
            self.inner.sentence_count,
            self.inner.urgency_score(),
            self.inner.formality_score()
        )
    }
}

impl From<LinguisticFeatures> for PyLinguisticFeatures {
    fn from(inner: LinguisticFeatures) -> Self {
        PyLinguisticFeatures { inner }
    }
}

/// Engine for inferring human state from text.
///
/// The InferenceEngine extracts linguistic features from text and maps
/// them to axis estimates. All inferences are:
///
/// - **Declared**: Every estimate includes its source
/// - **Bounded**: Inferred confidence is capped at 0.7
/// - **Subordinate**: Self-report always overrides inference
///
/// Example:
///     >>> engine = InferenceEngine()
///     >>>
///     >>> # Simple inference
///     >>> state = engine.infer("I need this done IMMEDIATELY!!!")
///     >>> print(state.get("urgency_sensitivity").value)  # High urgency
///     >>>
///     >>> # With feature extraction
///     >>> features = engine.extract_features("Please help when you can.")
///     >>> print(f"Formality: {features.formality_score():.2f}")
///
/// For debugging:
///     >>> state = engine.infer("I'm worried this might not work...")
///     >>> estimate = state.get("anxiety_level")
///     >>> print(f"Value: {estimate.value:.2f}")
///     >>> print(f"Confidence: {estimate.confidence:.2f}")
///     >>> print(f"Source: {estimate.source}")
///     >>> print(f"Features used: {estimate.source.features_used}")
#[pyclass(name = "InferenceEngine")]
#[derive(Clone)]
pub struct PyInferenceEngine {
    inner: InferenceEngine,
}

#[pymethods]
impl PyInferenceEngine {
    /// Create a new InferenceEngine with default configuration.
    #[new]
    fn new() -> Self {
        PyInferenceEngine {
            inner: InferenceEngine::new(),
        }
    }

    /// Infer state from text.
    ///
    /// Analyzes the text and returns estimates for all axes that
    /// can be inferred with sufficient confidence.
    ///
    /// Args:
    ///     text: The text to analyze
    ///
    /// Returns:
    ///     InferredState with axis estimates
    ///
    /// Example:
    ///     >>> state = engine.infer("Help! I need this urgently!")
    ///     >>> for estimate in state.all():
    ///     ...     print(f"{estimate.axis}: {estimate.value:.2f}")
    fn infer(&self, text: &str) -> PyInferredState {
        PyInferredState::from(self.inner.infer(text))
    }

    /// Extract linguistic features without inference.
    ///
    /// Useful for debugging or building custom inference logic.
    ///
    /// Args:
    ///     text: The text to analyze
    ///
    /// Returns:
    ///     LinguisticFeatures with all extracted metrics
    fn extract_features(&self, text: &str) -> PyLinguisticFeatures {
        PyLinguisticFeatures::from(self.inner.extract_features(text))
    }

    fn __repr__(&self) -> String {
        "InferenceEngine()".to_string()
    }
}

/// Quick inference function for simple use cases.
///
/// Creates a default engine and infers state from text.
/// For repeated inference, prefer creating an InferenceEngine once.
///
/// Args:
///     text: The text to analyze
///
/// Returns:
///     InferredState with axis estimates
///
/// Example:
///     >>> from attuned import infer
///     >>> state = infer("I'm feeling overwhelmed with all this work!")
///     >>> print(state.to_dict())
#[pyfunction(name = "infer")]
pub fn infer_text(text: &str) -> PyInferredState {
    PyInferredState::from(attuned_infer::infer(text))
}

/// Extract linguistic features from text.
///
/// Creates a default extractor and extracts features from text.
/// For repeated extraction, prefer creating an InferenceEngine.
///
/// Args:
///     text: The text to analyze
///
/// Returns:
///     LinguisticFeatures with all extracted metrics
///
/// Example:
///     >>> from attuned import extract_features
///     >>> features = extract_features("This is URGENT!!!")
///     >>> print(f"Urgency: {features.urgency_score():.2f}")
#[pyfunction]
pub fn extract_features(text: &str) -> PyLinguisticFeatures {
    let extractor = LinguisticExtractor::new();
    PyLinguisticFeatures::from(extractor.extract(text))
}
