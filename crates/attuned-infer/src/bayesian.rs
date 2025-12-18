//! Bayesian state estimation for axis values.
//!
//! This module implements principled uncertainty tracking using
//! Bayesian updating. Each axis is modeled as a latent variable
//! with a posterior distribution that updates with each observation.
//!
//! Key properties:
//! - Single observations can't swing estimates too wildly
//! - Uncertainty is explicit and quantified
//! - Self-report sets variance to near-zero (nuclear override)
//! - Old inferences decay naturally

use crate::estimate::{AxisEstimate, InferenceSource, MAX_INFERRED_CONFIDENCE};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Prior distribution for an axis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prior {
    /// Prior mean (expected value before observation).
    pub mean: f32,
    /// Prior variance (uncertainty before observation).
    pub variance: f32,
    /// Human-readable reason for this prior.
    pub reason: String,
}

impl Prior {
    /// Create a neutral prior (0.5 with high uncertainty).
    pub fn neutral() -> Self {
        Self {
            mean: 0.5,
            variance: 0.25, // High uncertainty
            reason: "neutral default".to_string(),
        }
    }

    /// Create a prior from a specific value with given confidence.
    pub fn from_value(value: f32, confidence: f32, reason: impl Into<String>) -> Self {
        // Map confidence to variance: high confidence → low variance
        let variance = (1.0 - confidence).powi(2) * 0.25 + 0.01;
        Self {
            mean: value.clamp(0.0, 1.0),
            variance,
            reason: reason.into(),
        }
    }

    /// Create a prior biased toward low values.
    pub fn biased_low(reason: impl Into<String>) -> Self {
        Self {
            mean: 0.3,
            variance: 0.15,
            reason: reason.into(),
        }
    }

    /// Create a prior biased toward high values.
    pub fn biased_high(reason: impl Into<String>) -> Self {
        Self {
            mean: 0.7,
            variance: 0.15,
            reason: reason.into(),
        }
    }
}

impl Default for Prior {
    fn default() -> Self {
        Self::neutral()
    }
}

/// An observation of an axis value.
#[derive(Clone, Debug)]
pub struct Observation {
    /// Observed value.
    pub value: f32,
    /// Observation noise (measurement uncertainty).
    pub noise_variance: f32,
    /// Source of this observation.
    pub source: InferenceSource,
    /// When this observation was made.
    pub timestamp: DateTime<Utc>,
}

impl Observation {
    /// Create a new observation.
    pub fn new(value: f32, noise_variance: f32, source: InferenceSource) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            noise_variance: noise_variance.max(0.001),
            source,
            timestamp: Utc::now(),
        }
    }

    /// Create an observation from linguistic inference.
    ///
    /// Linguistic observations have moderate noise.
    pub fn from_linguistic(value: f32, features_used: Vec<String>) -> Self {
        Self::new(
            value,
            0.04, // Moderate noise
            InferenceSource::Linguistic {
                features_used,
                feature_values: std::collections::HashMap::new(),
            },
        )
    }

    /// Create an observation from delta analysis.
    pub fn from_delta(value: f32, z_score: f32, metric: String, baseline_messages: usize) -> Self {
        // Higher |z_score| = more confidence = less noise
        let noise = (0.1 / (1.0 + z_score.abs())).max(0.02);
        Self::new(
            value,
            noise,
            InferenceSource::Delta {
                baseline_messages,
                z_score,
                metric,
            },
        )
    }

    /// Create a self-report observation (very low noise).
    pub fn from_self_report(value: f32) -> Self {
        Self::new(value, 0.001, InferenceSource::SelfReport)
    }
}

/// Configuration for the Bayesian updater.
#[derive(Clone, Debug)]
pub struct BayesianConfig {
    /// Maximum update per observation (prevents wild swings).
    pub max_update: f32,
    /// Minimum variance (prevents overconfidence).
    pub min_variance: f32,
    /// Variance added per second without observation (uncertainty grows).
    pub variance_growth_rate: f32,
    /// Maximum confidence for inferred values.
    pub max_inferred_confidence: f32,
}

impl Default for BayesianConfig {
    fn default() -> Self {
        Self {
            max_update: 0.3,              // Max 0.3 shift per observation
            min_variance: 0.001,          // Never fully certain
            variance_growth_rate: 0.0001, // Slow uncertainty growth
            max_inferred_confidence: MAX_INFERRED_CONFIDENCE,
        }
    }
}

