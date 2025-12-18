//! Delta analysis for detecting deviation from baseline behavior.
//!
//! Instead of asking "who is this person?", delta analysis asks
//! "how is this person different right now than usual?"
//!
//! This catches transient states (stress, urgency, frustration) without
//! needing to model stable personality traits.

use crate::features::LinguisticFeatures;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Signals derived from deviation analysis.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeltaSignals {
    /// Z-score for message length (positive = longer than usual).
    pub length_z: f32,
    /// Z-score for complexity (positive = more complex than usual).
    pub complexity_z: f32,
    /// Z-score for emotional intensity.
    pub emotional_z: f32,
    /// Z-score for formality.
    pub formality_z: f32,
    /// Z-score for urgency indicators.
    pub urgency_z: f32,
    /// Z-score for uncertainty/hedging.
    pub uncertainty_z: f32,
    /// Number of messages in the baseline.
    pub baseline_size: usize,
}

impl DeltaSignals {
    /// Check if any signal is significantly elevated (|z| > threshold).
    pub fn has_significant_deviation(&self, threshold: f32) -> bool {
        self.length_z.abs() > threshold
            || self.complexity_z.abs() > threshold
            || self.emotional_z.abs() > threshold
            || self.formality_z.abs() > threshold
            || self.urgency_z.abs() > threshold
            || self.uncertainty_z.abs() > threshold
    }

    /// Get the most extreme deviation.
    pub fn max_deviation(&self) -> (&'static str, f32) {
        let signals = [
            ("length", self.length_z),
            ("complexity", self.complexity_z),
            ("emotional", self.emotional_z),
            ("formality", self.formality_z),
            ("urgency", self.urgency_z),
            ("uncertainty", self.uncertainty_z),
        ];

        signals
            .into_iter()
            .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
            .unwrap_or(("none", 0.0))
    }
}

/// Running statistics for a single metric.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct RunningStat {
    count: usize,
    mean: f64,
    m2: f64, // For Welford's algorithm
}

impl RunningStat {
    fn new() -> Self {
        Self::default()
    }

    /// Update with a new value using Welford's online algorithm.
    fn update(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    /// Remove an old value (approximate - assumes FIFO).
    fn remove(&mut self, value: f64) {
        if self.count <= 1 {
            *self = Self::new();
            return;
        }

        // Inverse Welford (approximate)
        let delta = value - self.mean;
        self.mean = (self.mean * self.count as f64 - value) / (self.count - 1) as f64;
        let delta2 = value - self.mean;
        self.m2 -= delta * delta2;
        self.m2 = self.m2.max(0.0); // Numerical stability
        self.count -= 1;
    }

    /// Get current variance.
    fn variance(&self) -> f64 {
        if self.count < 2 {
            return 1.0; // High uncertainty with few samples
        }
        (self.m2 / (self.count - 1) as f64).max(0.001)
    }

    /// Get standard deviation.
    fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    /// Calculate z-score for a new value.
    fn z_score(&self, value: f64) -> f64 {
        if self.count < 3 {
            return 0.0; // Not enough data
        }
        (value - self.mean) / self.std_dev()
    }
}

/// A point in the baseline history.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BaselinePoint {
    length: f64,
    complexity: f64,
    emotional: f64,
    formality: f64,
    urgency: f64,
    uncertainty: f64,
}

impl From<&LinguisticFeatures> for BaselinePoint {
    fn from(f: &LinguisticFeatures) -> Self {
        Self {
            length: f.word_count as f64,
            complexity: f.complexity_score() as f64,
            emotional: f.emotional_intensity() as f64,
            formality: f.formality_score() as f64,
            urgency: f.urgency_score() as f64,
            uncertainty: f.uncertainty_score() as f64,
        }
    }
}

/// Baseline statistics for a user.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Baseline {
    /// Maximum number of messages to track.
    max_size: usize,
    /// Historical values (FIFO queue).
    history: VecDeque<BaselinePoint>,
    /// Running statistics for each metric.
    length_stat: RunningStat,
    complexity_stat: RunningStat,
    emotional_stat: RunningStat,
    formality_stat: RunningStat,
    urgency_stat: RunningStat,
    uncertainty_stat: RunningStat,
}

