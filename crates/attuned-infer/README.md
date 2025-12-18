# attuned-infer

Fast, transparent inference of human state axes from natural language for [Attuned](https://github.com/JtPerez-Acle/Attuned).

## Overview

This crate provides linguistic analysis to infer user state from text. All inferences are:

- **Fast**: <35 microseconds per message
- **Transparent**: Every estimate includes explanation of features that drove it
- **Bounded**: Maximum confidence 0.7 (self-report = 1.0)
- **Subordinate**: Self-report always overrides inference

## Quick Start

```rust
use attuned_infer::{InferenceEngine, LinguisticExtractor};

let engine = InferenceEngine::new();

// Analyze a user message
let message = "I really need this done ASAP!!! Can you help???";
let inferred = engine.infer_from_text(message)?;

// Check inferred axes with sources
for (axis, estimate) in inferred.estimates() {
    println!("{}: {:.2} (confidence: {:.2})",
        axis, estimate.value, estimate.confidence);
    println!("  Source: {:?}", estimate.source);
}
```

## Validated Axes

| Axis | Evidence Level | Notes |
|------|---------------|-------|
| Formality | Strong | Pronoun usage, contractions |
| Emotional intensity | Strong | Punctuation, capitalization |
| Anxiety/stress | Moderate-Strong | Hedge words, uncertainty markers |
| Assertiveness | Moderate | Command forms, hedging |
| Urgency | Moderate | Time words, punctuation |

## Architecture

```
[Message] -> LinguisticExtractor -> LinguisticFeatures
                                          |
[History] -> DeltaAnalyzer ---------> BayesianUpdater -> InferredState
                                          |
[Self-Report] ----------------------> Override (confidence -> 1.0)
```

## Features Extracted

- Hedge words ("maybe", "perhaps", "I think")
- Urgency markers ("ASAP", "urgent", "now")
- Formality indicators (contractions, pronouns)
- Emotional punctuation (!, ?, CAPS)
- Certainty language ("definitely", "absolutely")

## License

Apache-2.0
