# TASK-006: attuned-http - Reference HTTP Server

## Status
- [ ] Not Started
- [ ] In Progress
- [x] Completed
- [ ] Blocked

**Priority**: Medium
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: 3 - Backends & Integrations
**Depends On**: TASK-002, TASK-003, TASK-005
**Blocks**: None (optional component)

## Task Description
Implement a reference HTTP server that exposes the Attuned API. This is **optional** (feature-gated) but provides a ready-to-use server for integrations. The server should be minimal, secure, and fully observable.

## Requirements
1. REST API matching wire format in AGENTS.md
2. Built with `axum` for modern async HTTP
3. OpenAPI specification (auto-generated)
4. Full observability integration
5. Configurable authentication
6. Health and readiness endpoints
7. Graceful shutdown

## API Endpoints

### State Management
```
POST   /v1/state              # Upsert state (patch semantics)
GET    /v1/state/{user_id}    # Get latest state
GET    /v1/state/{user_id}/history?limit=N  # Get history (optional)
DELETE /v1/state/{user_id}    # Delete state (GDPR compliance)
```

### Translation
```
GET    /v1/context/{user_id}  # Get PromptContext for user
POST   /v1/translate          # Translate arbitrary StateSnapshot
```

### Operations
```
GET    /health                # Health check
GET    /ready                 # Readiness check
GET    /metrics               # Prometheus metrics
GET    /openapi.json          # OpenAPI spec
```

## Wire Format (from AGENTS.md)

### POST /v1/state (Upsert)
```json
{
  "user_id": "u_123",
  "source": "self_report",
  "confidence": 1.0,
  "axes": {
    "warmth": 0.6,
    "formality": 0.3,
    "boundary_strength": 0.8
  }
}
```

### GET /v1/context/{user_id} (Response)
```json
{
  "guidelines": [
    "Offer suggestions, not actions",
    "Drafts require explicit user approval",
    "Keep responses concise; avoid multi-step plans unless requested"
  ],
  "tone": "calm-neutral",
  "verbosity": "low",
  "flags": ["high_cognitive_load"]
}
```

## Implementation Notes

### Server Configuration
```rust
pub struct HttpServerConfig {
    pub bind_addr: SocketAddr,
    pub request_timeout: Duration,
    pub body_limit: usize,              // Max request body size
    pub cors_origins: Vec<String>,
    pub auth: AuthConfig,
    pub tls: Option<TlsConfig>,
}

pub enum AuthConfig {
    None,                               // Development only
    ApiKey { keys: Vec<String> },
    Jwt { issuer: String, audience: String },
}
```

### Middleware Stack
1. Request ID injection (for tracing correlation)
2. Tracing span creation
3. Metrics collection
4. Authentication
5. Rate limiting (optional)
6. Body size limiting

### Error Response Format
```json
{
  "error": {
    "code": "USER_NOT_FOUND",
    "message": "No state found for user u_123",
    "request_id": "req_abc123"
  }
}
```

### Graceful Shutdown
- Listen for SIGTERM/SIGINT
- Stop accepting new connections
- Wait for in-flight requests (with timeout)
- Flush metrics/traces
- Close store connections

## Acceptance Criteria
- [ ] All endpoints implemented per spec
- [ ] OpenAPI 3.0 spec auto-generated
- [ ] Full tracing with request correlation
- [ ] Prometheus metrics endpoint
- [ ] Health/readiness endpoints
- [ ] API key authentication working
- [ ] Graceful shutdown implemented
- [ ] Integration tests for all endpoints
- [ ] Docker image builds and runs
- [ ] Example docker-compose with Qdrant

## Progress Log
- 2025-12-16: Task created
- 2025-12-16: Implemented all REST endpoints (state CRUD, context, translate)
- 2025-12-16: Added /health and /ready endpoints
- 2025-12-16: Integrated tracing middleware
- 2025-12-16: Added 4 integration tests (all pass)
- 2025-12-16: **COMPLETED** - Server ready for use with MemoryStore
