# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Attuned is a Rust framework for representing human state as interpretable vectors (23 axes across 6 categories) and translating that state into machine-consumable interaction constraints for LLM/agent systems. It produces **context**, not actions.

**Hard constraints from the manifesto:**
- Never execute actions (no messages, events, transactions, API calls)
- Never optimize for persuasion (no engagement/conversion metrics, no nudges)
- Never covertly infer (self-report overrides inference)
- Never store content (state descriptors only, no message history)

## Build Commands

```bash
cargo build --workspace          # Build all crates
cargo test --workspace           # Run all tests
cargo test -p attuned-core       # Test specific crate
cargo test -p attuned-core translator::tests::test_high_cognitive_load  # Single test
cargo clippy --workspace         # Lint
cargo fmt --all                  # Format
cargo bench -p attuned-core      # Run benchmarks
cargo audit                      # Security audit
cargo run -p attuned-cli -- serve  # Run HTTP server (127.0.0.1:8080)
```

## Architecture

Six crates in dependency order:

```
attuned-cli → attuned-http → attuned-store → attuned-core
                   ↓          attuned-qdrant ↗
              attuned-infer ──────────────────↗
```

| Crate | Purpose | Key Exports |
|-------|---------|-------------|
| `attuned-core` | Core types, 23 axes, translator, telemetry | `StateSnapshot`, `RuleTranslator`, `PromptContext`, `CANONICAL_AXES` |
| `attuned-store` | Storage trait + in-memory backend | `StateStore` trait, `MemoryStore` |
| `attuned-qdrant` | Qdrant vector DB backend (stubs) | `QdrantStore`, `QdrantConfig` |
| `attuned-http` | Axum HTTP server with auth/rate-limit | `Server`, `ServerConfig`, `AuthConfig` |
| `attuned-infer` | NLP inference of axes from text | `InferenceEngine`, `LinguisticExtractor`, `Baseline` |
| `attuned-cli` | CLI wrapper | - |

## Key Patterns

**State Flow:**
1. Client POSTs `StateSnapshot` (user_id + axes 0.0-1.0 + source)
2. Store persists via `StateStore::upsert_latest()`
3. Client GETs context → `RuleTranslator::to_prompt_context()` generates `PromptContext`
4. `PromptContext` (guidelines, tone, verbosity, flags) conditions LLM behavior

**Axis System (`attuned-core/src/axes.rs`):**
- 23 canonical axes with governance metadata (`intent`, `forbidden_uses`)
- Categories: Cognitive(4), Emotional(4), Social(5), Preferences(4), Control(4), Safety(2)
- Axes are immutable after v1.0; adding new axes requires full `AxisDefinition` with governance review
- All values normalized to [0.0, 1.0]

**Adding a New Axis:**
1. Define `AxisDefinition` constant in `axes.rs` with all fields including `forbidden_uses`
2. Add to `CANONICAL_AXES` array
3. Update test count in `test_canonical_axes_count`
4. Add translation rules in `RuleTranslator` if needed

## API Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/v1/state` | Upsert state (patch semantics) |
| GET | `/v1/state/{user_id}` | Get latest state |
| GET | `/v1/context/{user_id}` | Get translated PromptContext |
| DELETE | `/v1/state/{user_id}` | Delete state (GDPR) |
| POST | `/v1/translate` | Translate arbitrary state |
| GET | `/health`, `/ready` | Health checks |

## Inference System (`attuned-infer`)

Fast (<35μs), auditable inference of axes from natural language. All inferences are:
- **Declared** - every estimate includes `InferenceSource` explaining which features drove it
- **Bounded** - max confidence 0.7 for inferred values (self-report = 1.0)
- **Subordinate** - self-report always overrides inference

**Architecture:**
```
[Message] → LinguisticExtractor → LinguisticFeatures
                                        ↓
[History] → DeltaAnalyzer ────────→ BayesianUpdater → InferredState
                                        ↓
[Self-Report] ──────────────────→ Override (σ² → 0)
```

**Research Validation (see DEEP_RESEARCH.md):**
| Axis | Evidence | Status |
|------|----------|--------|
| Formality | Strong | ✅ Validated |
| Emotional intensity | Strong | ✅ Validated |
| Anxiety/stress | Moderate-Strong | ⚠️ Needs negative emotion words |
| Assertiveness | Moderate | ✅ Validated |
| Urgency | Moderate | ⚠️ Context-dependent |
| Cognitive load | Weak | ❌ Don't infer from text alone |

**Key files:**
- `features.rs` - linguistic feature extraction (hedge words, urgency, formality)
- `bayesian.rs` - Bayesian state estimation with uncertainty tracking
- `delta.rs` - baseline deviation detection ("you're acting different than usual")
- `engine.rs` - orchestrates the pipeline

## Code Style

- `#![deny(missing_docs)]` on all crates - all public items need doc comments
- Tests in same file as code under `#[cfg(test)] mod tests`
- Translators are pure functions (no learning, adaptation, or inference)
- Storage is async (`async_trait` for `StateStore`)
