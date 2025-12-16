//! Benchmarks for the RuleTranslator.

use attuned_core::{RuleTranslator, Source, StateSnapshot, Translator};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn create_minimal_snapshot() -> StateSnapshot {
    StateSnapshot::builder()
        .user_id("bench_user")
        .source(Source::SelfReport)
        .build()
        .unwrap()
}

fn create_full_snapshot() -> StateSnapshot {
    StateSnapshot::builder()
        .user_id("bench_user")
        .source(Source::SelfReport)
        .confidence(1.0)
        // Cognitive axes
        .axis("cognitive_load", 0.7)
        .axis("decision_fatigue", 0.5)
        .axis("tolerance_for_complexity", 0.6)
        .axis("urgency_sensitivity", 0.4)
        // Emotional axes
        .axis("emotional_openness", 0.6)
        .axis("emotional_stability", 0.7)
        .axis("anxiety_level", 0.3)
        .axis("need_for_reassurance", 0.4)
        // Social axes
        .axis("warmth", 0.8)
        .axis("formality", 0.3)
        .axis("boundary_strength", 0.6)
        .axis("assertiveness", 0.5)
        .axis("reciprocity_expectation", 0.5)
        // Preference axes
        .axis("ritual_need", 0.2)
        .axis("transactional_preference", 0.6)
        .axis("verbosity_preference", 0.5)
        .axis("directness_preference", 0.7)
        // Control axes
        .axis("autonomy_preference", 0.8)
        .axis("suggestion_tolerance", 0.6)
        .axis("interruption_tolerance", 0.4)
        .axis("reflection_vs_action_bias", 0.5)
        // Safety axes
        .axis("stakes_awareness", 0.6)
        .axis("privacy_sensitivity", 0.7)
        .build()
        .unwrap()
}

fn create_high_load_snapshot() -> StateSnapshot {
    StateSnapshot::builder()
        .user_id("bench_user")
        .source(Source::SelfReport)
        .axis("cognitive_load", 0.95)
        .axis("decision_fatigue", 0.9)
        .axis("anxiety_level", 0.8)
        .axis("boundary_strength", 0.9)
        .build()
        .unwrap()
}

fn translator_benchmarks(c: &mut Criterion) {
    let translator = RuleTranslator::default();

    let minimal = create_minimal_snapshot();
    let full = create_full_snapshot();
    let high_load = create_high_load_snapshot();

    let mut group = c.benchmark_group("translator");

    group.bench_function("minimal_snapshot", |b| {
        b.iter(|| translator.to_prompt_context(black_box(&minimal)))
    });

    group.bench_function("full_snapshot", |b| {
        b.iter(|| translator.to_prompt_context(black_box(&full)))
    });

    group.bench_function("high_load_snapshot", |b| {
        b.iter(|| translator.to_prompt_context(black_box(&high_load)))
    });

    group.finish();
}

fn translator_scaling(c: &mut Criterion) {
    let translator = RuleTranslator::default();

    let mut group = c.benchmark_group("translator_scaling");

    for axis_count in [1, 5, 10, 15, 20, 23] {
        let snapshot = {
            let mut builder = StateSnapshot::builder()
                .user_id("bench_user")
                .source(Source::SelfReport);

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

            for axis in axes.iter().take(axis_count) {
                builder = builder.axis(*axis, 0.5);
            }

            builder.build().unwrap()
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(axis_count),
            &snapshot,
            |b, snapshot| b.iter(|| translator.to_prompt_context(black_box(snapshot))),
        );
    }

    group.finish();
}

criterion_group!(benches, translator_benchmarks, translator_scaling);
criterion_main!(benches);
