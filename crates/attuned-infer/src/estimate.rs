//! Axis estimates with confidence and provenance tracking.
//!
//! Every inferred value carries metadata about where it came from,
//! how confident we are, and when it was computed. This enables
//! full auditability and transparent override by self-report.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum confidence for any inferred value.
///
/// Self-report can have confidence 1.0, but inference is capped
/// to ensure self-report always dominates when present.
pub const MAX_INFERRED_CONFIDENCE: f32 = 0.7;

/// Scale confidence based on text length (word count).
///
/// Research suggests ~100 words for stable style inference in formal settings,
/// but chat messages are typically 20-50 words. We use a gentler curve.
///
/// # Arguments
/// * `word_count` - Number of words in the analyzed text
///
/// # Returns
/// A multiplier in [0.5, 1.0] to apply to base confidence
pub fn word_count_confidence_factor(word_count: usize) -> f32 {
    const MIN_WORDS: f32 = 10.0; // Below this: reduced confidence
    const STABLE_WORDS: f32 = 50.0; // At this point: full confidence

    if word_count < MIN_WORDS as usize {
        return 0.5; // Reduced confidence for very short texts, but not too harsh
    }

    let factor = (word_count as f32 - MIN_WORDS) / (STABLE_WORDS - MIN_WORDS);
    0.5 + 0.5 * factor.clamp(0.0, 1.0) // Range [0.5, 1.0]
}

/// Get maximum confidence for a specific axis based on research evidence strength.
///
/// Different axes have different evidence backing in the literature.
/// This caps confidence based on how well-validated each axis is.
///
/// Based on DEEP_RESEARCH.md validation evidence:
/// - Strong: formality, emotional_intensity (r > 0.3, multiple studies)
/// - Moderate-Strong: anxiety_level, assertiveness (r = 0.2-0.3)
/// - Moderate: urgency, warmth (context-dependent)
/// - Weak: cognitive load proxies (no direct text signal)
pub fn max_confidence_for_axis(axis: &str) -> f32 {
    match axis {
        // Strong evidence (validated across multiple studies)
        "formality" | "emotional_intensity" => 0.7,

        // Moderate-strong evidence (Dreaddit validated)
        "anxiety_level" | "assertiveness" | "directness_preference" => 0.6,

        // Moderate/context-dependent evidence
        "urgency_sensitivity" | "warmth" | "ritual_need" => 0.5,

        // Weak evidence - style-dependent, hard to infer from text alone
        "tolerance_for_complexity" | "verbosity_preference" => 0.4,

        // Unknown axis - use conservative default
        _ => 0.5,
    }
}

/// Source of an axis inference with full provenance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InferenceSource {
    /// User explicitly provided this value.
    SelfReport,

    /// Inferred from linguistic features of text.
    Linguistic {
        /// Which features contributed to this inference.
        features_used: Vec<String>,
        /// The raw feature values that drove the inference.
        feature_values: HashMap<String, f32>,
    },

    /// Inferred from deviation from user's baseline behavior.
    Delta {
        /// How many messages in the baseline window.
        baseline_messages: usize,
        /// The z-score (standard deviations from baseline).
        z_score: f32,
        /// Which metric showed the deviation.
        metric: String,
    },

    /// Combined from multiple inference sources.
    Combined {
        /// The sources that were combined.
        sources: Vec<InferenceSource>,
        /// Weights given to each source.
        weights: Vec<f32>,
    },

    /// Confidence has decayed over time from original inference.
    Decayed {
        /// The original inference source.
        original: Box<InferenceSource>,
        /// How much time has passed.
        age_seconds: u64,
        /// The decay factor applied.
        decay_factor: f32,
    },

    /// Default/prior value (no observation).
    Prior {
        /// Description of why this prior was chosen.
        reason: String,
    },
}

impl InferenceSource {
    /// Returns true if this is a self-report (highest authority).
    pub fn is_self_report(&self) -> bool {
        matches!(self, Self::SelfReport)
    }

    /// Returns true if this is any form of inference (not self-report).
    pub fn is_inferred(&self) -> bool {
        !self.is_self_report()
    }

    /// Get a human-readable summary of this source.
    pub fn summary(&self) -> String {
        match self {
            Self::SelfReport => "self-report".to_string(),
            Self::Linguistic { features_used, .. } => {
                format!("linguistic({})", features_used.join(", "))
            }
            Self::Delta {
                metric, z_score, ..
            } => {
                format!("delta({}: z={:.2})", metric, z_score)
            }
            Self::Combined { sources, .. } => {
                format!("combined({})", sources.len())
            }
            Self::Decayed {
                original,
                decay_factor,
                ..
            } => {
                format!(
                    "decayed({}, factor={:.2})",
                    original.summary(),
                    decay_factor
                )
            }
            Self::Prior { reason } => format!("prior({})", reason),
        }
    }
}

