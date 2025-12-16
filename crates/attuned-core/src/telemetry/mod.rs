//! Observability infrastructure for Attuned.
//!
//! This module provides:
//! - Structured logging with [`tracing`]
//! - Metrics collection
//! - Health check primitives
//! - Audit event types
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use attuned_core::telemetry::{init_tracing, TracingConfig};
//!
//! fn main() {
//!     let _guard = init_tracing(TracingConfig::default());
//!     // All Attuned operations now emit traces
//! }
//! ```

mod setup;

pub use setup::{init_tracing, init_tracing_from_env, TelemetryBuilder, TelemetryGuard};

use serde::{Deserialize, Serialize};

/// Configuration for tracing/logging.
#[derive(Clone, Debug)]
pub struct TracingConfig {
    /// Output format.
    pub format: TracingFormat,
    /// Service name for distributed tracing.
    pub service_name: String,
    /// Minimum log level.
    pub level: String,
    /// Include source file and line in logs.
    pub include_file_line: bool,
    /// Include target (module path) in logs.
    pub include_target: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            format: TracingFormat::Pretty,
            service_name: "attuned".to_string(),
            level: "info".to_string(),
            include_file_line: false,
            include_target: true,
        }
    }
}

/// Tracing output format.
#[derive(Clone, Debug, Default)]
pub enum TracingFormat {
    /// Human-readable colored output (development).
    #[default]
    Pretty,
    /// JSON structured output (production).
    Json,
    /// Compact single-line output.
    Compact,
}

/// Configuration for OpenTelemetry export.
#[derive(Clone, Debug)]
pub struct OtelConfig {
    /// OTLP endpoint URL.
    pub endpoint: String,
    /// Service name.
    pub service_name: String,
    /// Service version.
    pub service_version: String,
    /// Sample rate (0.0 - 1.0).
    pub sample_rate: f64,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4317".to_string(),
            service_name: "attuned".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            sample_rate: 1.0,
        }
    }
}

/// Health status of a component.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    /// Component is fully operational.
    Healthy,
    /// Component is operational but with issues.
    Degraded,
    /// Component is not operational.
    Unhealthy,
}

/// Health status of a single component.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name.
    pub name: String,
    /// Component status.
    pub status: HealthState,
    /// Response latency in milliseconds.
    pub latency_ms: Option<u64>,
    /// Additional status message.
    pub message: Option<String>,
}

impl ComponentHealth {
    /// Create a healthy component status.
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthState::Healthy,
            latency_ms: None,
            message: None,
        }
    }

    /// Create a healthy component status with latency.
    pub fn healthy_with_latency(name: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            name: name.into(),
            status: HealthState::Healthy,
            latency_ms: Some(latency_ms),
            message: None,
        }
    }

    /// Create an unhealthy component status.
    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthState::Unhealthy,
            latency_ms: None,
            message: Some(message.into()),
        }
    }

    /// Create a degraded component status.
    pub fn degraded(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthState::Degraded,
            latency_ms: None,
            message: Some(message.into()),
        }
    }
}

/// Overall system health status.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall status (worst of all components).
    pub status: HealthState,
    /// Service version.
    pub version: String,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
    /// Individual component health checks.
    pub checks: Vec<ComponentHealth>,
}

impl HealthStatus {
    /// Create a new health status from component checks.
    pub fn from_checks(checks: Vec<ComponentHealth>, uptime_seconds: u64) -> Self {
        let status = checks
            .iter()
            .map(|c| &c.status)
            .fold(HealthState::Healthy, |acc, s| match (&acc, s) {
                (HealthState::Unhealthy, _) | (_, HealthState::Unhealthy) => HealthState::Unhealthy,
                (HealthState::Degraded, _) | (_, HealthState::Degraded) => HealthState::Degraded,
                _ => HealthState::Healthy,
            });

        Self {
            status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds,
            checks,
        }
    }
}

/// Trait for components that can report their health.
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform a health check and return the status.
    async fn check(&self) -> ComponentHealth;
}

