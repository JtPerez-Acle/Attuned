# Attuned Architecture

## Workspace Structure

```
attuned/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── attuned-core/          # Core types, axes, translator, telemetry
│   ├── attuned-store/         # Storage abstraction + in-memory backend
│   ├── attuned-qdrant/        # Qdrant storage backend (STUBS ONLY)
│   ├── attuned-http/          # Reference HTTP server
│   └── attuned-cli/           # CLI development tool (basic structure)
├── tasks/                     # Task tracking for agentic development
│   └── archive/               # Completed tasks
├── docs/                      # Additional documentation
└── tests/                     # Integration tests
```

## Crate Dependency Graph

```
attuned-cli
    └── attuned-http
        └── attuned-store
            └── attuned-core

attuned-qdrant
    └── attuned-store
        └── attuned-core
```

## Crate Details

### attuned-core (Core Library)
**Location**: `crates/attuned-core/src/`
**Purpose**: Core types, axes, translators, telemetry

**Key Files**:
- `lib.rs` - Public API exports
- `types.rs` - Source enum (SelfReport, Inferred, Mixed)
- `axes.rs` - 23 canonical axes with metadata
- `snapshot.rs` - StateSnapshot + builder pattern
- `translator.rs` - Translator trait + RuleTranslator implementation
- `error.rs` - Error types
- `telemetry/mod.rs` - Health checks, audit events, metrics
- `telemetry/setup.rs` - Tracing initialization

**Key Types**:
- `StateSnapshot` - User state with axes, source, confidence
- `PromptContext` - Output of translation (guidelines, tone, verbosity, flags)
- `Source` - Enum: SelfReport, Inferred, Mixed
- `Verbosity` - Enum: Low, Medium, High
- `RuleTranslator` - Rule-based translator implementation

### attuned-store (Storage Abstraction)
**Location**: `crates/attuned-store/src/`
**Purpose**: StateStore trait and in-memory implementation

**Key Files**:
- `lib.rs` - Public exports
- `traits.rs` - StateStore async trait definition
- `memory.rs` - MemoryStore implementation (thread-safe, DashMap-based)
- `error.rs` - Storage errors

**Key Traits**:
```rust
#[async_trait]
pub trait StateStore: Send + Sync {
    async fn upsert_latest(&self, snapshot: StateSnapshot) -> Result<()>;
    async fn get_latest(&self, user_id: &str) -> Result<Option<StateSnapshot>>;
    async fn delete(&self, user_id: &str) -> Result<()>;
    async fn get_history(&self, user_id: &str, limit: usize) -> Result<Vec<StateSnapshot>>;
}
```

### attuned-qdrant (Qdrant Backend)
**Location**: `crates/attuned-qdrant/src/`
**Status**: STUBS ONLY - methods return `todo!()`

**Key Files**:
- `lib.rs` - Public exports
- `config.rs` - QdrantConfig struct
- `store.rs` - QdrantStore skeleton (needs implementation)
- `error.rs` - Qdrant-specific errors

### attuned-http (HTTP Server)
**Location**: `crates/attuned-http/src/`
**Purpose**: Reference HTTP server using Axum

**Key Files**:
- `lib.rs` - Public exports
- `server.rs` - Server setup, router, graceful shutdown
- `handlers.rs` - Request handlers for all endpoints
- `config.rs` - ServerConfig struct
- `error.rs` - HTTP errors

**API Endpoints**:
| Method | Path | Description |
|--------|------|-------------|
| POST | /v1/state | Upsert state (patch semantics) |
| GET | /v1/state/{user_id} | Get latest state |
| DELETE | /v1/state/{user_id} | Delete state (GDPR) |
| GET | /v1/context/{user_id} | Get translated PromptContext |
| POST | /v1/translate | Translate arbitrary state |
| GET | /health | Health check |
| GET | /ready | Readiness check |

### attuned-cli (CLI Tool)
**Location**: `crates/attuned-cli/src/`
**Status**: Basic structure only, commands print placeholders

**Key Files**:
- `main.rs` - Clap-based CLI with serve, get, set subcommands

## Key Design Decisions

1. **Builder Pattern for StateSnapshot**: Provides ergonomic construction with validation
2. **Async Trait for Storage**: Enables async backends (Qdrant, databases)
3. **DashMap for MemoryStore**: Thread-safe concurrent access without locks
4. **Axum for HTTP**: Modern, performant, tower-compatible
5. **Feature-gated Backends**: Qdrant is optional to keep core lightweight

## Test Organization
- Unit tests: In same file as code (`#[cfg(test)] mod tests`)
- Integration tests: `tests/` directory (if any)
- Doc tests: In documentation comments

## Configuration Patterns
- Structs with `Default` implementations
- Builder patterns where appropriate
- Environment variable support via Clap for CLI
