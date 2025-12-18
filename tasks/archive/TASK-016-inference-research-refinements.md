# TASK-016: Inference Refinements Based on Research Validation

## Status
- [ ] Not Started
- [ ] In Progress
- [x] Completed
- [ ] Blocked

**Priority**: High
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: Refinement
**Depends On**: TASK-015 (validation informs priorities)
**Blocks**: None

## Task Description
Refine attuned-infer based on deep research findings (DEEP_RESEARCH.md). The research validated some of our approaches and identified gaps. This task implements the evidence-based improvements.

## Background
Research evidence summary:

| Axis | Evidence | Our Status | Action |
|------|----------|------------|--------|
| Formality | Strong | Good | Keep |
| Emotional intensity | Strong | Mislabeled | Rename axis |
| Anxiety/stress | Moderate-Strong | Missing signals | Add features |
| Assertiveness | Moderate | Good | Keep |
| Urgency | Moderate | Context-dependent | Add confidence penalty |
| Cognitive load | Weak | Not mapped | Keep omitted |

## Requirements

### 1. Add Negative Emotion Words (HIGH PRIORITY)
Research (Rook 2022, Pennebaker) shows negative affect words are the strongest anxiety signal.

**File**: `crates/attuned-infer/src/features.rs`

Add new word list:
```rust
const NEGATIVE_EMOTION_WORDS: &[&str] = &[
    // Anxiety-specific (LIWC Anxiety category)
    "worried", "worry", "worries", "anxious", "anxiety", "nervous",
    "afraid", "fear", "scared", "panic", "stressed", "stress",
    "tense", "uneasy", "dread", "dreading",
    // General negative affect
    "upset", "frustrated", "annoyed", "angry", "mad",
    "sad", "depressed", "hopeless", "miserable",
    "terrible", "awful", "horrible", "worst",
    // Distress markers
    "struggling", "suffering", "overwhelmed", "exhausted",
    "desperate", "helpless", "stuck", "lost",
];
```

Add to `LinguisticFeatures`:
```rust
pub negative_emotion_count: usize,
pub negative_emotion_density: f32,  // per sentence
```

### 2. Use First-Person Pronouns for Stress
We compute `first_person_ratio` but don't use it. Research shows high I/me usage correlates with stress and self-focus.

**File**: `crates/attuned-infer/src/engine.rs`

Update anxiety mapping:
```rust
// Current (incomplete):
let anxiety = f.uncertainty_score();

// Should be:
let anxiety = f.uncertainty_score() * 0.4
    + f.negative_emotion_density * 0.4
    + f.first_person_ratio * 0.2;
```

### 3. Add Word Count Confidence Scaling
Research says ~100 words minimum for stable style inference.

**File**: `crates/attuned-infer/src/estimate.rs` or `engine.rs`

```rust
/// Scale confidence based on text length.
/// Research suggests ~100 words for stable inference.
fn word_count_confidence_factor(word_count: usize) -> f32 {
    const MIN_WORDS: f32 = 20.0;   // Floor - below this, very low confidence
    const STABLE_WORDS: f32 = 100.0;  // At this point, full confidence

    if word_count < MIN_WORDS as usize {
        return 0.3;  // Very low confidence
    }

    ((word_count as f32 - MIN_WORDS) / (STABLE_WORDS - MIN_WORDS))
        .clamp(0.3, 1.0)
}
```

Apply to all axis estimates before returning.

### 4. Axis-Specific Confidence Caps
Different axes have different evidence strength.

**File**: `crates/attuned-infer/src/estimate.rs`

```rust
/// Maximum confidence by axis based on research evidence strength.
pub fn max_confidence_for_axis(axis: &str) -> f32 {
    match axis {
        // Strong evidence
        "formality" | "emotional_intensity" => 0.7,

        // Moderate-strong evidence
        "anxiety_level" | "assertiveness" | "directness_preference" => 0.6,

        // Moderate/context-dependent
        "urgency_sensitivity" | "warmth" | "ritual_need" => 0.5,

        // Weak evidence - style-dependent
        "tolerance_for_complexity" | "verbosity_preference" => 0.4,

        // Default for unknown
        _ => 0.5,
    }
}
```

### 5. Rename emotional_openness → emotional_intensity
We're measuring intensity (!, CAPS), not openness (willingness to engage emotionally). Fix the semantic mismatch.

**File**: `crates/attuned-infer/src/engine.rs`

```rust
// Change from:
mappings.push(("emotional_openness".into(), ...));

// To:
mappings.push(("emotional_intensity".into(), ...));
```

Note: `emotional_intensity` is not a canonical axis in attuned-core. Options:
- Add it to axes.rs as new axis
- Map to existing axis differently
- Keep as infer-specific output that doesn't map to core axes

### 6. Add Absolutist Words for Anxiety
Research links "always", "never", "completely" to anxious/depressive thinking.

```rust
const ABSOLUTIST_WORDS: &[&str] = &[
    "always", "never", "nothing", "everything", "completely",
    "totally", "absolutely", "entirely", "impossible", "definitely",
];

pub absolutist_count: usize,
```

Use as secondary anxiety signal (not certainty).

### 7. Reconsider Warmth Mapping
Current warmth formula is speculative:
```rust
let warmth = (1.0 - f.formality_score()) * 0.5
    + f.emotional_intensity() * 0.3
    + (f.politeness_count as f32 / 3.0).clamp(0.0, 1.0) * 0.2;
```

Options:
- Remove warmth inference entirely (low evidence)
- Keep but cap confidence at 0.4
- Find research supporting or refuting

## Implementation Order
1. Add negative emotion words (biggest gap)
2. Implement word count confidence scaling
3. Add axis-specific confidence caps
4. Use first-person pronouns in anxiety mapping
5. Rename emotional_openness → emotional_intensity
6. Add absolutist words
7. Review warmth (possibly remove)

## Testing
- Update existing tests for new features
- Add tests for confidence scaling
- Run Dreaddit validation (TASK-015) before and after to measure improvement

## Success Criteria
- [ ] Negative emotion words added and used in anxiety inference
- [ ] Word count confidence scaling implemented
- [ ] Axis-specific confidence caps implemented
- [ ] First-person pronouns contribute to anxiety score
- [ ] All tests pass
- [ ] Benchmarks still meet performance targets (<100μs)
- [ ] (Bonus) Improved Dreaddit F1 score vs baseline

## References
- DEEP_RESEARCH.md - full research findings
- Pennebaker LIWC documentation (word categories)
- Rook et al. 2022 - negative emotion in anxiety
- Tausczik & Pennebaker 2010 - LIWC psychometrics