/// Audit event types for state mutations.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// New state created for a user.
    StateCreated,
    /// Existing state updated.
    StateUpdated,
    /// State deleted.
    StateDeleted,
}

/// Audit event for tracking state mutations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Type of event.
    pub event_type: AuditEventType,
    /// User ID affected.
    pub user_id: String,
    /// Source of the state change.
    pub source: crate::Source,
    /// Which axes were modified.
    pub axes_changed: Vec<String>,
    /// Confidence of the new state.
    pub confidence: f32,
    /// Trace ID for correlation (if available).
    pub trace_id: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event.
    pub fn new(
        event_type: AuditEventType,
        user_id: impl Into<String>,
        source: crate::Source,
        axes_changed: Vec<String>,
        confidence: f32,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            event_type,
            user_id: user_id.into(),
            source,
            axes_changed,
            confidence,
            trace_id: None,
        }
    }

    /// Set the trace ID for correlation.
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Emit this audit event to the tracing system.
    pub fn emit(&self) {
        tracing::info!(
            event_type = ?self.event_type,
            user_id = %self.user_id,
            source = %self.source,
            axes_changed = ?self.axes_changed,
            confidence = %self.confidence,
            trace_id = ?self.trace_id,
            "audit_event"
        );
    }
}

/// Metric names used by Attuned.
pub mod metric_names {
    /// Counter: Total state update operations.
    pub const STATE_UPDATES_TOTAL: &str = "attuned_state_updates_total";
    /// Counter: Total state read operations.
    pub const STATE_READS_TOTAL: &str = "attuned_state_reads_total";
    /// Counter: Total translation operations.
    pub const TRANSLATIONS_TOTAL: &str = "attuned_translations_total";
    /// Counter: Total errors by type.
    pub const ERRORS_TOTAL: &str = "attuned_errors_total";
    /// Histogram: State update duration in seconds.
    pub const STATE_UPDATE_DURATION: &str = "attuned_state_update_duration_seconds";
    /// Histogram: State read duration in seconds.
    pub const STATE_READ_DURATION: &str = "attuned_state_read_duration_seconds";
    /// Histogram: Translation duration in seconds.
    pub const TRANSLATION_DURATION: &str = "attuned_translation_duration_seconds";
    /// Gauge: Number of active users (with state in last N minutes).
    pub const ACTIVE_USERS: &str = "attuned_active_users";
    /// Histogram: HTTP request duration in seconds.
    pub const HTTP_REQUEST_DURATION: &str = "attuned_http_request_duration_seconds";
}

/// Span names used by Attuned.
pub mod span_names {
    /// Span for state upsert operations.
    pub const STORE_UPSERT: &str = "attuned.store.upsert";
    /// Span for state get operations.
    pub const STORE_GET: &str = "attuned.store.get";
    /// Span for state delete operations.
    pub const STORE_DELETE: &str = "attuned.store.delete";
    /// Span for translation operations.
    pub const TRANSLATE: &str = "attuned.translate";
    /// Span for health check operations.
    pub const HEALTH_CHECK: &str = "attuned.health_check";
    /// Span for HTTP request handling.
    pub const HTTP_REQUEST: &str = "attuned.http.request";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_aggregation() {
        let checks = vec![
            ComponentHealth::healthy("store"),
            ComponentHealth::degraded("qdrant", "high latency"),
        ];

        let status = HealthStatus::from_checks(checks, 100);
        assert_eq!(status.status, HealthState::Degraded);
    }

    #[test]
    fn test_health_status_unhealthy_dominates() {
        let checks = vec![
            ComponentHealth::healthy("store"),
            ComponentHealth::unhealthy("qdrant", "connection failed"),
            ComponentHealth::degraded("cache", "high miss rate"),
        ];

        let status = HealthStatus::from_checks(checks, 100);
        assert_eq!(status.status, HealthState::Unhealthy);
    }

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::StateUpdated,
            "user_123",
            crate::Source::SelfReport,
            vec!["warmth".to_string(), "formality".to_string()],
            1.0,
        );

        assert_eq!(event.user_id, "user_123");
        assert_eq!(event.axes_changed.len(), 2);
    }
}
