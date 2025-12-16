# Remaining Tasks for Attuned

## Priority Order (Recommended)

### 1. TASK-011: Release v1.0 Preparation (High Priority)
**File**: `tasks/TASK-011-release-v1.md`
**Effort**: ~2 hours
**Why**: Final polish before public release

**Scope**:
- Version bump
- CHANGELOG update
- crates.io publishing prep
- Documentation review

### 2. TASK-004: Qdrant Backend (Medium Priority)
**File**: `tasks/TASK-004-attuned-qdrant-backend.md`
**Effort**: ~4 hours
**Why**: Enables persistent storage beyond in-memory

**Scope**:
- Implement QdrantStore struct
- Connect to Qdrant and ensure collection exists
- Implement StateStore trait methods
- Add integration tests (requires Qdrant instance)

**Implementation Notes**:
- Latest snapshot point id: `{user_id}::latest`
- History snapshot point id: `{user_id}::{unix_ms}`
- See `crates/attuned-qdrant/src/store.rs` for skeleton

### 3. TASK-014: Python Bindings (Medium Priority)
**File**: `tasks/TASK-014-python-bindings.md`
**Effort**: ~10 hours
**Why**: Python is dominant in AI/ML; enables LangChain/LlamaIndex integration

**Scope**:
- PyO3 bindings for core types
- Async HTTP client
- Type stubs for IDE support
- Publish to PyPI as `attuned`

### 4. TASK-007: CLI Tool Completion (Low Priority)
**File**: `tasks/TASK-007-attuned-cli.md`
**Effort**: ~2 hours
**Why**: Nice to have for development/debugging

**Scope**:
- Implement `get` command
- Implement `set` command
- Add shell completions
- Add configuration file support

## Completed Tasks

| Task | Description |
|------|-------------|
| TASK-013 | Governance & schema formalization (MANIFESTO.md, AxisDefinition, forbidden_uses) |
| TASK-012 | Security hardening (headers, rate limiting, auth, PII redaction) |
| TASK-001 | Workspace scaffolding |
| TASK-002 | Core types & traits |
| TASK-003 | StateStore + MemoryStore |
| TASK-005 | Observability infrastructure |
| TASK-006 | HTTP server |
| TASK-008 | Testing infrastructure (property tests, benchmarks) |
| TASK-010 | CI/CD pipeline (GitHub Actions) |
| TASK-009 | Documentation (partial) |

## Known Issues

1. **Axis count mismatch**: AGENTS.md says 24 axes, implementation has 23. The count is correct; spec was off by one.

2. **Doc-tests ignored**: Some doc-tests marked `ignore` because they require runtime setup. Intentional.

3. **No Cargo.lock tracked**: `.gitignore` excludes it since this is a library workspace.
