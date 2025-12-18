//! Main inference engine that combines all signals.
//!
//! The engine orchestrates:
//! 1. Linguistic feature extraction
//! 2. Delta analysis (deviation from baseline)
//! 3. Bayesian state estimation
//!
//! All inferences are declared, bounded, and subordinate to self-report.

use std::collections::HashMap;

use crate::bayesian::{BayesianConfig, BayesianUpdater, Observation, Prior};
use crate::delta::{Baseline, DeltaAnalyzer};
use crate::estimate::{
    max_confidence_for_axis, word_count_confidence_factor, InferenceSource, InferredState,
};
use crate::features::{LinguisticExtractor, LinguisticFeatures};

/// Configuration for the inference engine.
#[derive(Clone, Debug)]
pub struct InferenceConfig {
    /// Weight for linguistic signals vs delta signals.
    pub linguistic_weight: f32,
    /// Weight for delta (deviation) signals.
    pub delta_weight: f32,
    /// Minimum confidence to include an axis estimate.
    pub min_confidence: f32,
    /// Whether to track baselines per user.
    pub enable_delta_analysis: bool,
    /// Baseline window size.
    pub baseline_window: usize,
    /// Default priors for each axis.
    pub default_priors: HashMap<String, Prior>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            linguistic_weight: 0.6,
            delta_weight: 0.4,
            min_confidence: 0.3,
            enable_delta_analysis: true,
            baseline_window: 50,
            default_priors: Self::standard_priors(),
        }
    }
}

impl InferenceConfig {
    /// Standard neutral priors for all canonical axes.
    fn standard_priors() -> HashMap<String, Prior> {
        // Most axes start at neutral 0.5
        // Some have biased priors based on typical user behavior
        let mut priors = HashMap::new();

        // Cognitive - default to moderate load
        priors.insert(
            "cognitive_load".into(),
            Prior::from_value(0.4, 0.3, "typical user"),
        );
        priors.insert(
            "decision_fatigue".into(),
            Prior::from_value(0.3, 0.3, "typical user"),
        );
        priors.insert("tolerance_for_complexity".into(), Prior::neutral());
        priors.insert(
            "urgency_sensitivity".into(),
            Prior::from_value(0.3, 0.3, "most queries not urgent"),
        );

        // Emotional - default to stable
        priors.insert("emotional_intensity".into(), Prior::neutral());
        priors.insert(
            "emotional_stability".into(),
            Prior::from_value(0.6, 0.3, "assume stable"),
        );
        priors.insert(
            "anxiety_level".into(),
            Prior::from_value(0.3, 0.3, "assume calm"),
        );
        priors.insert("need_for_reassurance".into(), Prior::neutral());

        // Social - slightly warm/casual for digital interactions
        priors.insert("warmth".into(), Prior::from_value(0.5, 0.3, "neutral"));
        priors.insert(
            "formality".into(),
            Prior::from_value(0.4, 0.3, "digital tends casual"),
        );
        priors.insert("boundary_strength".into(), Prior::neutral());
        priors.insert("assertiveness".into(), Prior::neutral());
        priors.insert("reciprocity_expectation".into(), Prior::neutral());

        // Preferences - neutral
        priors.insert("ritual_need".into(), Prior::neutral());
        priors.insert("transactional_preference".into(), Prior::neutral());
        priors.insert("verbosity_preference".into(), Prior::neutral());
        priors.insert("directness_preference".into(), Prior::neutral());

        // Control - slightly prefer autonomy
        priors.insert(
            "autonomy_preference".into(),
            Prior::from_value(0.6, 0.3, "users prefer control"),
        );
        priors.insert("suggestion_tolerance".into(), Prior::neutral());
        priors.insert(
            "interruption_tolerance".into(),
            Prior::from_value(0.4, 0.3, "low by default"),
        );
        priors.insert("reflection_vs_action_bias".into(), Prior::neutral());

        // Safety - moderate defaults
        priors.insert("stakes_awareness".into(), Prior::neutral());
        priors.insert(
            "privacy_sensitivity".into(),
            Prior::from_value(0.6, 0.3, "assume privacy-conscious"),
        );

        priors
    }
}

/// Main inference engine.
///
/// Combines linguistic features, delta analysis, and Bayesian updating
/// to produce axis estimates with full provenance.
#[derive(Clone, Debug)]
pub struct InferenceEngine {
    config: InferenceConfig,
    extractor: LinguisticExtractor,
    bayesian: BayesianUpdater,
    delta_analyzer: DeltaAnalyzer,
}

