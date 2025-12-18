# Attuned Technical Documentation

## Document Index

| Document | Description |
|----------|-------------|
| [01-system-architecture.md](01-system-architecture.md) | System overview, crate dependencies, data flow, design principles |
| [02-axis-model.md](02-axis-model.md) | The 23-axis human state model, categories, governance metadata |
| [03-inference-engine.md](03-inference-engine.md) | NLP inference internals, feature extraction, Bayesian updates |
| [04-http-api-reference.md](04-http-api-reference.md) | Complete HTTP API documentation with examples |
| [05-integration-guide.md](05-integration-guide.md) | How to integrate Attuned with your application |
| [dreaddit-validation.md](dreaddit-validation.md) | Research validation results against Dreaddit dataset |

---

## Quick Reference

### What is Attuned?

Attuned is a Rust framework that:
1. Represents human state as **23 interpretable axes** (0.0-1.0)
2. **Infers** axis values from natural language text
3. **Translates** state into LLM-consumable context (guidelines, tone, flags)

It produces **context**, not actions. It never sends messages, makes API calls, or optimizes for engagement.

### Core Flow

```
User Message ──► Inference ──► State Storage ──► Translation ──► LLM Context
                    │                                               │
              "I'm worried"                                   "Use warm tone"
               anxiety: 0.72                                  "Be reassuring"
```

### Key APIs

```bash
# Store user state
POST /v1/state {"user_id": "x", "axes": {"warmth": 0.8}}

# Infer from text
POST /v1/infer {"message": "I'm stressed!!!"}

# Get LLM context
GET /v1/context/user123
```

### Key Guarantees

| Principle | Implementation |
|-----------|----------------|
| **Declared** | Every estimate includes `InferenceSource` |
| **Bounded** | Max inference confidence = 0.7 |
| **Subordinate** | Self-report always overrides inference |
| **Private** | Message text never stored, only derived axes |

---

## Crate Overview

```
attuned-core      # Types: StateSnapshot, PromptContext, 23 axes
attuned-store     # Storage: StateStore trait, MemoryStore
attuned-infer     # NLP: LinguisticExtractor, BayesianUpdater
attuned-http      # Server: Axum handlers, /v1/* endpoints
attuned-cli       # CLI: `cargo run -p attuned-cli -- serve`
```

---

## Getting Started

```bash
# 1. Build with inference
cargo build --release --features inference

# 2. Run server
cargo run -p attuned-cli -- serve

# 3. Test inference
curl -X POST http://localhost:8080/v1/infer \
  -H "Content-Type: application/json" \
  -d '{"message": "I urgently need help!!!", "include_features": true}'

# 4. Get context for LLM
curl http://localhost:8080/v1/context/user123
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v0.0.9 | 2025-12-16 | Added inference integration, Dreaddit validation |
| v0.0.8 | - | Initial HTTP server, core types |

---

## Research Foundation

The inference engine is validated against peer-reviewed research:

- **Dreaddit Dataset** (Turcan & McKeown 2019): 3,553 Reddit posts with stress labels
- **LIWC Categories**: Negative affect (r=0.40), First-person (r=0.32)
- **Absolutist Thinking** (Al-Mosaiwi 2018): Anxiety/depression markers
- **Formality Detection**: Multiple studies (r > 0.3)

See [dreaddit-validation.md](dreaddit-validation.md) for full validation results.
