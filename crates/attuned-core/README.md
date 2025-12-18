# attuned-core

Core types and traits for [Attuned](https://github.com/JtPerez-Acle/Attuned) - human state representation for LLM systems.

## Overview

This crate provides the foundational types for representing human state as interpretable vectors across 23 canonical axes. It includes:

- **StateSnapshot** - Point-in-time human state representation
- **RuleTranslator** - Converts state to LLM-consumable `PromptContext`
- **Canonical Axes** - 23 validated axes across 6 categories with governance metadata

## Quick Start

```rust
use attuned_core::{StateSnapshot, RuleTranslator, Source};

// Create a state snapshot
let state = StateSnapshot::builder()
    .user_id("user_123")
    .source(Source::SelfReport)
    .axis("cognitive_load", 0.8)
    .axis("stress_level", 0.6)
    .axis("warmth", 0.7)
    .build()?;

// Translate to prompt context
let translator = RuleTranslator::default();
let context = translator.to_prompt_context(&state);

// Use context.guidelines in your LLM system prompt
println!("Tone: {}", context.tone);
println!("Verbosity: {:?}", context.verbosity);
println!("Guidelines: {}", context.guidelines);
```

## Axis Categories

| Category | Axes | Purpose |
|----------|------|---------|
| Cognitive | 4 | Mental load, decision fatigue, complexity tolerance, focus |
| Emotional | 4 | Stress, frustration, confidence, enthusiasm |
| Social | 5 | Formality, warmth, assertiveness, patience, trust |
| Preferences | 4 | Verbosity, detail level, example preference, pacing |
| Control | 4 | Autonomy, agency, transparency, override |
| Safety | 2 | Vulnerability, crisis indicators |

## Design Principles

- **No Actions**: Produces context, never executes actions
- **No Persuasion**: No engagement optimization or nudging
- **Transparent**: All axes have declared governance metadata
- **Self-Report Priority**: User-declared state always overrides inference

## License

Apache-2.0
