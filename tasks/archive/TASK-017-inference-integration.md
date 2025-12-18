# TASK-017: Integrate attuned-infer with HTTP Server and Storage

## Status
- [ ] Not Started
- [ ] In Progress
- [x] Completed
- [ ] Blocked

**Priority**: Medium
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: Integration
**Depends On**: TASK-016 (refinements), attuned-http, attuned-store
**Blocks**: None

## Task Description
Connect the attuned-infer crate to the rest of the Attuned system. Currently, inference is an isolated island - it needs to:
1. Optionally run on incoming messages via HTTP API
2. Store/retrieve baselines per user
3. Merge inferred state with self-reported state
4. Expose inference results through the API

## Background
Current architecture:
```
Client → HTTP API → StateStore → StateSnapshot
                  ↓
              RuleTranslator → PromptContext
```

Target architecture:
```
Client → HTTP API → [InferenceEngine] → StateStore → StateSnapshot
              ↑              ↓
         (message text)   (baseline)
                              ↓
                        RuleTranslator → PromptContext
```

## Requirements

### 1. Add Message Text to State Endpoint
Currently `/v1/state` accepts axes only. Add optional `message` field for inference.

**File**: `crates/attuned-http/src/handlers.rs`

```rust
#[derive(Deserialize)]
pub struct StateRequest {
    pub user_id: String,
    pub source: Source,
    pub confidence: Option<f32>,
    pub axes: HashMap<String, f32>,
    // NEW: Optional message for inference
    pub message: Option<String>,
}
```

### 2. Add Inference to AppState
The HTTP server needs access to inference engine and per-user baselines.

**File**: `crates/attuned-http/src/handlers.rs`

```rust
pub struct AppState<S: StateStore> {
    pub store: S,
    pub translator: RuleTranslator,
    // NEW
    pub inference_engine: Option<InferenceEngine>,
    pub baselines: DashMap<String, Baseline>,  // user_id → Baseline
}
```

### 3. Inference Flow in State Handler
When message is provided, run inference and merge with explicit axes.

```rust
async fn handle_state_post(state: &AppState, req: StateRequest) {
    let mut axes = req.axes;

    if let (Some(engine), Some(message)) = (&state.inference_engine, &req.message) {
        // Get or create baseline for user
        let baseline = state.baselines
            .entry(req.user_id.clone())
            .or_insert_with(|| engine.new_baseline());

        // Run inference
        let inferred = engine.infer_with_baseline(&message, baseline, None);

        // Merge: explicit axes override inferred
        for estimate in inferred.all() {
            if !axes.contains_key(&estimate.axis) {
                // Only use inferred if not explicitly provided
                axes.insert(estimate.axis.clone(), estimate.value);
            }
        }
    }

    // Continue with normal state storage...
}
```

### 4. Baseline Persistence (Optional)
Baselines are currently in-memory. For production, they should persist.

Options:
- Store in StateStore alongside snapshots
- Separate baseline collection in Qdrant
- Redis/memory cache with TTL

Simplest: Add to StateStore trait:
```rust
async fn get_baseline(&self, user_id: &str) -> Result<Option<Baseline>>;
async fn upsert_baseline(&self, user_id: &str, baseline: &Baseline) -> Result<()>;
```

### 5. New Inference Endpoint
Add endpoint to get inference without storing state.

```
POST /v1/infer
{
    "message": "I'm really stressed about this deadline!!!",
    "user_id": "optional_for_baseline"
}

Response:
{
    "estimates": [
        {
            "axis": "anxiety_level",
            "value": 0.72,
            "confidence": 0.58,
            "source": {
                "type": "linguistic",
                "features_used": ["negative_emotion_count", "exclamation_ratio"]
            }
        },
        ...
    ],
    "features": {  // Optional debug info
        "word_count": 8,
        "negative_emotion_count": 1,
        ...
    }
}
```

### 6. Configuration
Add inference config to ServerConfig.

```rust
pub struct ServerConfig {
    // ... existing fields ...

    /// Enable automatic inference from message text
    pub enable_inference: bool,

    /// Inference configuration
    pub inference_config: Option<InferenceConfig>,
}
```

### 7. Feature Flag
Inference should be optional (not all deployments want it).

```toml
# Cargo.toml
[features]
default = []
inference = ["attuned-infer"]
```

## API Changes Summary

| Endpoint | Change |
|----------|--------|
| `POST /v1/state` | Add optional `message` field |
| `POST /v1/infer` | NEW - inference without storage |
| `GET /v1/context/{id}` | No change (uses stored state) |

## Implementation Order
1. Add attuned-infer as optional dependency
2. Add inference config to ServerConfig
3. Add AppState fields for engine and baselines
4. Implement `/v1/infer` endpoint (standalone)
5. Add message field to state endpoint
6. Implement merge logic
7. (Optional) Baseline persistence

## Testing
- Test inference endpoint returns valid estimates
- Test state endpoint with message performs inference
- Test explicit axes override inferred values
- Test baseline accumulation across requests
- Benchmark: inference shouldn't add >50μs to request latency

## Security Considerations
- Message text should NOT be logged (PII)
- Rate limit inference endpoint (CPU-intensive)
- Consider message length limits

## Success Criteria
- [x] Inference endpoint works standalone
- [x] State endpoint accepts optional message
- [x] Inferred axes merge correctly with explicit axes
- [x] Baselines persist across requests per user (in-memory, DashMap)
- [x] Feature-flagged (can disable inference)
- [x] No performance regression on non-inference paths
- [x] Tests pass

## References
- attuned-infer/src/lib.rs - inference API
- attuned-http/src/handlers.rs - current handlers
- MANIFESTO.md - self-report sovereignty (explicit always wins)
