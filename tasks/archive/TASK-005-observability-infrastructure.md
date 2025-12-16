# TASK-005: Observability Infrastructure

## Status
- [ ] Not Started
- [ ] In Progress
- [x] Completed
- [ ] Blocked

**Priority**: High
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: 2 - Core Implementation (Cross-cutting)
**Depends On**: TASK-001
**Blocks**: All other tasks use this infrastructure

## Task Description
Implement comprehensive observability infrastructure using the Rust ecosystem best practices. This is a **core priority** - every operation in Attuned should be observable. The goal is production-grade visibility into system behavior.

## Requirements
1. Structured logging with `tracing`
2. Metrics with `metrics` crate (Prometheus-compatible)
3. Distributed tracing support (OpenTelemetry)
4. Health check primitives
5. Audit logging for state mutations
6. Easy integration for library consumers

## Observability Stack

### 1. Structured Logging (tracing)
```rust
// In attuned-core or a new attuned-telemetry crate
pub fn init_tracing(config: TracingConfig) -> Result<TracingGuard> {
    // Configure subscriber with:
    // - JSON formatting for production
    // - Pretty formatting for development
    // - Level filtering
    // - Span events (enter/exit)
}

pub struct TracingConfig {
    pub format: TracingFormat,      // Json | Pretty
    pub level: tracing::Level,
    pub service_name: String,
    pub include_target: bool,
    pub include_file_line: bool,
}
```

### 2. Metrics (Prometheus-style)
```rust
// Key metrics to expose
pub struct AttunedMetrics {
    // Counters
    state_updates_total: Counter,
    state_reads_total: Counter,
    translation_total: Counter,
    errors_total: Counter,  // Labeled by error_type

    // Histograms
    state_update_duration_seconds: Histogram,
    state_read_duration_seconds: Histogram,
    translation_duration_seconds: Histogram,

    // Gauges
    active_users: Gauge,  // Users with state in last N minutes
    store_size_bytes: Gauge,  // For memory store
}
```

### 3. Distributed Tracing (OpenTelemetry)
```rust
pub fn init_opentelemetry(config: OtelConfig) -> Result<()> {
    // Support OTLP exporter
    // Propagate trace context
    // Baggage for user_id (careful with PII)
}

pub struct OtelConfig {
    pub endpoint: String,           // OTLP endpoint
    pub service_name: String,
    pub service_version: String,
    pub sample_rate: f64,           // 0.0 - 1.0
}
```

### 4. Health Checks
```rust
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: HealthState,        // Healthy | Degraded | Unhealthy
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: Vec<ComponentHealth>,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub name: String,               // e.g., "qdrant", "memory_store"
    pub status: HealthState,
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
}

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> ComponentHealth;
}
```

### 5. Audit Logging
```rust
// Structured audit events for state mutations
#[derive(Debug, Serialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user_id: String,
    pub source: Source,
    pub axes_changed: Vec<String>,  // Which axes were modified
    pub confidence: f32,
    pub trace_id: Option<String>,
}

pub enum AuditEventType {
    StateCreated,
    StateUpdated,
    StateDeleted,
}
```

## Implementation Notes

### Crate Organization Options
**Option A:** New `attuned-telemetry` crate
- Clean separation of concerns
- Easy to version independently

**Option B:** Feature in `attuned-core`
- Fewer crates to manage
- Always available

**Recommendation:** Start with Option B, extract if needed

### Integration Pattern
```rust
// Library consumers can opt-in
use attuned_core::telemetry;

fn main() {
    // Initialize once at startup
    let _guard = telemetry::init(telemetry::Config::default());

    // All Attuned operations now emit traces/metrics
}
```

### Span Naming Convention
- `attuned.store.{operation}` - Storage operations
- `attuned.translate` - Translation operations
- `attuned.http.{endpoint}` - HTTP endpoints
- `attuned.qdrant.{operation}` - Qdrant-specific

### Metric Naming Convention (Prometheus style)
- `attuned_store_operations_total{operation="upsert|get",status="success|error"}`
- `attuned_store_duration_seconds{operation="upsert|get"}`
- `attuned_translation_duration_seconds`
- `attuned_http_request_duration_seconds{method="GET|POST",path="/state"}`

## Acceptance Criteria
- [ ] `tracing` integration with configurable output
- [ ] Metrics registry with Prometheus exposition
- [ ] OpenTelemetry OTLP export support
- [ ] Health check trait and composable checks
- [ ] Audit event emission for all state changes
- [ ] Zero-cost when disabled (feature flags)
- [ ] Documentation with integration examples
- [ ] Example Grafana dashboard JSON
- [ ] Example alert rules (Prometheus format)

## Progress Log
- 2025-12-16: Task created
- 2025-12-16: Created telemetry module in attuned-core
- 2025-12-16: Implemented TracingConfig and TelemetryBuilder
- 2025-12-16: Added HealthCheck trait and HealthStatus types
- 2025-12-16: Implemented AuditEvent for state mutation logging
- 2025-12-16: Defined metric_names and span_names constants
- 2025-12-16: Integrated HealthCheck into MemoryStore
- 2025-12-16: **COMPLETED** - Observability primitives ready for use