impl Baseline {
    /// Create a new baseline tracker.
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            history: VecDeque::with_capacity(max_size),
            length_stat: RunningStat::new(),
            complexity_stat: RunningStat::new(),
            emotional_stat: RunningStat::new(),
            formality_stat: RunningStat::new(),
            urgency_stat: RunningStat::new(),
            uncertainty_stat: RunningStat::new(),
        }
    }

    /// Add a new message to the baseline.
    pub fn add(&mut self, features: &LinguisticFeatures) {
        let point = BaselinePoint::from(features);

        // If at capacity, remove oldest
        if self.history.len() >= self.max_size {
            if let Some(old) = self.history.pop_front() {
                self.length_stat.remove(old.length);
                self.complexity_stat.remove(old.complexity);
                self.emotional_stat.remove(old.emotional);
                self.formality_stat.remove(old.formality);
                self.urgency_stat.remove(old.urgency);
                self.uncertainty_stat.remove(old.uncertainty);
            }
        }

        // Update running statistics
        self.length_stat.update(point.length);
        self.complexity_stat.update(point.complexity);
        self.emotional_stat.update(point.emotional);
        self.formality_stat.update(point.formality);
        self.urgency_stat.update(point.urgency);
        self.uncertainty_stat.update(point.uncertainty);

        self.history.push_back(point);
    }

    /// Number of messages in the baseline.
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Check if baseline is empty.
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Check if we have enough data for meaningful analysis.
    pub fn is_ready(&self) -> bool {
        self.history.len() >= 5 // Need at least 5 messages
    }

    /// Get baseline mean for message length.
    pub fn mean_length(&self) -> f64 {
        self.length_stat.mean
    }

    /// Get baseline mean for complexity.
    pub fn mean_complexity(&self) -> f64 {
        self.complexity_stat.mean
    }
}

impl Default for Baseline {
    fn default() -> Self {
        Self::new(50) // Default to 50-message window
    }
}

/// Delta analyzer for detecting deviations from baseline.
#[derive(Clone, Debug, Default)]
pub struct DeltaAnalyzer {
    /// Z-score threshold for significance.
    pub significance_threshold: f32,
}

