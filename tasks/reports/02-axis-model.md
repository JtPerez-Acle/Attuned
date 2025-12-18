# The 23-Axis Human State Model

## Overview

Attuned represents human state as a vector of 23 normalized axes (0.0-1.0), organized into 6 categories. Each axis has governance metadata defining its intended use and forbidden applications.

## Axis Categories

### 1. Cognitive (4 axes)
Mental processing state - how the person is thinking.

| Axis | Range | Low (0.0) | High (1.0) | Inference Evidence |
|------|-------|-----------|------------|-------------------|
| `cognitive_load` | 0.0-1.0 | Clear-headed, spare capacity | Overwhelmed, at capacity | Weak - don't infer from text |
| `decision_fatigue` | 0.0-1.0 | Fresh, decisive | Exhausted, avoidant | Weak - context-dependent |
| `tolerance_for_complexity` | 0.0-1.0 | Needs simplicity | Welcomes nuance | Moderate |
| `urgency_sensitivity` | 0.0-1.0 | Relaxed about time | Time-pressured | Moderate |

### 2. Emotional (4 axes)
Affective state - how the person is feeling.

| Axis | Range | Low (0.0) | High (1.0) | Inference Evidence |
|------|-------|-----------|------------|-------------------|
| `emotional_intensity` | 0.0-1.0 | Muted, flat affect | Highly expressive | Strong (r > 0.3) |
| `emotional_stability` | 0.0-1.0 | Volatile, reactive | Steady, resilient | Moderate |
| `anxiety_level` | 0.0-1.0 | Calm, secure | Anxious, worried | Strong (Dreaddit validated) |
| `need_for_reassurance` | 0.0-1.0 | Self-assured | Needs validation | Moderate |

### 3. Social (5 axes)
Interpersonal preferences - how the person relates to others.

| Axis | Range | Low (0.0) | High (1.0) | Inference Evidence |
|------|-------|-----------|------------|-------------------|
| `warmth` | 0.0-1.0 | Distant, formal | Warm, friendly | Moderate |
| `formality` | 0.0-1.0 | Casual, relaxed | Formal, professional | Strong (multiple studies) |
| `boundary_strength` | 0.0-1.0 | Open, permeable | Firm boundaries | Moderate |
| `assertiveness` | 0.0-1.0 | Deferential | Direct, confident | Moderate-Strong |
| `reciprocity_expectation` | 0.0-1.0 | Gives freely | Expects exchange | Weak |

### 4. Preferences (4 axes)
Interaction style preferences.

| Axis | Range | Low (0.0) | High (1.0) | Inference Evidence |
|------|-------|-----------|------------|-------------------|
| `ritual_need` | 0.0-1.0 | Gets to point | Values pleasantries | Moderate |
| `directness_preference` | 0.0-1.0 | Indirect, diplomatic | Direct, explicit | Moderate-Strong |
| `verbosity_preference` | 0.0-1.0 | Terse, minimal | Detailed, expansive | Weak |
| `structure_preference` | 0.0-1.0 | Fluid, organic | Organized, systematic | Moderate |

### 5. Control (4 axes)
Agency and autonomy needs.

| Axis | Range | Low (0.0) | High (1.0) | Inference Evidence |
|------|-------|-----------|------------|-------------------|
| `autonomy_need` | 0.0-1.0 | Welcomes guidance | Self-directed | Moderate |
| `information_need` | 0.0-1.0 | Trusts defaults | Wants full picture | Moderate |
| `predictability_need` | 0.0-1.0 | Comfortable with surprise | Needs advance notice | Moderate |
| `control_need` | 0.0-1.0 | Delegates easily | Maintains control | Moderate |

### 6. Safety (2 axes)
Vulnerability and risk state.

| Axis | Range | Low (0.0) | High (1.0) | Inference Evidence |
|------|-------|-----------|------------|-------------------|
| `vulnerability_state` | 0.0-1.0 | Resilient, buffered | Fragile, at-risk | Moderate |
| `trust_level` | 0.0-1.0 | Guarded, skeptical | Open, trusting | Weak |

## Axis Definition Structure

Each axis is defined with governance metadata:

```rust
pub struct AxisDefinition {
    /// Unique identifier (snake_case)
    pub name: &'static str,

    /// Human-readable description
    pub description: &'static str,

    /// Category grouping
    pub category: AxisCategory,

    /// Meaning of 0.0 value
    pub low_label: &'static str,

    /// Meaning of 1.0 value
    pub high_label: &'static str,

    /// Default value when unknown
    pub default_value: f32,

    /// Intended legitimate uses
    pub intent: &'static str,

    /// Explicitly forbidden applications
    pub forbidden_uses: &'static [&'static str],
}
```

## Example: Anxiety Level Axis

```rust
pub const ANXIETY_LEVEL: AxisDefinition = AxisDefinition {
    name: "anxiety_level",
    description: "Current anxiety or worry state",
    category: AxisCategory::Emotional,
    low_label: "calm",
    high_label: "anxious",
    default_value: 0.3,
    intent: "Adjust reassurance and pacing to reduce distress",
    forbidden_uses: &[
        "Targeting vulnerable users for manipulation",
        "Exploiting anxiety to drive engagement",
        "Discriminating based on mental state",
    ],
};
```

## Confidence-Weighted Axes

Each axis value is paired with confidence metadata:

```rust
pub struct AxisEstimate {
    pub axis: String,           // e.g., "anxiety_level"
    pub value: f32,             // 0.0-1.0
    pub confidence: f32,        // 0.0-1.0 (capped at 0.7 for inference)
    pub variance: f32,          // For Bayesian updates
    pub source: InferenceSource,
    pub timestamp: DateTime<Utc>,
}
```

## Axis-Specific Confidence Caps

Based on research evidence strength, each axis has a maximum confidence:

| Evidence Level | Max Confidence | Axes |
|----------------|----------------|------|
| Strong | 0.7 | `formality`, `emotional_intensity` |
| Moderate-Strong | 0.6 | `anxiety_level`, `assertiveness`, `directness_preference` |
| Moderate | 0.5 | `urgency_sensitivity`, `warmth`, `ritual_need` |
| Weak | 0.4 | `tolerance_for_complexity`, `verbosity_preference` |

## StateSnapshot Structure

The complete state for a user at a point in time:

```rust
pub struct StateSnapshot {
    pub user_id: String,
    pub updated_at_unix_ms: i64,
    pub source: Source,        // SelfReport, Inferred, Mixed
    pub confidence: f32,       // Overall confidence
    pub axes: BTreeMap<String, f32>,  // axis_name → value
}
```

## Validation Rules

- All axis values must be in [0.0, 1.0]
- User IDs cannot be empty
- Unknown axis names are allowed (forward compatibility)
- Canonical axes are immutable after v1.0

## Translation to Context

The `RuleTranslator` converts axis values to `PromptContext`:

```rust
// Example rules (simplified)
if cognitive_load > 0.7 {
    guidelines.push("Use simple, clear language");
    guidelines.push("Break complex topics into steps");
    flags.push("high_cognitive_load");
}

if warmth > 0.6 {
    tone = "warm and supportive";
}

if anxiety_level > 0.6 {
    guidelines.push("Acknowledge concerns before solutions");
    flags.push("needs_reassurance");
}
```