/// Bayesian state updater for a single axis.
///
/// Maintains posterior distribution (mean, variance) and updates
/// with each observation using standard Bayesian updating.
#[derive(Clone, Debug, Default)]
pub struct BayesianUpdater {
    config: BayesianConfig,
}

impl BayesianUpdater {
    /// Create a new updater with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an updater with custom configuration.
    pub fn with_config(config: BayesianConfig) -> Self {
        Self { config }
    }

    /// Perform a Bayesian update given prior and observation.
    ///
    /// Returns the posterior distribution as an AxisEstimate.
    pub fn update(&self, axis: &str, prior: &Prior, observation: &Observation) -> AxisEstimate {
        // Special case: self-report is authoritative
        if observation.source.is_self_report() {
            return AxisEstimate::self_report(axis, observation.value);
        }

        // Standard Bayesian update for Gaussian:
        // posterior_var = 1 / (1/prior_var + 1/obs_var)
        // posterior_mean = posterior_var * (prior_mean/prior_var + obs_value/obs_var)

        let prior_precision = 1.0 / prior.variance;
        let obs_precision = 1.0 / observation.noise_variance;

        let posterior_precision = prior_precision + obs_precision;
        let posterior_variance = (1.0 / posterior_precision).max(self.config.min_variance);

        let posterior_mean =
            posterior_variance * (prior.mean * prior_precision + observation.value * obs_precision);

        // Apply max update constraint
        let clamped_mean = if (posterior_mean - prior.mean).abs() > self.config.max_update {
            if posterior_mean > prior.mean {
                prior.mean + self.config.max_update
            } else {
                prior.mean - self.config.max_update
            }
        } else {
            posterior_mean
        };

        // Clamp to valid range
        let final_mean = clamped_mean.clamp(0.0, 1.0);

        // Convert variance to confidence
        let confidence = AxisEstimate::variance_to_confidence(posterior_variance)
            .min(self.config.max_inferred_confidence);

        AxisEstimate {
            axis: axis.to_string(),
            value: final_mean,
            confidence,
            variance: posterior_variance,
            source: observation.source.clone(),
            timestamp: observation.timestamp,
        }
    }

    /// Update an existing estimate with a new observation.
    ///
    /// The existing estimate serves as the prior.
    pub fn update_estimate(
        &self,
        existing: &AxisEstimate,
        observation: &Observation,
    ) -> AxisEstimate {
        // Special case: self-report always wins
        if observation.source.is_self_report() {
            return AxisEstimate::self_report(&existing.axis, observation.value);
        }

        // Can't override self-report with inference
        if existing.source.is_self_report() {
            return existing.clone();
        }

        // Use existing estimate as prior
        let prior = Prior {
            mean: existing.value,
            variance: existing.variance,
            reason: "previous estimate".to_string(),
        };

        self.update(&existing.axis, &prior, observation)
    }

    /// Grow uncertainty over time without observations.
    ///
    /// Variance increases linearly, representing growing uncertainty
    /// about stale estimates.
    pub fn grow_uncertainty(&self, estimate: &AxisEstimate, elapsed_seconds: f64) -> AxisEstimate {
        if estimate.source.is_self_report() {
            // Self-report doesn't decay (user's stated preference persists)
            return estimate.clone();
        }

        let growth = self.config.variance_growth_rate * elapsed_seconds as f32;
        let new_variance = (estimate.variance + growth).min(0.25); // Cap at neutral uncertainty

        let new_confidence = AxisEstimate::variance_to_confidence(new_variance)
            .min(self.config.max_inferred_confidence);

        AxisEstimate {
            axis: estimate.axis.clone(),
            value: estimate.value,
            confidence: new_confidence,
            variance: new_variance,
            source: InferenceSource::Decayed {
                original: Box::new(estimate.source.clone()),
                age_seconds: elapsed_seconds as u64,
                decay_factor: estimate.variance / new_variance,
            },
            timestamp: estimate.timestamp,
        }
    }

