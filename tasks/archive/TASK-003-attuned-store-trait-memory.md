# TASK-003: attuned-store - StateStore Trait & In-Memory Backend

## Status
- [ ] Not Started
- [ ] In Progress
- [x] Completed
- [ ] Blocked

**Priority**: High
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: 2 - Core Implementation
**Depends On**: TASK-001, TASK-002
**Blocks**: TASK-004, TASK-006

## Task Description
Implement the StateStore trait and a production-ready in-memory store. The trait defines the storage contract; the in-memory store serves as the default backend for single-process apps and testing.

## Requirements
1. Define `StateStore` async trait
2. Implement thread-safe in-memory store using `DashMap` or `RwLock<HashMap>`
3. Support upsert semantics (insert or update)
4. Optional: history snapshots (configurable retention)
5. Proper error types with context
6. Instrument with tracing for observability

## StateStore Trait (from AGENTS.md)

```rust
#[async_trait]
pub trait StateStore: Send + Sync {
    async fn upsert_latest(&self, snapshot: StateSnapshot) -> Result<()>;
    async fn get_latest(&self, user_id: &str) -> Result<Option<StateSnapshot>>;

    // Optional extensions for history
    async fn get_history(&self, user_id: &str, limit: usize) -> Result<Vec<StateSnapshot>> {
        Ok(vec![]) // Default: no history
    }
}
```

## Implementation Notes

### In-Memory Store
```rust
pub struct MemoryStore {
    latest: DashMap<String, StateSnapshot>,
    history: Option<DashMap<String, VecDeque<StateSnapshot>>>,
    config: MemoryStoreConfig,
}

pub struct MemoryStoreConfig {
    pub enable_history: bool,
    pub max_history_per_user: usize,  // Default: 100
}
```

### Observability Hooks
- `tracing::instrument` on all trait methods
- Emit spans: `attuned.store.upsert`, `attuned.store.get`
- Include user_id in span context (be mindful of cardinality)

### Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("user not found: {user_id}")]
    UserNotFound { user_id: String },
    #[error("storage error: {0}")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}
```

## Acceptance Criteria
- [ ] `StateStore` trait defined in attuned-store
- [ ] `MemoryStore` implements `StateStore`
- [ ] Thread-safe for concurrent access
- [ ] History support (optional, configurable)
- [ ] All operations instrumented with tracing
- [ ] Unit tests for concurrent access patterns
- [ ] Benchmark for throughput (ops/sec)

## Progress Log
- 2025-12-16: Task created
- 2025-12-16: Defined StateStore async trait
- 2025-12-16: Implemented MemoryStore with DashMap for thread-safety
- 2025-12-16: Added optional history support with configurable retention
- 2025-12-16: Instrumented all operations with tracing
- 2025-12-16: Added 6 unit tests including concurrent access test
- 2025-12-16: **COMPLETED** - Production-ready in-memory store
