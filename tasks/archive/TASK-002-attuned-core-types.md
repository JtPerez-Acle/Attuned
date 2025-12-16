# TASK-002: attuned-core - Types & Traits

## Status
- [ ] Not Started
- [ ] In Progress
- [x] Completed
- [ ] Blocked

**Priority**: High
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: 2 - Core Implementation
**Depends On**: TASK-001
**Blocks**: TASK-003, TASK-004, TASK-005, TASK-006

## Task Description
Implement the core types and traits defined in AGENTS.md. This crate is the heart of Attuned - pure types with no side effects.

## Requirements
1. Implement all core types: `AxisValue`, `StateSnapshot`, `Source`, `PromptContext`, `Verbosity`
2. Define the `Translator` trait
3. Implement `RuleTranslator` with configurable thresholds
4. Define the canonical axis schema (24 axes from AGENTS.md)
5. Add validation for axis values (must be in [0.0, 1.0])
6. Implement `serde` serialization/deserialization
7. Add comprehensive documentation

## Core Types (from AGENTS.md)

```rust
pub type AxisValue = f32;  // Always in [0.0, 1.0]

pub struct StateSnapshot {
    pub user_id: String,
    pub updated_at_unix_ms: i64,
    pub source: Source,
    pub confidence: f32,
    pub axes: BTreeMap<String, AxisValue>,
}

pub enum Source { SelfReport, Inferred, Mixed }

pub struct PromptContext {
    pub guidelines: Vec<String>,
    pub tone: String,
    pub verbosity: Verbosity,
    pub flags: Vec<String>,
}

pub enum Verbosity { Low, Medium, High }
```

## Canonical Axes (24)
- **Cognitive**: `cognitive_load`, `decision_fatigue`, `tolerance_for_complexity`, `urgency_sensitivity`
- **Emotional**: `emotional_openness`, `emotional_stability`, `anxiety_level`, `need_for_reassurance`
- **Social**: `warmth`, `formality`, `boundary_strength`, `assertiveness`, `reciprocity_expectation`
- **Preferences**: `ritual_need`, `transactional_preference`, `verbosity_preference`, `directness_preference`
- **Control**: `autonomy_preference`, `suggestion_tolerance`, `interruption_tolerance`, `reflection_vs_action_bias`
- **Safety**: `stakes_awareness`, `privacy_sensitivity`

## Implementation Notes
- Use `#[derive(Clone, Debug, Serialize, Deserialize)]` consistently
- Consider `#[non_exhaustive]` for enums to allow future additions
- Validate axis values at construction time
- `StateSnapshot::new()` should take a builder or validated inputs
- RuleTranslator thresholds: `hi = 0.7`, `lo = 0.3` as defaults

## Acceptance Criteria
- [ ] All types implemented with proper derives
- [ ] `Translator` trait defined
- [ ] `RuleTranslator` implemented per AGENTS.md spec
- [ ] Axis schema defined as constants
- [ ] Validation prevents out-of-range axis values
- [ ] JSON serialization matches wire format in AGENTS.md
- [ ] Unit tests for RuleTranslator edge cases
- [ ] Documentation for all public items

## Progress Log
- 2025-12-16: Task created
- 2025-12-16: Implemented all core types (StateSnapshot, Source, PromptContext, Verbosity)
- 2025-12-16: Created StateSnapshotBuilder with validation
- 2025-12-16: Implemented RuleTranslator with configurable thresholds
- 2025-12-16: Defined 23 canonical axes (spec said 24, actual count is 23)
- 2025-12-16: Added comprehensive unit tests (16 tests pass)
- 2025-12-16: **COMPLETED** - All types working with full serde support
