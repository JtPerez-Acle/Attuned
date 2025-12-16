# TASK-013: Governance & Schema Formalization

## Status
- [ ] Not Started
- [ ] In Progress
- [x] Completed
- [ ] Blocked

**Priority**: High
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: 4 - Quality & Testing
**Depends On**: TASK-012 (Security)
**Blocks**: TASK-011 (Release)

## Task Description

Formalize Attuned's governance model and axis schema to prevent:
1. Axis semantic drift across ecosystem
2. Translator power creep (covert agency)
3. Misuse by downstream agents

This task implements recommendations from external review to ensure Attuned remains a **context lens, not a manipulation engine**.

## Requirements

### A. Manifesto Document (`MANIFESTO.md`)

A 1-2 page philosophical document defining:
1. What Attuned is (core identity)
2. What it refuses to become (hard constraints)
3. Failure modes we actively defend against
4. Governance principles for contributors

**Not marketing copy** - a technical/philosophical statement that:
- Attracts aligned users/contributors
- Repels those seeking manipulation tools
- Provides decision framework for future changes

### B. Formalized Axis Schema

Transform axes from constants to rich schema:

```rust
pub struct AxisDefinition {
    /// Canonical name (immutable after v1.0)
    pub name: &'static str,

    /// Human-readable description
    pub description: &'static str,

    /// What 0.0 means (low anchor)
    pub low_anchor: &'static str,

    /// What 1.0 means (high anchor)
    pub high_anchor: &'static str,

    /// Semantic category
    pub category: AxisCategory,

    /// Intended use cases
    pub intent: &'static [&'static str],

    /// Explicitly forbidden interpretations/uses
    pub forbidden_uses: &'static [&'static str],

    /// Version when introduced
    pub since: &'static str,

    /// Deprecation info (if any)
    pub deprecated: Option<DeprecationInfo>,
}

pub enum AxisCategory {
    Cognitive,
    Emotional,
    Social,
    Preferences,
    Control,
    Safety,
}

pub struct DeprecationInfo {
    pub since: &'static str,
    pub reason: &'static str,
    pub replacement: Option<&'static str>,
}
```

### C. Translator Governance

Document rules for what the Translator is allowed to do:

**Allowed:**
- Map axis values to tone descriptors
- Generate behavioral guidelines
- Set verbosity levels
- Add flags for edge conditions

**Forbidden:**
- Infer hidden axes from other axes
- Optimize for engagement/conversion
- Override user self-report
- Add "adaptive" heuristics that learn

### D. Documentation Updates

1. **SECURITY.md**: Add misuse warnings section
2. **README.md**: Link to manifesto
3. **Inline docs**: Add governance notes to Translator

## Deliverables

| File | Description |
|------|-------------|
| `MANIFESTO.md` | Core philosophy document |
| `crates/attuned-core/src/axes.rs` | Formalized AxisDefinition schema |
| `crates/attuned-core/src/translator.rs` | Governance comments |
| `SECURITY.md` | Misuse warnings section |
| `README.md` | Link to manifesto |

## Axis Definitions to Formalize

All 23 axes need full definitions:

### Cognitive (4)
- `cognitive_load`
- `decision_fatigue`
- `tolerance_for_complexity`
- `urgency_sensitivity`

### Emotional (4)
- `emotional_openness`
- `emotional_stability`
- `anxiety_level`
- `need_for_reassurance`

### Social (5)
- `warmth`
- `formality`
- `boundary_strength`
- `assertiveness`
- `reciprocity_expectation`

### Preferences (4)
- `ritual_need`
- `transactional_preference`
- `verbosity_preference`
- `directness_preference`

### Control (4)
- `autonomy_preference`
- `suggestion_tolerance`
- `interruption_tolerance`
- `reflection_vs_action_bias`

### Safety (2)
- `stakes_awareness`
- `privacy_sensitivity`

## Example Axis Definition

```rust
pub const COGNITIVE_LOAD: AxisDefinition = AxisDefinition {
    name: "cognitive_load",
    description: "Current mental bandwidth consumption and available processing capacity",
    low_anchor: "Relaxed and clear-headed; high capacity for complexity and nuance",
    high_anchor: "Overwhelmed or distracted; needs simplification and focus",
    category: AxisCategory::Cognitive,
    intent: &[
        "Adjust response complexity and depth",
        "Gate multi-step suggestions and plans",
        "Determine appropriate information density",
    ],
    forbidden_uses: &[
        "Infer user intelligence or cognitive capability",
        "Target users with high load for simplified 'fast' conversions",
        "Bypass user autonomy when cognitive load is elevated",
        "Use as justification for withholding information user requested",
    ],
    since: "0.1.0",
    deprecated: None,
};
```

## Acceptance Criteria

- [x] MANIFESTO.md created and reviewed
- [x] AxisDefinition struct implemented
- [x] All 23 axes have full definitions with forbidden_uses
- [x] Translator has governance comments
- [x] SECURITY.md has misuse warnings
- [x] README.md links to manifesto
- [x] All tests passing
- [x] Documentation builds without warnings

## Future Considerations (Post-v1.0)

1. **Axis Versioning**: Semantic versioning for axis schema
2. **Deprecation Tooling**: Warnings when using deprecated axes
3. **Compatibility Checker**: Tool to verify axis usage against schema
4. **Extension Registry**: Community axes with clear governance

## Progress Log

- 2025-12-16: Task created based on external review recommendations
- 2025-12-16: Task completed
  - Created MANIFESTO.md with philosophy, refusals, failure modes, and governance
  - Implemented AxisDefinition with intent, forbidden_uses, since, deprecated fields
  - Implemented AxisCategory enum with Display trait
  - All 23 axes defined with 3+ intents and 3+ forbidden uses each
  - Added translator governance comments to translator.rs
  - Added misuse warnings section to SECURITY.md
  - Updated README.md with manifesto badge and contributing guidance
  - All 52 tests passing