    /// Combine multiple observations into a single posterior.
    ///
    /// Useful when multiple signals about the same axis arrive at once.
    pub fn combine_observations(
        &self,
        axis: &str,
        prior: &Prior,
        observations: &[Observation],
    ) -> AxisEstimate {
        if observations.is_empty() {
            return AxisEstimate::prior(axis, prior.mean, 0.5, &prior.reason);
        }

        // Check for self-report (dominates everything)
        if let Some(sr) = observations.iter().find(|o| o.source.is_self_report()) {
            return AxisEstimate::self_report(axis, sr.value);
        }

        // Iteratively update with each observation
        let mut current = AxisEstimate {
            axis: axis.to_string(),
            value: prior.mean,
            confidence: AxisEstimate::variance_to_confidence(prior.variance),
            variance: prior.variance,
            source: InferenceSource::Prior {
                reason: prior.reason.clone(),
            },
            timestamp: Utc::now(),
        };

        let sources: Vec<InferenceSource> = observations.iter().map(|o| o.source.clone()).collect();
        let weights: Vec<f32> = observations
            .iter()
            .map(|o| 1.0 / o.noise_variance)
            .collect();

        for obs in observations {
            current = self.update_estimate(&current, obs);
        }

        // Update source to reflect combination
        AxisEstimate {
            source: InferenceSource::Combined { sources, weights },
            ..current
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neutral_prior() {
        let prior = Prior::neutral();
        assert_eq!(prior.mean, 0.5);
        assert!(prior.variance > 0.1); // High uncertainty
    }

    #[test]
    fn test_basic_update() {
        let updater = BayesianUpdater::new();
        let prior = Prior::neutral();
        let obs = Observation::from_linguistic(0.8, vec!["warmth".into()]);

        let posterior = updater.update("warmth", &prior, &obs);

        // Should move toward observation
        assert!(posterior.value > prior.mean);
        assert!(posterior.value < 0.8); // But not all the way
                                        // Variance should decrease
        assert!(posterior.variance < prior.variance);
    }

    #[test]
    fn test_self_report_dominates() {
        let updater = BayesianUpdater::new();
        let prior = Prior::from_value(0.2, 0.8, "strong belief in low value");
        let obs = Observation::from_self_report(0.9);

        let posterior = updater.update("warmth", &prior, &obs);

        assert_eq!(posterior.value, 0.9);
        assert_eq!(posterior.confidence, 1.0);
    }

    #[test]
    fn test_max_update_constraint() {
        let updater = BayesianUpdater::with_config(BayesianConfig {
            max_update: 0.1,
            ..Default::default()
        });

        let prior = Prior::from_value(0.2, 0.5, "prior");
        let obs = Observation::from_linguistic(0.9, vec![]); // Big jump

        let posterior = updater.update("warmth", &prior, &obs);

        // Should be constrained to max_update
        assert!(posterior.value <= 0.3 + 0.01); // 0.2 + 0.1 with epsilon
    }

    #[test]
    fn test_cannot_override_self_report() {
        let updater = BayesianUpdater::new();

        let self_report = AxisEstimate::self_report("warmth", 0.9);
        let obs = Observation::from_linguistic(0.2, vec![]);

        let result = updater.update_estimate(&self_report, &obs);

        // Self-report should persist
        assert_eq!(result.value, 0.9);
        assert!(result.source.is_self_report());
    }

    #[test]
    fn test_confidence_capping() {
        let updater = BayesianUpdater::new();
        let prior = Prior::neutral();

        // Many confident observations
        let obs = Observation::new(
            0.8,
            0.001,
            InferenceSource::Linguistic {
                features_used: vec![],
                feature_values: std::collections::HashMap::new(),
            },
        );

        let posterior = updater.update("warmth", &prior, &obs);

        // Should still be capped
        assert!(posterior.confidence <= MAX_INFERRED_CONFIDENCE);
    }

    #[test]
    fn test_uncertainty_growth() {
        let updater = BayesianUpdater::new();

        let estimate = AxisEstimate::inferred(
            "warmth",
            0.7,
            0.6,
            InferenceSource::Linguistic {
                features_used: vec![],
                feature_values: std::collections::HashMap::new(),
            },
        );

        let aged = updater.grow_uncertainty(&estimate, 3600.0); // 1 hour

        assert!(aged.variance > estimate.variance);
        assert!(aged.confidence < estimate.confidence);
    }

    #[test]
    fn test_combine_observations() {
        let updater = BayesianUpdater::new();
        let prior = Prior::neutral();

        let observations = vec![
            Observation::from_linguistic(0.7, vec!["feat1".into()]),
            Observation::from_linguistic(0.8, vec!["feat2".into()]),
        ];

        let combined = updater.combine_observations("warmth", &prior, &observations);

        // Should be somewhere between prior and observations
        assert!(combined.value > 0.5);
        // Should have combined source
        assert!(matches!(combined.source, InferenceSource::Combined { .. }));
    }
}