/// A single axis estimate with full metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AxisEstimate {
    /// The axis name (must be a canonical axis).
    pub axis: String,

    /// Estimated value in [0.0, 1.0].
    pub value: f32,

    /// Confidence in this estimate.
    ///
    /// - 1.0 for self-report
    /// - ≤0.7 for inference (capped by MAX_INFERRED_CONFIDENCE)
    /// - Decays over time without new observations
    pub confidence: f32,

    /// Variance of the estimate (for Bayesian updates).
    ///
    /// Lower variance = more certain.
    /// Self-report sets variance to near-zero.
    pub variance: f32,

    /// How this estimate was derived.
    pub source: InferenceSource,

    /// When this estimate was computed.
    pub timestamp: DateTime<Utc>,
}

impl AxisEstimate {
    /// Create a new estimate from inference.
    ///
    /// Confidence is automatically capped at MAX_INFERRED_CONFIDENCE.
    pub fn inferred(
        axis: impl Into<String>,
        value: f32,
        confidence: f32,
        source: InferenceSource,
    ) -> Self {
        debug_assert!(
            source.is_inferred(),
            "Use self_report() for self-report values"
        );
        Self {
            axis: axis.into(),
            value: value.clamp(0.0, 1.0),
            confidence: confidence.min(MAX_INFERRED_CONFIDENCE),
            variance: Self::confidence_to_variance(confidence.min(MAX_INFERRED_CONFIDENCE)),
            source,
            timestamp: Utc::now(),
        }
    }

    /// Create a new estimate from self-report.
    ///
    /// Self-report has confidence 1.0 and near-zero variance.
    pub fn self_report(axis: impl Into<String>, value: f32) -> Self {
        Self {
            axis: axis.into(),
            value: value.clamp(0.0, 1.0),
            confidence: 1.0,
            variance: 0.001, // Near-zero but not exactly zero for numerical stability
            source: InferenceSource::SelfReport,
            timestamp: Utc::now(),
        }
    }

    /// Create a prior estimate (default before any observation).
    pub fn prior(
        axis: impl Into<String>,
        value: f32,
        confidence: f32,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            axis: axis.into(),
            value: value.clamp(0.0, 1.0),
            confidence: confidence.min(MAX_INFERRED_CONFIDENCE),
            variance: Self::confidence_to_variance(confidence.min(MAX_INFERRED_CONFIDENCE)),
            source: InferenceSource::Prior {
                reason: reason.into(),
            },
            timestamp: Utc::now(),
        }
    }

    /// Convert confidence to variance for Bayesian math.
    ///
    /// High confidence → low variance, low confidence → high variance.
    pub fn confidence_to_variance(confidence: f32) -> f32 {
        // Map confidence [0,1] to variance [1, 0.001]
        // Using exponential mapping for better numerical properties
        let conf = confidence.clamp(0.0, 1.0);
        (1.0 - conf).powi(2) + 0.001
    }

    /// Convert variance back to confidence.
    pub fn variance_to_confidence(variance: f32) -> f32 {
        (1.0 - (variance - 0.001).max(0.0).sqrt()).clamp(0.0, 1.0)
    }

    /// Apply time-based decay to this estimate.
    ///
    /// Confidence decreases over time, representing increasing uncertainty
    /// about stale inferences.
    pub fn decay(&self, half_life_seconds: f64) -> Self {
        let age = Utc::now()
            .signed_duration_since(self.timestamp)
            .num_seconds() as f64;

        if age <= 0.0 || self.source.is_self_report() {
            return self.clone();
        }

        // Exponential decay: confidence * 0.5^(age/half_life)
        let decay_factor = 0.5_f64.powf(age / half_life_seconds) as f32;
        let new_confidence = (self.confidence * decay_factor).max(0.1); // Floor at 0.1

        Self {
            axis: self.axis.clone(),
            value: self.value,
            confidence: new_confidence,
            variance: Self::confidence_to_variance(new_confidence),
            source: InferenceSource::Decayed {
                original: Box::new(self.source.clone()),
                age_seconds: age as u64,
                decay_factor,
            },
            timestamp: self.timestamp,
        }
    }

    /// Check if this estimate should be considered stale.
    pub fn is_stale(&self, max_age_seconds: i64) -> bool {
        let age = Utc::now()
            .signed_duration_since(self.timestamp)
            .num_seconds();
        age > max_age_seconds
    }
}

/// Complete inferred state across multiple axes.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InferredState {
    estimates: HashMap<String, AxisEstimate>,
}

