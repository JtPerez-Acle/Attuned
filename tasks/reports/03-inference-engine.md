# Inference Engine Technical Documentation

## Overview

The `attuned-infer` crate provides fast (<500μs), auditable inference of human state axes from natural language text. All inferences are:

- **Declared**: Every estimate includes `InferenceSource` explaining derivation
- **Bounded**: Maximum confidence 0.7 (self-report = 1.0)
- **Subordinate**: Self-report always overrides inference

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         InferenceEngine                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   [Message Text]                                                        │
│         │                                                               │
│         ▼                                                               │
│   ┌─────────────────────────────────────────────────────────────────┐  │
│   │              LinguisticExtractor (~100μs)                        │  │
│   │                                                                  │  │
│   │   Input: "I'm really worried about this deadline!!!"            │  │
│   │                                                                  │  │
│   │   Output: LinguisticFeatures {                                  │  │
│   │       word_count: 7,                                            │  │
│   │       negative_emotion_count: 1,  // "worried"                  │  │
│   │       exclamation_ratio: 0.43,    // 3/7 chars                  │  │
│   │       first_person_ratio: 0.14,   // "I'm"                      │  │
│   │       urgency_word_count: 1,      // "deadline"                 │  │
│   │       ...                                                       │  │
│   │   }                                                             │  │
│   └─────────────────────────────────────────────────────────────────┘  │
│         │                                                               │
│         ▼                                                               │
│   ┌─────────────────────────────────────────────────────────────────┐  │
│   │              DeltaAnalyzer (~200μs)                              │  │
│   │                                                                  │  │
│   │   Compares current message to user's Baseline                   │  │
│   │                                                                  │  │
│   │   Output: DeltaSignals {                                        │  │
│   │       length_z: 0.5,      // Slightly longer than usual         │  │
│   │       urgency_z: 2.1,     // Much more urgent than usual        │  │
│   │       emotional_z: 1.8,   // More emotional than usual          │  │
│   │       ...                                                       │  │
│   │   }                                                             │  │
│   └─────────────────────────────────────────────────────────────────┘  │
│         │                                                               │
│         ▼                                                               │
│   ┌─────────────────────────────────────────────────────────────────┐  │
│   │              BayesianUpdater (~50μs)                             │  │
│   │                                                                  │  │
│   │   Combines priors + observations with uncertainty tracking      │  │
│   │                                                                  │  │
│   │   Output: InferredState {                                       │  │
│   │       anxiety_level: 0.72 (conf: 0.58)                         │  │
│   │       urgency_sensitivity: 0.81 (conf: 0.45)                   │  │
│   │       emotional_intensity: 0.68 (conf: 0.52)                   │  │
│   │   }                                                             │  │
│   └─────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. LinguisticExtractor

Extracts linguistic features from text without using LLMs or heavy NLP libraries.

#### Feature Categories

**Basic Metrics**
```rust
pub struct LinguisticFeatures {
    pub char_count: usize,
    pub word_count: usize,
    pub sentence_count: usize,
    pub avg_word_length: f32,
    pub avg_sentence_length: f32,
}
```

**Punctuation Signals**
```rust
pub exclamation_ratio: f32,  // !/total_chars - emotional intensity
pub question_ratio: f32,      // ?/total_chars - uncertainty/seeking
pub caps_ratio: f32,          // CAPS/total - emphasis/urgency
pub ellipsis_count: usize,    // ... - hesitation/trailing off
```

**Lexical Categories**
```rust
// Uncertainty markers (validated against stress)
pub hedge_count: usize,        // "maybe", "perhaps", "I think"
pub hedge_density: f32,        // hedge_count / word_count

// Anxiety signals (Dreaddit validated, r=0.27)
pub negative_emotion_count: usize,  // "worried", "stressed", "anxious"
pub negative_emotion_density: f32,

// Absolutist thinking (Al-Mosaiwi 2018)
pub absolutist_count: usize,   // "always", "never", "completely"
pub absolutist_density: f32,

// Urgency markers
pub urgency_word_count: usize, // "asap", "urgent", "immediately"

// Formality indicators
pub politeness_count: usize,   // "please", "thank you"
pub contraction_count: usize,  // "don't", "can't" (informal)
pub first_person_ratio: f32,   // "I", "me", "my" (self-focus)
```

