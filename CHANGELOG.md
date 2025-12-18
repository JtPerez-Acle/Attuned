# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2025-12-18

### Highlights

**Attuned is ready for production.** This release delivers a complete framework for representing human state and translating it into LLM-consumable context.

- **23 axes** across 6 categories (Cognitive, Emotional, Social, Preferences, Control, Safety)
- **Python bindings** with simple `Attuned` class - works with any LLM
- **Validated behavior** - tested effect sizes up to d=7.4 for verbosity control
- **Inference engine** - detect user state from text (<35μs latency)

### Added

#### Core Framework (Rust)
- **attuned-core**: Core types and traits
  - `StateSnapshot` with builder pattern and validation
  - `PromptContext` with guidelines, tone, verbosity, flags
  - `RuleTranslator` with explicit guidelines for all axes
  - 23 canonical axes with governance metadata (`intent`, `forbidden_uses`)
  - Telemetry module with health checks and audit events
- **attuned-store**: Storage abstraction
  - `StateStore` async trait
  - `MemoryStore` thread-safe in-memory implementation
- **attuned-http**: HTTP reference server
  - REST API for state management (CRUD)
  - Context translation endpoint
  - Authentication and rate limiting
- **attuned-infer**: NLP inference engine
  - Linguistic feature extraction
  - Bayesian state estimation
  - Baseline deviation detection
  - <35μs inference latency
- **attuned-cli**: CLI tool with `serve` command

#### Python Bindings
- **Simple API**: `Attuned` class with all 23 axes as keyword arguments
- **Presets**: 7 pre-configured states (anxious_user, busy_executive, etc.)
- **Integrations**: OpenAI, Anthropic, LiteLLM (100+ providers)
- **Type stubs**: Full `.pyi` files for IDE autocomplete
- **Cross-platform wheels**: Linux, macOS, Windows (Python 3.9-3.12)

#### Interactive Demo
- Split-screen comparison (vanilla LLM vs Attuned)
- Real-time axis detection from user messages
- Manual axis sliders for experimentation
- Copy-paste integration code generator

### Validated Results

| Scenario | Without Attuned | With Attuned | Reduction |
|----------|-----------------|--------------|-----------|
| Anxious user | 395 tokens | 64 tokens | 84% |
| Overwhelmed | ~400 tokens | ~50 tokens | 87% |

Effect sizes (Cohen's d):
- Verbosity control: d=7.40 (massive)
- Warmth/tone: d=4.11 (very large)
- Cognitive load adaptation: d=1.90 (large)

### Philosophy

Attuned follows hard constraints from [MANIFESTO.md](MANIFESTO.md):
- **Never execute actions** - provides context, not control
- **Never optimize for persuasion** - no engagement metrics
- **Never covertly infer** - self-report overrides inference
- **Never store content** - state descriptors only

### Architecture

```
attuned-cli → attuned-http → attuned-store → attuned-core
                   ↓          attuned-qdrant ↗
              attuned-infer ──────────────────↗
```

### Not Yet Implemented
- Qdrant backend (stubs only)
- Full CLI functionality beyond `serve`
- OpenTelemetry export

---

## Release Checklist

- [x] All tests passing
- [x] Clippy warnings resolved
- [x] Documentation complete
- [x] CHANGELOG updated
- [x] Version numbers bumped (1.0.0)
- [ ] Git tag created
- [ ] Crates published to crates.io
- [ ] Python wheels published to PyPI
