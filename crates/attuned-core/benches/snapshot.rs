//! Benchmarks for StateSnapshot operations.

use attuned_core::{Source, StateSnapshot};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn snapshot_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_creation");

    group.bench_function("minimal", |b| {
        b.iter(|| {
            StateSnapshot::builder()
                .user_id(black_box("bench_user"))
                .build()
                .unwrap()
        })
    });

    group.bench_function("with_source_and_confidence", |b| {
        b.iter(|| {
            StateSnapshot::builder()
                .user_id(black_box("bench_user"))
                .source(Source::SelfReport)
                .confidence(1.0)
                .build()
                .unwrap()
        })
    });

    group.bench_function("single_axis", |b| {
        b.iter(|| {
            StateSnapshot::builder()
                .user_id(black_box("bench_user"))
                .axis("warmth", 0.7)
                .build()
                .unwrap()
        })
    });

    group.bench_function("five_axes", |b| {
        b.iter(|| {
            StateSnapshot::builder()
                .user_id(black_box("bench_user"))
                .axis("warmth", 0.7)
                .axis("formality", 0.3)
                .axis("cognitive_load", 0.5)
                .axis("boundary_strength", 0.6)
                .axis("verbosity_preference", 0.5)
                .build()
                .unwrap()
        })
    });

    group.finish();
}

fn snapshot_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_axis_scaling");

    let axes = [
        "cognitive_load",
        "decision_fatigue",
        "tolerance_for_complexity",
        "urgency_sensitivity",
        "emotional_openness",
        "emotional_stability",
        "anxiety_level",
        "need_for_reassurance",
        "warmth",
        "formality",
        "boundary_strength",
        "assertiveness",
        "reciprocity_expectation",
        "ritual_need",
        "transactional_preference",
        "verbosity_preference",
        "directness_preference",
        "autonomy_preference",
        "suggestion_tolerance",
        "interruption_tolerance",
        "reflection_vs_action_bias",
        "stakes_awareness",
        "privacy_sensitivity",
    ];

    for axis_count in [1, 5, 10, 15, 20, 23] {
        group.bench_with_input(
            BenchmarkId::from_parameter(axis_count),
            &axis_count,
            |b, &count| {
                b.iter(|| {
                    let mut builder = StateSnapshot::builder().user_id("bench_user");

                    for axis in axes.iter().take(count) {
                        builder = builder.axis(*axis, 0.5);
                    }

                    builder.build().unwrap()
                })
            },
        );
    }

    group.finish();
}

fn snapshot_serialization(c: &mut Criterion) {
    let full_snapshot = StateSnapshot::builder()
        .user_id("bench_user")
        .source(Source::SelfReport)
        .confidence(1.0)
        .axis("cognitive_load", 0.7)
        .axis("warmth", 0.8)
        .axis("formality", 0.3)
        .axis("boundary_strength", 0.6)
        .axis("verbosity_preference", 0.5)
        .build()
        .unwrap();

    let json = serde_json::to_string(&full_snapshot).unwrap();

    let mut group = c.benchmark_group("snapshot_serialization");

    group.bench_function("serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&full_snapshot)).unwrap())
    });

    group.bench_function("deserialize", |b| {
        b.iter(|| serde_json::from_str::<StateSnapshot>(black_box(&json)).unwrap())
    });

    group.bench_function("roundtrip", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(black_box(&full_snapshot)).unwrap();
            serde_json::from_str::<StateSnapshot>(&serialized).unwrap()
        })
    });

    group.finish();
}

fn snapshot_access(c: &mut Criterion) {
    let snapshot = StateSnapshot::builder()
        .user_id("bench_user")
        .axis("warmth", 0.7)
        .axis("formality", 0.3)
        .axis("cognitive_load", 0.5)
        .build()
        .unwrap();

    let mut group = c.benchmark_group("snapshot_access");

    group.bench_function("get_existing_axis", |b| {
        b.iter(|| snapshot.get_axis(black_box("warmth")))
    });

    group.bench_function("get_missing_axis", |b| {
        b.iter(|| snapshot.get_axis(black_box("unknown_axis")))
    });

    group.finish();
}

criterion_group!(
    benches,
    snapshot_creation,
    snapshot_scaling,
    snapshot_serialization,
    snapshot_access
);
criterion_main!(benches);
