# TASK-004: attuned-qdrant - Qdrant Storage Backend

## Status
- [x] Not Started (Post-v1.0)
- [ ] In Progress
- [ ] Completed
- [ ] Blocked

**Priority**: Low (Post-v1.0 Enhancement)
**Created**: 2025-12-16
**Last Updated**: 2025-12-18
**Phase**: 7 - Post-v1.0 Enhancements
**Depends On**: TASK-002, TASK-003 (completed)
**Blocks**: None (optional backend)

> **Note**: Stubs exist. MemoryStore is sufficient for v1.0.0. This task is deferred to post-release for production distributed deployments.

## Task Description
Implement a Qdrant-backed StateStore for persistent, distributed state storage. Qdrant is used as a **snapshot store**, not for semantic search. This enables production deployments with state durability across restarts.

## Requirements
1. Implement `StateStore` trait for Qdrant
2. Connection pooling and retry logic
3. Collection auto-creation with proper schema
4. Latest + history snapshot support
5. Configurable via environment or struct
6. Full observability (tracing, metrics)

## Qdrant Point Schema (from AGENTS.md)

**Point ID Convention:**
- Latest: `{user_id}::latest`
- History: `{user_id}::{unix_ms}`

**Payload:**
```json
{
  "user_id": "u_123",
  "updated_at_unix_ms": 1702742400000,
  "source": "self_report",
  "confidence": 0.95,
  "schema_version": 1,
  "axes": { "warmth": 0.6, "formality": 0.3 }
}
```

**Vector:** The axes as a dense vector (for optional similarity queries)

## Implementation Notes

### Configuration
```rust
pub struct QdrantStoreConfig {
    pub url: String,                    // e.g., "http://localhost:6334"
    pub collection_name: String,        // Default: "attuned_state"
    pub api_key: Option<String>,
    pub enable_history: bool,
    pub history_retention_days: Option<u32>,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
}
```

### Connection Management
- Use `qdrant_client` crate
- Implement health check method
- Retry with exponential backoff on transient failures
- Circuit breaker pattern for cascading failure prevention

### Collection Setup
```rust
async fn ensure_collection(&self) -> Result<()> {
    // Create collection if not exists
    // Vector size = number of axes (24 for MVP)
    // Distance: Cosine (for optional similarity)
}
```

### Observability
- Spans: `attuned.qdrant.upsert`, `attuned.qdrant.get`, `attuned.qdrant.health`
- Metrics: `attuned_qdrant_request_duration_seconds`, `attuned_qdrant_errors_total`
- Include Qdrant operation type in span attributes

## Acceptance Criteria
- [ ] `QdrantStore` implements `StateStore`
- [ ] Auto-creates collection on first use
- [ ] Connection pooling works correctly
- [ ] Retry logic handles transient failures
- [ ] History snapshots with optional retention
- [ ] Health check endpoint support
- [ ] Full tracing instrumentation
- [ ] Integration tests (requires Qdrant instance)
- [ ] Docker Compose for local testing

## Progress Log
- 2025-12-16: Task created