**Readability Metrics**
```rust
pub flesch_reading_ease: f32,     // 0-100, higher = easier
pub flesch_kincaid_grade: f32,    // US grade level
```

#### Word Lists

**Hedge Words** (uncertainty markers)
```rust
const HEDGE_WORDS: &[&str] = &[
    "maybe", "perhaps", "possibly", "probably", "might",
    "could", "somewhat", "sort of", "kind of", "i think",
    "i guess", "i suppose", "it seems", "apparently",
    "arguably", "presumably", "likely", "unlikely",
];
```

**Negative Emotion Words** (anxiety/stress signals)
```rust
const NEGATIVE_EMOTION_WORDS: &[&str] = &[
    // Anxiety-specific
    "worried", "worry", "anxious", "nervous", "afraid",
    "fear", "scared", "panic", "stressed", "tense",
    // General negative affect
    "upset", "frustrated", "annoyed", "angry", "sad",
    "depressed", "hopeless", "miserable", "terrible",
    // Distress markers
    "struggling", "suffering", "overwhelmed", "exhausted",
    "desperate", "helpless", "confused", "lost",
];
```

**Absolutist Words** (cognitive distortion markers)
```rust
const ABSOLUTIST_WORDS: &[&str] = &[
    "always", "never", "completely", "totally", "absolutely",
    "entirely", "nothing", "everything", "everyone", "nobody",
    "impossible", "definitely", "certainly", "constantly",
];
```

**Urgency Words**
```rust
const URGENCY_WORDS: &[&str] = &[
    "urgent", "asap", "immediately", "emergency", "critical",
    "deadline", "now", "hurry", "quick", "fast", "rush",
];
```

### 2. Feature-to-Axis Mapping

The `linguistic_to_axes()` function maps features to axis estimates:

```rust
// Formality (strong evidence)
let formality = f.formality_score();  // 0.0 = casual, 1.0 = formal
if formality > 0.3 || formality < 0.7 {
    axes.push(("formality", formality, vec!["formality_score"]));
}

// Anxiety (Dreaddit validated)
let anxiety = f.anxiety_score();  // Combines negative emotions + first-person + uncertainty
if anxiety > 0.3 {
    axes.push(("anxiety_level", anxiety, vec!["anxiety_score"]));
}

// Urgency
let urgency = f.urgency_score();  // Combines urgency words + caps + exclamations
if urgency > 0.3 {
    axes.push(("urgency_sensitivity", urgency, vec!["urgency_score"]));
}

// Emotional intensity (strong evidence)
let intensity = f.emotional_intensity();  // Exclamations + caps + emotion words
if intensity > 0.3 {
    axes.push(("emotional_intensity", intensity, vec!["emotional_intensity"]));
}
```

### 3. Anxiety Score Calculation

Research-validated formula (Dreaddit dataset, F1=0.68):

```rust
pub fn anxiety_score(&self) -> f32 {
    // Weights based on Dreaddit validation correlations
    let neg_emotion = (self.negative_emotion_density / 2.0).clamp(0.0, 1.0);  // r=0.27
    let first_person = (self.first_person_ratio * 5.0).clamp(0.0, 1.0);       // r=0.32
    let uncertainty = self.uncertainty_score();                               // r=0.31
    let absolutist = (self.absolutist_density / 2.0).clamp(0.0, 1.0);        // r=0.07

    // Weighted combination
    (0.35 * neg_emotion +
     0.35 * first_person +
     0.20 * uncertainty +
     0.10 * absolutist).clamp(0.0, 1.0)
}
```

### 4. DeltaAnalyzer (Baseline Deviation)

Detects when a user is "acting different than usual":

```rust
pub struct Baseline {
    observations: VecDeque<BaselineObservation>,
    max_observations: usize,  // Default: 20
}

pub struct BaselineObservation {
    length: f64,       // Message length
    urgency: f64,      // Urgency score
    emotional: f64,    // Emotional intensity
    complexity: f64,   // Readability
    formality: f64,    // Formality score
    uncertainty: f64,  // Hedge density
}
```