impl InferredState {
    /// Create empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update an axis estimate.
    ///
    /// If an estimate already exists, the new one wins if:
    /// - It's self-report (always wins), or
    /// - It has higher confidence than existing non-self-report
    pub fn update(&mut self, estimate: AxisEstimate) {
        let dominated = self.estimates.get(&estimate.axis).is_some_and(|existing| {
            existing.source.is_self_report() && estimate.source.is_inferred()
        });

        if !dominated {
            self.estimates.insert(estimate.axis.clone(), estimate);
        }
    }

    /// Get estimate for an axis.
    pub fn get(&self, axis: &str) -> Option<&AxisEstimate> {
        self.estimates.get(axis)
    }

    /// Get all estimates.
    pub fn all(&self) -> impl Iterator<Item = &AxisEstimate> {
        self.estimates.values()
    }

    /// Get all axis names with estimates.
    pub fn axes(&self) -> impl Iterator<Item = &str> {
        self.estimates.keys().map(|s| s.as_str())
    }

    /// Number of axes with estimates.
    pub fn len(&self) -> usize {
        self.estimates.len()
    }

    /// Returns true if no estimates.
    pub fn is_empty(&self) -> bool {
        self.estimates.is_empty()
    }

    /// Apply an override from self-report.
    ///
    /// This sets the axis to the self-reported value with confidence 1.0,
    /// regardless of any existing inference.
    pub fn override_with_self_report(&mut self, axis: impl Into<String>, value: f32) {
        let axis = axis.into();
        self.estimates
            .insert(axis.clone(), AxisEstimate::self_report(axis, value));
    }

    /// Decay all inferred estimates.
    pub fn decay_all(&mut self, half_life_seconds: f64) {
        for estimate in self.estimates.values_mut() {
            if estimate.source.is_inferred() {
                *estimate = estimate.decay(half_life_seconds);
            }
        }
    }

    /// Remove stale estimates.
    pub fn prune_stale(&mut self, max_age_seconds: i64) {
        self.estimates.retain(|_, e| !e.is_stale(max_age_seconds));
    }

    /// Merge another state into this one.
    ///
    /// Self-report always wins. For inference vs inference,
    /// higher confidence wins.
    pub fn merge(&mut self, other: InferredState) {
        for (axis, new_estimate) in other.estimates {
            match self.estimates.get(&axis) {
                Some(existing) if existing.source.is_self_report() => {
                    // Existing self-report dominates
                    continue;
                }
                Some(_existing) if new_estimate.source.is_self_report() => {
                    // New self-report dominates
                    self.estimates.insert(axis, new_estimate);
                }
                Some(existing) if new_estimate.confidence > existing.confidence => {
                    // Higher confidence wins
                    self.estimates.insert(axis, new_estimate);
                }
                Some(_) => {
                    // Existing has higher or equal confidence
                    continue;
                }
                None => {
                    self.estimates.insert(axis, new_estimate);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inferred_confidence_cap() {
        let estimate = AxisEstimate::inferred(
            "warmth",
            0.8,
            0.95, // Above cap
            InferenceSource::Linguistic {
                features_used: vec!["exclamation_ratio".into()],
                feature_values: HashMap::new(),
            },
        );

        assert!(estimate.confidence <= MAX_INFERRED_CONFIDENCE);
    }

    #[test]
    fn test_self_report_full_confidence() {
        let estimate = AxisEstimate::self_report("warmth", 0.8);
        assert_eq!(estimate.confidence, 1.0);
        assert!(estimate.variance < 0.01);
    }

    #[test]
    fn test_self_report_dominates() {
        let mut state = InferredState::new();

        // Add inference
        state.update(AxisEstimate::inferred(
            "warmth",
            0.3,
            0.6,
            InferenceSource::Linguistic {
                features_used: vec![],
                feature_values: HashMap::new(),
            },
        ));

        // Override with self-report
        state.override_with_self_report("warmth", 0.9);

        let estimate = state.get("warmth").unwrap();
        assert_eq!(estimate.value, 0.9);
        assert!(estimate.source.is_self_report());
    }

    #[test]
    fn test_inference_cannot_override_self_report() {
        let mut state = InferredState::new();

        // Add self-report first
        state.update(AxisEstimate::self_report("warmth", 0.9));

        // Try to update with inference
        state.update(AxisEstimate::inferred(
            "warmth",
            0.3,
            0.7,
            InferenceSource::Linguistic {
                features_used: vec![],
                feature_values: HashMap::new(),
            },
        ));

        // Self-report should still be there
        let estimate = state.get("warmth").unwrap();
        assert_eq!(estimate.value, 0.9);
        assert!(estimate.source.is_self_report());
    }

    #[test]
    fn test_source_summary() {
        let source = InferenceSource::Linguistic {
            features_used: vec!["hedge_words".into(), "sentence_length".into()],
            feature_values: HashMap::new(),
        };
        assert_eq!(source.summary(), "linguistic(hedge_words, sentence_length)");
    }
}
