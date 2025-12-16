# Attuned (Rust)
**Attuned** is a minimal, **Rust-first** framework for representing human state as **interpretable vectors** and translating that state into **machine-consumable interaction constraints** for LLM/agent systems.

This is **not** a chatbot, **not** an agent, and **not** an automation engine. It produces *context*, not actions.

## Design goals
- **Speed & determinism:** pure Rust, low-latency state reads/writes, predictable behavior.
- **Legibility:** every dimension is human-readable and overrideable.
- **Agency by construction:** no side-effects, no hidden optimizers, no “auto-send” patterns.
- **Composable:** embed in any app or agent stack; bring your own transport, storage, and LLM.

## Non-goals (hard constraints)
- **No action execution:** never send messages, schedule events, move money, click buttons, or call third-party APIs on the user’s behalf.
- **No persuasion optimization:** no engagement maximization, no conversion objectives, no nudges that optimize for external outcomes.
- **No covert inference drift:** self-report always overrides inference; inferred signals must be bounded and explainable.
- **No UI:** library only (optional reference server feature-gated).
- **No content memory:** Attuned stores *state descriptors*, not personal content or message history.

> See `NON_GOALS.md` in a complete repo. For this MVP doc, the constraints above are canonical.

---

## Core concepts

### 1) Axes (interpretable dimensions)
State is represented as values in `[0.0, 1.0]` along named axes (e.g., `warmth`, `formality`, `boundary_strength`).

**MVP recommendation:** start with **20–40 axes** that are **empirically interpretable** and stable under override.

Example axes (24):
- Cognitive: `cognitive_load`, `decision_fatigue`, `tolerance_for_complexity`, `urgency_sensitivity`
- Emotional: `emotional_openness`, `emotional_stability`, `anxiety_level`, `need_for_reassurance`
- Social: `warmth`, `formality`, `boundary_strength`, `assertiveness`, `reciprocity_expectation`
- Preferences: `ritual_need`, `transactional_preference`, `verbosity_preference`, `directness_preference`
- Control: `autonomy_preference`, `suggestion_tolerance`, `interruption_tolerance`, `reflection_vs_action_bias`
- Safety: `stakes_awareness`, `privacy_sensitivity`

### 2) State vector
A **StateVector** is a dense vector plus metadata. Vectors can be stored as `Vec<f32>` or `[f32; N]` for fixed-size builds.

### 3) Translator (vector → constraints)
A **Translator** converts the numeric state into a *semantic* representation, such as:
- A list of interaction guidelines
- A structured policy block
- A “system prompt” snippet (string) for LLM conditioning

MVP: **rule-based translators** for maximum transparency.

### 4) Stores (pluggable)
Attuned is storage-agnostic. Provide a trait for storing/retrieving the latest state snapshot:
- In-memory store (default)
- SQLite / Postgres store (feature)
- Qdrant store (feature) for fast state fetch + optional history

> Qdrant is used here as a **vector snapshot store**, not for semantic search.

---

## Minimal API (library)

### Types
```rust
/// Normalized axis value.
pub type AxisValue = f32;

#[derive(Clone, Debug)]
pub struct StateSnapshot {
    pub user_id: String,
    pub updated_at_unix_ms: i64,
    pub source: Source,
    pub confidence: f32,            // [0,1]
    pub axes: std::collections::BTreeMap<String, AxisValue>, // named axes
}

#[derive(Clone, Debug)]
pub enum Source {
    SelfReport,
    Inferred,
    Mixed,
}

#[derive(Clone, Debug)]
pub struct PromptContext {
    pub guidelines: Vec<String>,
    pub tone: String,               // e.g. "calm-neutral"
    pub verbosity: Verbosity,       // e.g. Low/Medium/High
    pub flags: Vec<String>,         // e.g. "high_cognitive_load"
}

#[derive(Clone, Debug)]
pub enum Verbosity { Low, Medium, High }
```

### Traits
```rust
#[async_trait::async_trait]
pub trait StateStore: Send + Sync {
    async fn upsert_latest(&self, snapshot: StateSnapshot) -> anyhow::Result<()>;
    async fn get_latest(&self, user_id: &str) -> anyhow::Result<Option<StateSnapshot>>;
}

pub trait Translator: Send + Sync {
    fn to_prompt_context(&self, snapshot: &StateSnapshot) -> PromptContext;
}
```