**Z-Score Calculation**
```rust
pub fn analyze(&self, features: &LinguisticFeatures) -> Option<DeltaSignals> {
    if self.observations.len() < 5 {
        return None;  // Need baseline first
    }

    let (mean, std) = self.stats();

    DeltaSignals {
        length_z: (features.word_count - mean.length) / std.length,
        urgency_z: (features.urgency_score() - mean.urgency) / std.urgency,
        emotional_z: (features.emotional_intensity() - mean.emotional) / std.emotional,
        // ...
    }
}
```

**Axis Adjustments**
```rust
// Z-score > 1.5 triggers adjustment
if signals.urgency_z > 1.5 {
    adjustments.push(("urgency_sensitivity", +0.2));
}
if signals.emotional_z > 1.5 {
    adjustments.push(("emotional_intensity", +0.2));
    adjustments.push(("emotional_stability", -0.1));  // Inverse relationship
}
```

### 5. BayesianUpdater

Combines observations with priors using Bayesian inference:

```rust
pub struct Prior {
    pub mean: f32,      // Expected value
    pub variance: f32,  // Uncertainty
}

pub fn update(&mut self, axis: &str, observation: f32, confidence: f32) -> f32 {
    let prior = self.priors.get(axis).unwrap_or(&Prior::neutral());

    // Weighted combination based on variances
    let obs_variance = AxisEstimate::confidence_to_variance(confidence);
    let weight = prior.variance / (prior.variance + obs_variance);

    let new_mean = prior.mean + weight * (observation - prior.mean);
    let new_variance = prior.variance * (1.0 - weight);

    // Update prior for next observation
    self.priors.insert(axis, Prior { mean: new_mean, variance: new_variance });

    new_mean
}
```

### 6. Confidence Scaling

**Word Count Factor**
Short messages have reduced confidence:

```rust
pub fn word_count_confidence_factor(word_count: usize) -> f32 {
    const MIN_WORDS: f32 = 10.0;    // Below: reduced confidence
    const STABLE_WORDS: f32 = 50.0; // At this: full confidence

    if word_count < MIN_WORDS as usize {
        return 0.5;  // Floor
    }

    let factor = (word_count - MIN_WORDS) / (STABLE_WORDS - MIN_WORDS);
    0.5 + 0.5 * factor.clamp(0.0, 1.0)  // Range [0.5, 1.0]
}
```

**Axis-Specific Caps**
Based on research evidence:

```rust
pub fn max_confidence_for_axis(axis: &str) -> f32 {
    match axis {
        "formality" | "emotional_intensity" => 0.7,       // Strong evidence
        "anxiety_level" | "assertiveness" => 0.6,         // Moderate-strong
        "urgency_sensitivity" | "warmth" => 0.5,          // Moderate
        "tolerance_for_complexity" | "verbosity" => 0.4,  // Weak
        _ => 0.5,  // Unknown
    }
}
```

## Usage Example

```rust
use attuned_infer::{InferenceEngine, InferredState};

let engine = InferenceEngine::default();

// Simple inference
let state = engine.infer("I'm really worried about this deadline!!!");

for estimate in state.all() {
    println!("{}: {:.2} (conf: {:.2}, source: {})",
        estimate.axis,
        estimate.value,
        estimate.confidence,
        estimate.source.summary()
    );
}
// Output:
// anxiety_level: 0.72 (conf: 0.58, source: linguistic(anxiety_score))
// urgency_sensitivity: 0.81 (conf: 0.45, source: linguistic(urgency_score))
// emotional_intensity: 0.68 (conf: 0.52, source: linguistic(emotional_intensity))
```

## Research Validation

Validated against Dreaddit dataset (3,553 Reddit posts with stress labels):

| Model | Precision | Recall | F1 | Improvement |
|-------|-----------|--------|-----|-------------|
| v1 (hedges only) | 0.576 | 0.588 | 0.582 | baseline |
| v2 (+ neg emotions) | 0.662 | 0.696 | 0.679 | +16.7% |
| LIWC comparison | 0.722 | 0.759 | 0.740 | reference |

Key correlations with stress label:
- `lex_liwc_negemo`: r=0.395 (strong)
- `first_person_ratio`: r=0.321 (strong)
- `uncertainty_score`: r=0.309 (moderate-strong)
- `negative_emotion_density`: r=0.266 (moderate)