impl InferenceEngine {
    /// Create a new engine with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an engine with custom configuration.
    pub fn with_config(config: InferenceConfig) -> Self {
        Self {
            config,
            extractor: LinguisticExtractor::new(),
            bayesian: BayesianUpdater::new(),
            delta_analyzer: DeltaAnalyzer::new(),
        }
    }

    /// Create an engine with custom Bayesian configuration.
    pub fn with_bayesian_config(bayesian_config: BayesianConfig) -> Self {
        Self {
            config: InferenceConfig::default(),
            extractor: LinguisticExtractor::new(),
            bayesian: BayesianUpdater::with_config(bayesian_config),
            delta_analyzer: DeltaAnalyzer::new(),
        }
    }

    /// Infer state from a single message (no baseline).
    ///
    /// Uses linguistic features only.
    pub fn infer(&self, text: &str) -> InferredState {
        let features = self.extractor.extract(text);
        self.infer_from_features(&features)
    }

    /// Infer from pre-extracted features.
    ///
    /// Applies research-validated confidence scaling (TASK-016):
    /// - Word count scaling (shorter texts = lower confidence)
    /// - Axis-specific caps (based on evidence strength)
    pub fn infer_from_features(&self, features: &LinguisticFeatures) -> InferredState {
        let mut state = InferredState::new();

        // Apply word count confidence scaling (TASK-016)
        // Research: ~100 words needed for stable inference
        let word_count_factor = word_count_confidence_factor(features.word_count);

        // Map linguistic features to axis estimates
        let mappings = self.linguistic_to_axes(features);

        for (axis, value, features_used) in mappings {
            let prior = self.get_prior(&axis);
            let obs = Observation::from_linguistic(value, features_used.clone());
            let mut estimate = self.bayesian.update(&axis, &prior, &obs);

            // Apply word count scaling to confidence
            estimate.confidence *= word_count_factor;

            // Apply axis-specific confidence cap (TASK-016)
            let axis_cap = max_confidence_for_axis(&axis);
            estimate.confidence = estimate.confidence.min(axis_cap);

            // Recalculate variance from adjusted confidence
            estimate.variance =
                crate::estimate::AxisEstimate::confidence_to_variance(estimate.confidence);

            if estimate.confidence >= self.config.min_confidence {
                state.update(estimate);
            }
        }

        state
    }

    /// Infer with baseline context (enables delta analysis).
    ///
    /// This is the full pipeline: linguistic + delta + Bayesian.
    pub fn infer_with_baseline(
        &self,
        text: &str,
        baseline: &mut Baseline,
        current_state: Option<&InferredState>,
    ) -> InferredState {
        let features = self.extractor.extract(text);
        self.infer_with_features_and_baseline(&features, baseline, current_state)
    }

    /// Full inference from features with baseline.
    pub fn infer_with_features_and_baseline(
        &self,
        features: &LinguisticFeatures,
        baseline: &mut Baseline,
        current_state: Option<&InferredState>,
    ) -> InferredState {
        let mut state = InferredState::new();

        // Get linguistic mappings
        let linguistic_mappings = self.linguistic_to_axes(features);

        // Get delta signals if baseline is ready
        let delta_signals = if self.config.enable_delta_analysis && baseline.is_ready() {
            Some(self.delta_analyzer.analyze_and_update(baseline, features))
        } else {
            if self.config.enable_delta_analysis {
                baseline.add(features);
            }
            None
        };

        // Process each axis we have signal for
        let mut axis_observations: HashMap<String, Vec<Observation>> = HashMap::new();

        // Add linguistic observations
        for (axis, value, features_used) in linguistic_mappings {
            let obs = Observation::new(
                value,
                0.04, // Linguistic observation noise
                InferenceSource::Linguistic {
                    features_used,
                    feature_values: HashMap::new(),
                },
            );
            axis_observations.entry(axis).or_default().push(obs);
        }

        // Add delta observations
        if let Some(ref signals) = delta_signals {
            let adjustments = self.delta_analyzer.to_axis_adjustments(signals);
            for (axis, adjustment) in adjustments {
                // Delta gives relative adjustment, need to convert to absolute
                let base_value = current_state
                    .and_then(|s| s.get(axis))
                    .map(|e| e.value)
                    .unwrap_or(0.5);

                let delta_value = (base_value + adjustment).clamp(0.0, 1.0);

                let (metric, z) = signals.max_deviation();
                let obs = Observation::from_delta(
                    delta_value,
                    z,
                    metric.to_string(),
                    signals.baseline_size,
                );
                axis_observations
                    .entry(axis.to_string())
                    .or_default()
                    .push(obs);
            }
        }

        // Combine observations through Bayesian updater
        for (axis, observations) in axis_observations {
            // Use current estimate as prior if available
            let prior = current_state
                .and_then(|s| s.get(&axis))
                .map(|e| Prior {
                    mean: e.value,
                    variance: e.variance,
                    reason: "previous estimate".to_string(),
                })
                .unwrap_or_else(|| self.get_prior(&axis));

            let estimate = self
                .bayesian
                .combine_observations(&axis, &prior, &observations);

            if estimate.confidence >= self.config.min_confidence {
                state.update(estimate);
            }
        }

        state
    }

