//! # attuned-core
//!
//! Core types and traits for Attuned - a Rust framework for representing human state
//! as interpretable vectors and translating them into interaction constraints for LLM systems.
//!
//! ## Overview
//!
//! Attuned produces *context*, not actions. It represents user state across interpretable
//! dimensions (axes) and translates that state into guidelines for LLM interactions.
//!
//! ## Core Types
//!
//! - [`StateSnapshot`] - A point-in-time capture of user state
//! - [`PromptContext`] - Translated guidelines for LLM conditioning
//! - [`Translator`] - Trait for converting state to context
//!
//! ## Example
//!
//! ```rust
//! use attuned_core::{StateSnapshot, Source, RuleTranslator, Translator};
//!
//! // Create a state snapshot
//! let snapshot = StateSnapshot::builder()
//!     .user_id("user_123")
//!     .source(Source::SelfReport)
//!     .axis("warmth", 0.7)
//!     .axis("cognitive_load", 0.9)
//!     .build()
//!     .unwrap();
//!
//! // Translate to prompt context
//! let translator = RuleTranslator::default();
//! let context = translator.to_prompt_context(&snapshot);
//!
//! // Use guidelines in your LLM system prompt
//! for guideline in &context.guidelines {
//!     println!("{}", guideline);
//! }
//! ```

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod axes;
mod error;
mod snapshot;
pub mod telemetry;
mod translator;
mod types;

pub use axes::{
    get_axis, is_valid_axis_name, Axis, AxisCategory, AxisDefinition, DeprecationInfo,
    CANONICAL_AXES,
};
pub use error::{AttunedError, ValidationError};
pub use snapshot::{StateSnapshot, StateSnapshotBuilder};
pub use telemetry::{
    init_tracing, init_tracing_from_env, AuditEvent, AuditEventType, ComponentHealth, HealthCheck,
    HealthState, HealthStatus, OtelConfig, TelemetryBuilder, TelemetryGuard, TracingConfig,
    TracingFormat,
};
pub use translator::{PromptContext, RuleTranslator, Thresholds, Translator, Verbosity};
pub use types::Source;
