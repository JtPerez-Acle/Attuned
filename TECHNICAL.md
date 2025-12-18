# Technical Specification

Quick-reference specification for Attuned developers. For detailed documentation, see [`tasks/reports/`](tasks/reports/).

## Python API (Recommended)

```python
from attuned import Attuned

# Simple API - set any axes, rest default to neutral (0.5)
state = Attuned(
    verbosity_preference=0.2,  # Brief
    warmth=0.9,                # Warm
    cognitive_load=0.8,        # User is overwhelmed
)

# Get prompt context string - works with ANY LLM
system_prompt = f"You are an assistant.\n\n{state.prompt()}"

# Presets for common patterns
state = Attuned.presets.anxious_user()     # Warm, reassuring
state = Attuned.presets.busy_executive()   # Brief, formal
state = Attuned.presets.learning_student() # Detailed, patient

# Integrations (thin wrappers)
from attuned.integrations.openai import AttunedOpenAI
from attuned.integrations.anthropic import AttunedAnthropic
from attuned.integrations.litellm import AttunedLiteLLM  # 100+ providers
```

### Python Types

| Type | Description |
|------|-------------|
| `Attuned` | Main class - set axes, get prompt context |
| `Attuned.presets` | Pre-configured states (anxious_user, busy_executive, etc.) |
| `StateSnapshot` | Low-level state container (advanced) |
| `RuleTranslator` | Converts state to prompt context (advanced) |
| `PromptContext` | Structured output with guidelines, tone, verbosity |
| `AttunedClient` | HTTP client for distributed deployments |

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
| **Emotional** | `emotional_openness` | Willingness to share feelings |
| | `emotional_stability` | Affect regulation capacity |
| | `anxiety_level` | Worry/concern state |
| | `need_for_reassurance` | Desire for validation |
| **Social** | `warmth` | Preference for friendly tone |
| | `formality` | Professional vs casual |
| | `boundary_strength` | Interaction limits |
| | `assertiveness` | Directness in expression |
| | `reciprocity_expectation` | Expected mutual engagement |
| **Preferences** | `ritual_need` | Preference for routine |
| | `transactional_preference` | Task-focused vs relational |
| | `verbosity_preference` | Response length |
| | `directness_preference` | Direct vs indirect |
| **Control** | `autonomy_preference` | Self-direction preference |
| | `suggestion_tolerance` | Openness to suggestions |
| | `interruption_tolerance` | Tolerance for interruptions |
| | `reflection_vs_action_bias` | Thinking vs doing preference |
| **Safety** | `stakes_awareness` | Importance of current context |
| | `privacy_sensitivity` | Data sharing comfort |

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

## Behavioral Validation (LLM Effects)

Tested with GPT-4o-mini, n=20 per condition, Welch's t-test:

| Axis | Effect | p-value | Cohen's d |
|------|--------|---------|-----------|
| `verbosity_preference=0.2` | -70% response length | <0.0001 | 7.40 |
| `verbosity_preference=0.9` | +31% response length | 0.0002 | 1.16 |
| `warmth=0.9` | 6.1 warm indicators/response | <0.0001 | 4.11 |
| `formality=0.9` | Formal language detected | 0.0086 | 0.83 |
| `cognitive_load=0.9` | -82% multi-step plans | <0.0001 | 1.90 |

**Total: 11/13 tests passed (85%)**

## Research References

| Feature | Source | Correlation |
|---------|--------|-------------|
| Negative emotion words | Dreaddit (Turcan 2019) | r=0.27 with stress |
| First-person pronouns | Dreaddit | r=0.32 with stress |
| Uncertainty markers | Dreaddit | r=0.31 with stress |
| Absolutist thinking | Al-Mosaiwi 2018 | Anxiety/depression marker |
| Formality detection | Multiple studies | r>0.3 with formality labels |

Validation: F1=0.68 on Dreaddit dataset (3,553 Reddit posts).
