# Attuned Project Overview

## Purpose
Attuned is a minimal, Rust-first framework for representing human state as interpretable vectors and translating that state into machine-consumable interaction constraints for LLM/agent systems.

**Key Identity**: This is NOT a chatbot, NOT an agent, and NOT an automation engine. It produces *context*, not actions.

## Design Goals
- **Speed & determinism**: Pure Rust, low-latency state reads/writes, predictable behavior
- **Legibility**: Every dimension is human-readable and overrideable
- **Agency by construction**: No side-effects, no hidden optimizers, no "auto-send" patterns
- **Composable**: Embed in any app or agent stack; bring your own transport, storage, and LLM

## Hard Constraints (Non-Goals)
1. **No action execution**: Never sends messages, schedules events, moves money, or calls third-party APIs
2. **No persuasion optimization**: No engagement maximization, no conversion objectives, no nudges
3. **No covert inference drift**: Self-report always overrides inference; inferred signals are bounded and explainable
4. **No UI**: Library only (optional reference server feature-gated)
5. **No content memory**: Stores state descriptors, not personal content or message history

## Tech Stack
- **Language**: Rust 1.75+ (tested with 1.92.0)
- **Build System**: Cargo workspace
- **Async Runtime**: Tokio
- **Serialization**: Serde/JSON
- **HTTP Server**: Axum 0.8
- **Storage**: In-memory (default), Qdrant (optional)
- **Observability**: tracing, metrics, OpenTelemetry

## License
Dual-licensed under MIT OR Apache-2.0

## Current Status (as of 2025-12-16)
- **Build Status**: Passing
- **Tests**: 26/26 passing
- **Core functionality**: Complete and tested
- **Remaining work**: Qdrant backend, CLI completion, CI/CD, security hardening

## Core Concepts

### Axes (24 Interpretable Dimensions)
State is represented as values in [0.0, 1.0] along named axes:

**Cognitive**: cognitive_load, decision_fatigue, tolerance_for_complexity, urgency_sensitivity

**Emotional**: emotional_openness, emotional_stability, anxiety_level, need_for_reassurance

**Social**: warmth, formality, boundary_strength, assertiveness, reciprocity_expectation

**Preferences**: ritual_need, transactional_preference, verbosity_preference, directness_preference

**Control**: autonomy_preference, suggestion_tolerance, interruption_tolerance, reflection_vs_action_bias

**Safety**: stakes_awareness, privacy_sensitivity

### State Flow
1. Collect self-report axes (sliders, CLI flags, or app state)
2. Persist latest snapshot via StateStore
3. Fetch snapshot before LLM call and translate to PromptContext
4. Inject guidelines into system prompt
5. Require explicit user approval for any outgoing communication
