# Attuned System Architecture

## Overview

Attuned is a Rust framework for representing human state as interpretable vectors and translating that state into machine-consumable interaction constraints for LLM/agent systems. It produces **context**, not actions.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              ATTUNED SYSTEM                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Client Application                                                        │
│         │                                                                   │
│         ▼                                                                   │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                        HTTP API (attuned-http)                       │  │
│   │  POST /v1/state    POST /v1/infer    GET /v1/context/{user_id}      │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│         │                    │                       │                      │
│         ▼                    ▼                       ▼                      │
│   ┌───────────┐    ┌─────────────────┐    ┌─────────────────────┐          │
│   │ StateStore│    │ InferenceEngine │    │   RuleTranslator    │          │
│   │ (storage) │    │  (attuned-infer)│    │   (attuned-core)    │          │
│   └───────────┘    └─────────────────┘    └─────────────────────┘          │
│         │                    │                       │                      │
│         ▼                    ▼                       ▼                      │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                     StateSnapshot (23 Axes)                          │  │
│   │   cognitive_load: 0.7  │  warmth: 0.6  │  anxiety_level: 0.4  │ ... │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                         PromptContext                                │  │
│   │   guidelines: ["Use simple language", "Be patient"]                  │  │
│   │   tone: "warm and supportive"                                        │  │
│   │   verbosity: "concise"                                               │  │
│   │   flags: ["high_cognitive_load", "needs_reassurance"]                │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│                           LLM/Agent System                                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Crate Dependency Graph

```
attuned-cli ─────► attuned-http ─────► attuned-store ─────► attuned-core
                        │                    │
                        │                    ▼
                        │              attuned-qdrant
                        │
                        ▼
                   attuned-infer
```

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| `attuned-core` | Core types, 23 axes, translator | `StateSnapshot`, `RuleTranslator`, `PromptContext`, `CANONICAL_AXES` |
| `attuned-store` | Storage trait + in-memory backend | `StateStore` trait, `MemoryStore` |
| `attuned-qdrant` | Qdrant vector DB backend | `QdrantStore`, `QdrantConfig` |
| `attuned-http` | Axum HTTP server | `Server`, `ServerConfig`, `AppState` |
| `attuned-infer` | NLP inference engine | `InferenceEngine`, `LinguisticExtractor`, `Baseline` |
| `attuned-cli` | CLI entry point | - |

## Core Design Principles

### 1. Declared Inference
Every inferred value includes its source and reasoning:
```rust
pub enum InferenceSource {
    SelfReport,                    // User explicitly stated
    Linguistic { features_used },  // Derived from text analysis
    Delta { z_score, metric },     // Deviation from baseline
    Combined { sources, weights }, // Multiple sources merged
    Decayed { original, age },     // Time-degraded estimate
    Prior { reason },              // Default assumption
}
```

### 2. Bounded Confidence
Inferred values are capped at 0.7 confidence. Self-report = 1.0.
```rust
pub const MAX_INFERRED_CONFIDENCE: f32 = 0.7;
```

### 3. Self-Report Sovereignty
Self-report **always** overrides inference:
```rust
// In InferredState::update()
if existing.source.is_self_report() && new.source.is_inferred() {
    return; // Inference cannot override self-report
}
```

### 4. No Actions, Only Context
Attuned produces `PromptContext` for LLMs - it never executes actions, sends messages, or makes API calls.

## Data Flow

### State Ingestion
```
1. Client POSTs StateSnapshot (user_id + axes + source)
2. Optional: Message text triggers inference
3. Inference merges with explicit axes (explicit wins)
4. Store persists via StateStore::upsert_latest()
```

### Context Generation
```
1. Client GETs /v1/context/{user_id}
2. Store retrieves latest StateSnapshot
3. RuleTranslator applies rules to generate PromptContext
4. PromptContext returned to client for LLM conditioning
```

### Inference Pipeline (when enabled)
```
[Message Text]
      │
      ▼
┌─────────────────┐
│ LinguisticExtractor │  Extract features (~100μs)
└─────────────────┘
      │
      ▼
┌─────────────────┐
│  DeltaAnalyzer  │  Compare to user's baseline
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ BayesianUpdater │  Update state estimates
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ InferredState   │  23 axes with confidence
└─────────────────┘
```

## Security Model

- **No PII in logs**: Message text is never logged
- **Rate limiting**: Configurable per-IP limits
- **API key auth**: Optional Bearer token authentication
- **GDPR compliance**: DELETE /v1/state/{user_id} removes all data

## Performance Characteristics

| Operation | Latency | Notes |
|-----------|---------|-------|
| Linguistic extraction | ~100μs | Pure Rust, no regex |
| Full inference | <500μs | Including baseline comparison |
| State storage | ~10μs | In-memory store |
| Context generation | ~50μs | Rule evaluation |

## Feature Flags

| Feature | Crate | Effect |
|---------|-------|--------|
| `inference` | attuned-http | Enables /v1/infer endpoint and message-based inference |