### Reference rule-based translator
```rust
pub struct RuleTranslator {
    pub thresholds: Thresholds,
}

pub struct Thresholds {
    pub hi: f32,
    pub lo: f32,
}

impl Translator for RuleTranslator {
    fn to_prompt_context(&self, s: &StateSnapshot) -> PromptContext {
        let get = |k: &str| *s.axes.get(k).unwrap_or(&0.5);

        let mut guidelines = vec![
            "Offer suggestions, not actions".to_string(),
            "Drafts require explicit user approval".to_string(),
            "Silence is acceptable if no action is required".to_string(),
        ];

        let cognitive_load = get("cognitive_load");
        if cognitive_load > self.thresholds.hi {
            guidelines.push("Keep responses concise; avoid multi-step plans unless requested".into());
        }

        let ritual_need = get("ritual_need");
        if ritual_need < self.thresholds.lo {
            guidelines.push("Avoid ceremonial gestures; keep interactions pragmatic".into());
        }

        let boundary_strength = get("boundary_strength");
        if boundary_strength > self.thresholds.hi {
            guidelines.push("Maintain firm boundaries; do not over-accommodate".into());
        }

        let warmth = get("warmth");
        let tone = if warmth > self.thresholds.hi { "warm-neutral" } else { "calm-neutral" }.to_string();

        let verbosity = if get("verbosity_preference") < self.thresholds.lo {
            Verbosity::Low
        } else if get("verbosity_preference") > self.thresholds.hi {
            Verbosity::High
        } else {
            Verbosity::Medium
        };

        let mut flags = vec![];
        if cognitive_load > self.thresholds.hi { flags.push("high_cognitive_load".into()); }

        PromptContext { guidelines, tone, verbosity, flags }
    }
}
```

---

## Reference wire format (JSON)
For integrations, keep the payload stable and explicit.

### Write partial state (patch semantics)
```json
{
  "user_id": "u_123",
  "source": "self_report",
  "confidence": 1.0,
  "axes": {
    "warmth": 0.6,
    "formality": 0.3,
    "boundary_strength": 0.8
  }
}
```

### Read prompt context
```json
{
  "guidelines": [
    "Offer suggestions, not actions",
    "Drafts require explicit user approval",
    "Silence is acceptable if no action is required",
    "Keep responses concise; avoid multi-step plans unless requested",
    "Maintain firm boundaries; do not over-accommodate"
  ],
  "tone": "calm-neutral",
  "verbosity": "low",
  "flags": ["high_cognitive_load"]
}
```

---

## Storage backends

### In-memory (default)
- Suitable for single-process apps and tests.
- Zero dependencies beyond async runtime.

### Qdrant (feature: `qdrant`)
Use Qdrant to store **latest** and optionally **history** snapshots.

Recommended convention:
- Latest snapshot point id: `{user_id}::latest`
- History snapshot point id: `{user_id}::{unix_ms}`

Payload:
- `user_id`, `updated_at_unix_ms`, `source`, `confidence`
- optional `schema_version` for future axis changes

**Important:** this is a *snapshot store*. Similarity search is optional and not required for MVP.

---

## Real results: evaluation without hype
Attuned is useful if it measurably improves *interaction quality* without removing agency.

### MVP metrics (practical)
- **Edit distance of drafts:** fewer user edits before sending (but never auto-send).
- **Time-to-decision:** shorter time from “nudge” to user decision (accept/reject/defer).
- **User override rate:** high overrides indicate bad axis design or poor mappings.
- **Regret checks:** quick after-action prompt: “Did this feel like *you*?” (binary + optional note).
- **Constraint violations:** count of times an integration tried to execute actions (should be zero at framework level).

### Test protocol
- Record state snapshots + produced `PromptContext` (no message content).
- Run A/B where B includes Attuned context injection.
- Measure outcomes above across 30–100 interactions.

---

## Security & privacy posture (baseline)
- Store only normalized axes + timestamps + source metadata.
- Avoid storing message content, bank data, or contact lists inside Attuned.
- If an integrator uses sensitive signals, ensure:
  - encryption at rest
  - explicit user consent
  - local-first option

Attuned is designed to be compatible with **local-only** deployments.

---

## Repository layout (suggested)
```
attuned/
├─ crates/
│  ├─ attuned-core/        # types, axis schema, translator traits
│  ├─ attuned-store/       # StateStore trait + in-memory store
│  ├─ attuned-qdrant/      # Qdrant backend (feature)
│  ├─ attuned-http/        # optional reference server (feature)
│  └─ attuned-cli/         # optional dev tool (feature)
├─ NON_GOALS.md
├─ README.md
└─ LICENSE
```

---

## License
Recommend **Apache-2.0** or **MIT** for maximal reuse.

---

## Quick start (conceptual)
1) Collect self-report axes (sliders, CLI flags, or app state).  
2) Persist latest snapshot via a `StateStore`.  
3) Fetch snapshot before LLM call and translate to `PromptContext`.  
4) Inject `guidelines` into your system prompt.  
5) Require explicit user approval for any outgoing communication.

That’s the whole framework.

