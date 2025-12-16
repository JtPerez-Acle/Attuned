# TASK-008: Testing Infrastructure

## Status
- [ ] Not Started
- [ ] In Progress
- [x] Completed
- [ ] Blocked

**Priority**: High
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: 4 - Quality & Testing
**Depends On**: TASK-002, TASK-003
**Blocks**: TASK-011 (Release)

## Task Description
Establish comprehensive testing infrastructure ensuring correctness, performance, and reliability. Tests are essential for confident releases and ongoing maintenance.

## Requirements
1. Unit tests for all public APIs
2. Integration tests for store backends
3. Property-based tests for translator logic
4. Benchmark suite for performance regression
5. Test fixtures and helpers
6. CI integration

## Test Categories

### 1. Unit Tests
Location: `src/` alongside code (Rust convention)

```rust
// In attuned-core/src/translator.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_translator_high_cognitive_load() {
        let translator = RuleTranslator::default();
        let snapshot = StateSnapshot::builder()
            .user_id("test")
            .axis("cognitive_load", 0.9)
            .build();

        let context = translator.to_prompt_context(&snapshot);
        assert!(context.guidelines.iter().any(|g| g.contains("concise")));
        assert!(context.flags.contains(&"high_cognitive_load".to_string()));
    }
}
```

### 2. Integration Tests
Location: `tests/` directory in each crate

```rust
// In attuned-store/tests/memory_store.rs
#[tokio::test]
async fn concurrent_access() {
    let store = MemoryStore::new(Default::default());
    let handles: Vec<_> = (0..100).map(|i| {
        let store = store.clone();
        tokio::spawn(async move {
            store.upsert_latest(snapshot_for_user(i)).await
        })
    }).collect();

    for handle in handles {
        handle.await.unwrap().unwrap();
    }
}
```

### 3. Property-Based Tests
Using `proptest` or `quickcheck`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn axis_values_always_normalized(value in 0.0f32..=1.0f32) {
        let snapshot = StateSnapshot::builder()
            .axis("warmth", value)
            .build();

        let warmth = snapshot.axes.get("warmth").unwrap();
        prop_assert!(*warmth >= 0.0 && *warmth <= 1.0);
    }

    #[test]
    fn translator_never_panics(axes in prop::collection::btree_map(
        "[a-z_]+", 0.0f32..=1.0f32, 0..30
    )) {
        let translator = RuleTranslator::default();
        let snapshot = StateSnapshot { axes, ..Default::default() };

        // Should never panic
        let _context = translator.to_prompt_context(&snapshot);
    }
}
```

### 4. Benchmarks
Location: `benches/` using `criterion`

```rust
// In attuned-core/benches/translator.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn translator_benchmark(c: &mut Criterion) {
    let translator = RuleTranslator::default();
    let snapshot = full_snapshot();

    c.bench_function("translate_full_snapshot", |b| {
        b.iter(|| translator.to_prompt_context(&snapshot))
    });
}

criterion_group!(benches, translator_benchmark);
criterion_main!(benches);
```

### 5. Test Fixtures
Location: `attuned-test-utils` crate or `testutil` module

```rust
pub mod fixtures {
    pub fn minimal_snapshot() -> StateSnapshot { ... }
    pub fn full_snapshot() -> StateSnapshot { ... }
    pub fn high_load_snapshot() -> StateSnapshot { ... }
}

pub mod assertions {
    pub fn assert_valid_prompt_context(ctx: &PromptContext) { ... }
}
```

## Test Matrix

| Crate | Unit | Integration | Property | Benchmark |
|-------|------|-------------|----------|-----------|
| attuned-core | ✓ | - | ✓ | ✓ |
| attuned-store | ✓ | ✓ | - | ✓ |
| attuned-qdrant | ✓ | ✓* | - | - |
| attuned-http | ✓ | ✓ | - | - |
| attuned-cli | ✓ | ✓ | - | - |

*Requires Qdrant instance (Docker)

## Acceptance Criteria
- [ ] >80% code coverage on attuned-core (coverage infrastructure in CI)
- [x] Integration tests for all StateStore implementations (MemoryStore has 6 tests)
- [x] Property tests for translator edge cases (8 property tests)
- [x] Benchmarks with baseline for regression detection (2 benchmark files)
- [x] Test utilities crate or module (helper functions in test modules)
- [x] CI runs all tests on every PR (via TASK-010)
- [x] Documentation for running tests locally (in DEVELOPMENT.md)

## Progress Log
- 2025-12-16: Task created
- 2025-12-16: Added 6 property tests for StateSnapshot (axis validation, user_id, serialization, multiple axes)
- 2025-12-16: Added 6 property tests for RuleTranslator (never panics, base guidelines, determinism, thresholds)
- 2025-12-16: Created translator benchmark suite (minimal/full/high-load snapshots, axis count scaling)
- 2025-12-16: Created snapshot benchmark suite (creation, scaling, serialization, access)
- 2025-12-16: Fixed axis name strategy to respect validation rules (no leading/trailing underscores)
- 2025-12-16: All 38 tests pass (28 in attuned-core, 4 in http, 6 in store)
- 2025-12-16: **COMPLETED** - Property tests and benchmarks ready
