# Technical Specification

Quick-reference specification for Attuned developers. For detailed documentation, see [`tasks/reports/`](tasks/reports/).

## Data Contracts

### StateSnapshot

```rust
struct StateSnapshot {
    user_id: String,           // Required, non-empty
    updated_at_unix_ms: u64,   // Set by system
    source: Source,            // self_report | inferred | mixed
    confidence: f32,           // [0.0, 1.0], default 1.0
    axes: BTreeMap<String, f32>, // axis_name -> value [0.0, 1.0]
}
```

### PromptContext

```rust
struct PromptContext {
    guidelines: Vec<String>,   // Behavioral instructions for LLM
    tone: String,              // e.g., "warm and supportive"
    verbosity: Verbosity,      // terse | concise | moderate | detailed
    flags: Vec<String>,        // e.g., ["needs_reassurance", "high_cognitive_load"]
}
```

### InferenceSource

```rust
enum InferenceSource {
    Linguistic { features_used: Vec<String> },  // From text analysis
    Delta { z_score: f32, metric: String },     // From baseline deviation
    Combined { source_count: usize },           // Multiple sources
    Prior { reason: String },                   // Default/no observation
}
```

## Canonical Axes (23)

All axes are normalized to `[0.0, 1.0]`.

| Category | Axis | Intent |
|----------|------|--------|
| **Cognitive** | `cognitive_load` | Current mental bandwidth |
| | `decision_fatigue` | Capacity for choices |
| | `tolerance_for_complexity` | Preference for detail |
| | `urgency_sensitivity` | Response to time pressure |
| **Emotional** | `emotional_intensity` | Current affect strength |
| | `emotional_stability` | Affect regulation capacity |
| | `anxiety_level` | Worry/concern state |
| | `need_for_reassurance` | Desire for validation |
| **Social** | `warmth` | Preference for friendly tone |
| | `formality` | Professional vs casual |
| | `boundary_strength` | Interaction limits |
| | `assertiveness` | Directness in expression |
| | `reciprocity_expectation` | Expected mutual engagement |
| **Preferences** | `ritual_need` | Preference for routine |
| | `directness_preference` | Direct vs indirect |
| | `verbosity_preference` | Response length |
| | `structure_preference` | Organized vs freeform |
| **Control** | `autonomy_need` | Self-direction need |
| | `information_need` | Detail requirement |
| | `predictability_need` | Surprise tolerance |
| | `control_need` | Influence over process |
| **Safety** | `vulnerability_state` | Current fragility |
| | `trust_level` | Openness to system |

## Inference Constraints

### Confidence Bounds

| Constraint | Value | Rationale |
|------------|-------|-----------|
| Max inference confidence | 0.7 | Self-report = 1.0, inference always subordinate |
| Min word count for inference | 10 | Below: confidence floor 0.5 |
| Stable word count | 50 | Full confidence factor |
| Baseline min observations | 5 | Before delta analysis activates |

### Axis-Specific Confidence Caps

| Axis | Max Confidence | Evidence Level |
|------|---------------|----------------|
| `formality` | 0.7 | Strong |
| `emotional_intensity` | 0.7 | Strong |
| `anxiety_level` | 0.6 | Moderate-Strong |
| `assertiveness` | 0.6 | Moderate-Strong |
| `urgency_sensitivity` | 0.5 | Moderate |
| `warmth` | 0.5 | Moderate |
| `tolerance_for_complexity` | 0.4 | Weak |
| `verbosity_preference` | 0.4 | Weak |

### Word Count Confidence Factor

```
f(words) = 0.5                           if words < 10
         = 0.5 + 0.5 * (words-10)/(50-10) if 10 <= words < 50
         = 1.0                            if words >= 50
```

## API Contracts

### POST /v1/state

**Request**
```json
{
  "user_id": "string (required)",
  "source": "self_report | inferred | mixed (default: self_report)",
  "confidence": "0.0-1.0 (default: 1.0)",
  "axes": { "axis_name": 0.0-1.0 },
  "message": "string (optional, triggers inference)"
}
```

**Behavior**: Patch semantics - only provided axes updated. Explicit axes override inferred.

### POST /v1/infer

**Request**
```json
{
  "message": "string (required)",
  "user_id": "string (optional, for baseline)",
  "include_features": "boolean (default: false)"
}
```

**Response**
```json
{
  "estimates": [{
    "axis": "string",
    "value": 0.0-1.0,
    "confidence": 0.0-0.7,
    "source": { "type": "linguistic", "features_used": [] }
  }],
  "features": { ... }  // Only if include_features=true
}
```

### GET /v1/context/{user_id}

**Response**
```json
{
  "guidelines": ["string"],
  "tone": "string",
  "verbosity": "terse | concise | moderate | detailed",
  "flags": ["string"]
}
```

## Performance Guarantees

| Operation | Target | Measured |
|-----------|--------|----------|
| Linguistic extraction | <200μs | ~100μs |
| Full inference pipeline | <500μs | ~350μs |
| Axis lookup | <10ns | 6ns |
| State translation | <500ns | 214ns |
| In-memory storage | <50μs | ~10μs |

## Invariants

1. **Self-Report Sovereignty**: `Source::SelfReport` axes always override `Source::Inferred`
2. **Confidence Ceiling**: Inferred confidence never exceeds 0.7
3. **No Content Storage**: Message text processed but never persisted
4. **Declared Sources**: Every `AxisEstimate` includes `InferenceSource`
5. **Bounded Values**: All axis values clamped to `[0.0, 1.0]`
6. **Patch Semantics**: `/v1/state` merges, never replaces entire state

## Security Defaults

| Setting | Default | Production Recommendation |
|---------|---------|---------------------------|
| Bind address | `127.0.0.1:8080` | Keep localhost, use reverse proxy |
| Body limit | 1 MB | Sufficient for text inference |
| Rate limit | 100 req/min | Adjust per use case |
| API auth | Disabled | Enable with `with_api_keys()` |
| Security headers | Enabled | Keep enabled |
| PII logging | Redacted | Never log user content |

## Feature Flags

| Flag | Crate | Effect |
|------|-------|--------|
| `inference` | `attuned-http` | Enables `/v1/infer`, adds `attuned-infer` + `dashmap` |

## Research References

| Feature | Source | Correlation |
|---------|--------|-------------|
| Negative emotion words | Dreaddit (Turcan 2019) | r=0.27 with stress |
| First-person pronouns | Dreaddit | r=0.32 with stress |
| Uncertainty markers | Dreaddit | r=0.31 with stress |
| Absolutist thinking | Al-Mosaiwi 2018 | Anxiety/depression marker |
| Formality detection | Multiple studies | r>0.3 with formality labels |

Validation: F1=0.68 on Dreaddit dataset (3,553 Reddit posts).