impl DeltaAnalyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self {
            significance_threshold: 1.5, // ~13% of distribution
        }
    }

    /// Create analyzer with custom threshold.
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            significance_threshold: threshold,
        }
    }

    /// Analyze a message against the baseline.
    ///
    /// Returns delta signals (z-scores) for each metric.
    /// Positive z-score = higher than usual, negative = lower than usual.
    pub fn analyze(&self, baseline: &Baseline, features: &LinguisticFeatures) -> DeltaSignals {
        if !baseline.is_ready() {
            return DeltaSignals {
                baseline_size: baseline.len(),
                ..Default::default()
            };
        }

        let point = BaselinePoint::from(features);

        DeltaSignals {
            length_z: baseline.length_stat.z_score(point.length) as f32,
            complexity_z: baseline.complexity_stat.z_score(point.complexity) as f32,
            emotional_z: baseline.emotional_stat.z_score(point.emotional) as f32,
            formality_z: baseline.formality_stat.z_score(point.formality) as f32,
            urgency_z: baseline.urgency_stat.z_score(point.urgency) as f32,
            uncertainty_z: baseline.uncertainty_stat.z_score(point.uncertainty) as f32,
            baseline_size: baseline.len(),
        }
    }

    /// Analyze and update baseline in one step.
    ///
    /// Analyzes against current baseline, then adds to baseline.
    pub fn analyze_and_update(
        &self,
        baseline: &mut Baseline,
        features: &LinguisticFeatures,
    ) -> DeltaSignals {
        let signals = self.analyze(baseline, features);
        baseline.add(features);
        signals
    }

    /// Map delta signals to axis adjustments.
    ///
    /// Returns (axis_name, adjustment) pairs where adjustment is in [-0.3, 0.3].
    pub fn to_axis_adjustments(&self, signals: &DeltaSignals) -> Vec<(&'static str, f32)> {
        let mut adjustments = Vec::new();

        // Helper to convert z-score to bounded adjustment
        let z_to_adj = |z: f32| -> f32 {
            // Sigmoid-like mapping: z=2 → ~0.2, z=3 → ~0.27
            (z / (1.0 + z.abs()) * 0.3).clamp(-0.3, 0.3)
        };

        // Length deviation → cognitive_load (shorter = higher load)
        if signals.length_z.abs() > self.significance_threshold {
            adjustments.push(("cognitive_load", z_to_adj(-signals.length_z)));
        }

        // Complexity deviation
        if signals.complexity_z.abs() > self.significance_threshold {
            adjustments.push(("tolerance_for_complexity", z_to_adj(signals.complexity_z)));
        }

        // Emotional intensity deviation
        if signals.emotional_z.abs() > self.significance_threshold {
            adjustments.push(("emotional_intensity", z_to_adj(signals.emotional_z)));
            // High emotional intensity often correlates with low stability
            adjustments.push(("emotional_stability", z_to_adj(-signals.emotional_z * 0.5)));
        }

        // Formality deviation
        if signals.formality_z.abs() > self.significance_threshold {
            adjustments.push(("formality", z_to_adj(signals.formality_z)));
        }

        // Urgency deviation
        if signals.urgency_z.abs() > self.significance_threshold {
            adjustments.push(("urgency_sensitivity", z_to_adj(signals.urgency_z)));
        }

        // Uncertainty deviation
        if signals.uncertainty_z.abs() > self.significance_threshold {
            adjustments.push(("anxiety_level", z_to_adj(signals.uncertainty_z)));
            adjustments.push(("assertiveness", z_to_adj(-signals.uncertainty_z)));
        }

        adjustments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::LinguisticExtractor;

    fn make_features(text: &str) -> LinguisticFeatures {
        LinguisticExtractor::new().extract(text)
    }

    #[test]
    fn test_baseline_building() {
        let mut baseline = Baseline::new(10);

        for _ in 0..5 {
            baseline.add(&make_features("This is a normal message."));
        }

        assert_eq!(baseline.len(), 5);
        assert!(baseline.is_ready());
    }

    #[test]
    fn test_baseline_fifo() {
        let mut baseline = Baseline::new(3);

        baseline.add(&make_features("Message one."));
        baseline.add(&make_features("Message two."));
        baseline.add(&make_features("Message three."));
        baseline.add(&make_features("Message four.")); // Should evict first

        assert_eq!(baseline.len(), 3);
    }

    #[test]
    fn test_detect_urgency_spike() {
        let mut baseline = Baseline::default();
        let analyzer = DeltaAnalyzer::new();

        // Build baseline with calm messages
        for _ in 0..10 {
            baseline.add(&make_features(
                "Here is my regular question about the product.",
            ));
        }

        // Analyze urgent message
        let urgent = make_features("URGENT!!! I need help RIGHT NOW! This is critical!!!");
        let signals = analyzer.analyze(&baseline, &urgent);

        // Should detect elevated urgency and emotional intensity
        assert!(signals.urgency_z > 1.0);
        assert!(signals.emotional_z > 1.0);
    }

    #[test]
    fn test_detect_terseness() {
        let mut baseline = Baseline::default();
        let analyzer = DeltaAnalyzer::new();

        // Build baseline with longer messages
        for _ in 0..10 {
            baseline.add(&make_features(
                "I wanted to ask about the features of your product and understand \
                 how it might help with my specific use case in data processing.",
            ));
        }

        // Analyze terse message
        let terse = make_features("ok");
        let signals = analyzer.analyze(&baseline, &terse);

        // Should detect shortened length
        assert!(signals.length_z < -1.0);
    }

    #[test]
    fn test_not_enough_baseline() {
        let baseline = Baseline::default();
        let analyzer = DeltaAnalyzer::new();

        let features = make_features("Hello world");
        let signals = analyzer.analyze(&baseline, &features);

        // Should return zeros when not enough data
        assert_eq!(signals.length_z, 0.0);
        assert!(!baseline.is_ready());
    }

    #[test]
    fn test_axis_adjustments() {
        let analyzer = DeltaAnalyzer::new();

        let signals = DeltaSignals {
            urgency_z: 2.5,
            emotional_z: 0.5, // Below threshold
            ..Default::default()
        };

        let adjustments = analyzer.to_axis_adjustments(&signals);

        // Should have urgency adjustment
        assert!(adjustments
            .iter()
            .any(|(axis, _)| *axis == "urgency_sensitivity"));
        // Should not have emotional adjustment (below threshold)
        assert!(!adjustments
            .iter()
            .any(|(axis, _)| *axis == "emotional_intensity"));
    }

    #[test]
    fn test_max_deviation() {
        let signals = DeltaSignals {
            length_z: 0.5,
            urgency_z: 3.0,
            emotional_z: -2.0,
            ..Default::default()
        };

        let (metric, z) = signals.max_deviation();
        assert_eq!(metric, "urgency");
        assert_eq!(z, 3.0);
    }
}
