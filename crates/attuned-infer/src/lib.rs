//! # attuned-infer
//!
//! Fast, transparent inference of human state axes from natural language.
//!
//! This crate provides **declared, bounded, and subordinate** inference -
//! all estimates are auditable, confidence-bounded, and always overridable
//! by self-report.
//!
//! ## Design Principles
//!
//! 1. **Declared**: Every inference includes its source and reasoning
//! 2. **Bounded**: Inferred values have capped confidence (default 0.7)
//! 3. **Subordinate**: Self-report always overrides inference
//! 4. **Fast**: Sub-millisecond inference for real-time use
//!
//! ## Architecture
//!
//! ```text
//! [Message] ──→ [LinguisticFeatures] ──→ [AxisEstimate]
//!                     ~100μs                    │
//!                                               ▼
//! [History] ──→ [DeltaAnalysis] ──────→ [BayesianUpdater] ──→ [StateEstimate]
//!                   ~500μs                      │
//!                                               ▼
//! [Self-Report] ─────────────────────→ [Override (σ² → 0)]
//! ```
//!
//! ## Example
//!
//! ```rust
//! use attuned_infer::{InferenceEngine, InferredState};
//!
//! let engine = InferenceEngine::default();
//!
//! // Infer state from a message
//! let state = engine.infer("I need help ASAP, this is urgent!!!");
//!
//! // All estimates include source and confidence
//! if let Some(urgency) = state.get("urgency_sensitivity") {
//!     println!("urgency: {:.2} (confidence: {:.2}, source: {:?})",
//!         urgency.value, urgency.confidence, urgency.source);
//! }
//! ```

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

mod bayesian;
mod delta;
mod engine;
mod estimate;
mod features;

pub use bayesian::{BayesianUpdater, Prior};
pub use delta::{Baseline, DeltaAnalyzer, DeltaSignals};
pub use engine::{infer, InferenceConfig, InferenceEngine};
pub use estimate::{
    max_confidence_for_axis, word_count_confidence_factor, AxisEstimate, InferenceSource,
    InferredState,
};
pub use features::{LinguisticExtractor, LinguisticFeatures};