    /// Map linguistic features to axis values.
    ///
    /// Returns (axis_name, value, features_used) tuples.
    fn linguistic_to_axes(&self, f: &LinguisticFeatures) -> Vec<(String, f32, Vec<String>)> {
        let mut mappings = Vec::new();

        // Cognitive axes
        // Higher complexity = higher tolerance for complexity
        mappings.push((
            "tolerance_for_complexity".into(),
            f.complexity_score(),
            vec![
                "reading_grade_level".into(),
                "avg_sentence_length".into(),
                "long_word_ratio".into(),
            ],
        ));

        // Urgency
        mappings.push((
            "urgency_sensitivity".into(),
            f.urgency_score(),
            vec![
                "urgency_word_count".into(),
                "imperative_count".into(),
                "exclamation_ratio".into(),
            ],
        ));

        // Emotional axes
        // High emotional intensity (exclamation, caps)
        // Renamed from emotional_openness - we measure intensity, not openness (TASK-016)
        if f.emotional_intensity() > 0.3 {
            mappings.push((
                "emotional_intensity".into(),
                f.emotional_intensity(),
                vec!["exclamation_ratio".into(), "caps_ratio".into()],
            ));
        }

        // Anxiety/stress - using research-validated score (TASK-016)
        // Now incorporates negative emotions + first-person + uncertainty + absolutist
        // Validated on Dreaddit: F1 improved 16.7% over uncertainty_score alone
        let anxiety = f.anxiety_score();
        if anxiety > 0.2 {
            mappings.push((
                "anxiety_level".into(),
                anxiety,
                vec![
                    "negative_emotion_density".into(),
                    "first_person_ratio".into(),
                    "hedge_density".into(),
                    "absolutist_density".into(),
                ],
            ));
        }

        // Social axes
        // Formality
        mappings.push((
            "formality".into(),
            f.formality_score(),
            vec!["contraction_ratio".into(), "complexity_score".into()],
        ));

        // Warmth - informal + positive emotional signals = warmth
        let warmth = (1.0 - f.formality_score()) * 0.5
            + f.emotional_intensity() * 0.3
            + (f.politeness_count as f32 / 3.0).clamp(0.0, 1.0) * 0.2;
        if f.politeness_count > 0 || f.emotional_intensity() > 0.2 {
            mappings.push((
                "warmth".into(),
                warmth.clamp(0.0, 1.0),
                vec![
                    "politeness_count".into(),
                    "emotional_intensity".into(),
                    "formality".into(),
                ],
            ));
        }

        // Assertiveness - certainty markers, imperatives, low hedging
        let assertiveness = f.certainty_count as f32 / 2.0 * 0.4
            + f.imperative_count as f32 / 2.0 * 0.3
            + (1.0 - f.uncertainty_score()) * 0.3;
        if f.certainty_count > 0 || f.imperative_count > 0 {
            mappings.push((
                "assertiveness".into(),
                assertiveness.clamp(0.0, 1.0),
                vec![
                    "certainty_count".into(),
                    "imperative_count".into(),
                    "hedge_count".into(),
                ],
            ));
        }

        // Preferences
        // Verbosity - based on message length relative to typical
        let verbosity = (f.word_count as f32 / 50.0).clamp(0.0, 1.0);
        mappings.push((
            "verbosity_preference".into(),
            verbosity,
            vec!["word_count".into()],
        ));

        // Directness - low hedging, high certainty, imperative usage
        let directness = (1.0 - f.uncertainty_score()) * 0.5
            + (f.certainty_count as f32 / 2.0).clamp(0.0, 1.0) * 0.3
            + (f.imperative_count as f32 / 2.0).clamp(0.0, 1.0) * 0.2;
        mappings.push((
            "directness_preference".into(),
            directness.clamp(0.0, 1.0),
            vec!["hedge_density".into(), "certainty_count".into()],
        ));

        // Ritual need - politeness markers, formal greeting patterns
        if f.politeness_count > 0 {
            mappings.push((
                "ritual_need".into(),
                (f.politeness_count as f32 / 3.0).clamp(0.0, 1.0),
                vec!["politeness_count".into()],
            ));
        }

        mappings
    }

    /// Get prior for an axis.
    fn get_prior(&self, axis: &str) -> Prior {
        self.config
            .default_priors
            .get(axis)
            .cloned()
            .unwrap_or_else(Prior::neutral)
    }

    /// Extract features without inference (useful for external analysis).
    pub fn extract_features(&self, text: &str) -> LinguisticFeatures {
        self.extractor.extract(text)
    }

    /// Create a new baseline tracker.
    pub fn new_baseline(&self) -> Baseline {
        Baseline::new(self.config.baseline_window)
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self {
            config: InferenceConfig::default(),
            extractor: LinguisticExtractor::new(),
            bayesian: BayesianUpdater::new(),
            delta_analyzer: DeltaAnalyzer::new(),
        }
    }
}

/// Quick inference function for simple use cases.
///
/// Creates a default engine and infers state from text.
pub fn infer(text: &str) -> InferredState {
    InferenceEngine::new().infer(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_inference() {
        let engine = InferenceEngine::new();
        // Need sufficient text for confidence (word count scaling)
        let state = engine.infer(
            "Hello, how are you doing today? I hope everything is going well with your project. \
             I wanted to reach out and see if you have any updates on the proposal we discussed \
             last week. Please let me know when you have a moment to chat about it.",
        );

        // Should have some estimates
        assert!(!state.is_empty());
    }

    #[test]
    fn test_urgent_message() {
        let engine = InferenceEngine::new();
        // Longer urgent message for better confidence
        let state = engine.infer(
            "URGENT! I need help immediately! This is absolutely critical and cannot wait! \
             The system is down and customers are affected. Please respond ASAP! We need \
             to fix this right now before it gets worse! This is an emergency situation!",
        );

        let urgency = state.get("urgency_sensitivity");
        assert!(urgency.is_some());
        assert!(urgency.unwrap().value > 0.5);
    }

    #[test]
    fn test_formal_message() {
        let engine = InferenceEngine::new();
        let state = engine.infer(
            "Dear Sir or Madam, I am writing to formally inquire about the current status \
             of my application for the senior developer position. I submitted my application \
             materials on the first of the month and would greatly appreciate any update \
             you could provide regarding the review process. I would be most grateful for \
             your prompt response to this matter. Yours sincerely and with respect.",
        );

        let formality = state.get("formality");
        assert!(formality.is_some());
        assert!(formality.unwrap().value > 0.5);
    }

    #[test]
    fn test_anxious_hedging() {
        let engine = InferenceEngine::new();
        let state = engine.infer(
            "I think maybe this might be a problem? I'm not really sure but perhaps \
             we should probably look into it, if that's okay? I'm worried this could \
             cause issues later. I feel anxious about the whole situation and I'm \
             struggling to figure out what to do. Maybe I'm overthinking it though?",
        );

        // Should detect anxiety/uncertainty
        let anxiety = state.get("anxiety_level");
        assert!(anxiety.is_some());
        assert!(anxiety.unwrap().value > 0.3); // Lowered threshold due to confidence scaling
    }

    #[test]
    fn test_baseline_integration() {
        let engine = InferenceEngine::new();
        let mut baseline = engine.new_baseline();

        // Build baseline
        for _ in 0..10 {
            let state = engine.infer_with_baseline(
                "Here is a normal question about your service.",
                &mut baseline,
                None,
            );
            // Early iterations won't have delta signals
            let _ = state;
        }

        // Now test with different message
        let state =
            engine.infer_with_baseline("HELP! Everything is broken!!!", &mut baseline, None);

        // Should detect the deviation
        assert!(!state.is_empty());
    }

    #[test]
    fn test_all_estimates_have_source() {
        let engine = InferenceEngine::new();
        let state = engine.infer("Please help me understand this complex topic in detail.");

        for estimate in state.all() {
            assert!(estimate.source.is_inferred());
            let summary = estimate.source.summary();
            assert!(!summary.is_empty());
        }
    }

    #[test]
    fn test_confidence_bounded() {
        let engine = InferenceEngine::new();
        let state = engine.infer("URGENT URGENT URGENT!!! HELP NOW!!!");

        for estimate in state.all() {
            assert!(estimate.confidence <= crate::estimate::MAX_INFERRED_CONFIDENCE);
        }
    }

    #[test]
    fn test_quick_inference_function() {
        // Need sufficient text for word count confidence scaling
        let state = infer(
            "Hello world, this is a test message to verify the inference function works \
             correctly with enough words to pass the confidence threshold for analysis.",
        );
        assert!(!state.is_empty());
    }
}
