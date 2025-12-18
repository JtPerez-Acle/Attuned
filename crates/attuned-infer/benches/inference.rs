//! Benchmarks for inference performance.
//!
//! Target: <100μs for linguistic extraction, <1ms for full pipeline.

use attuned_infer::{Baseline, InferenceEngine, LinguisticExtractor};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

const SHORT_TEXT: &str = "Hello, how are you?";

const MEDIUM_TEXT: &str = "I've been having some trouble with my account and would appreciate \
    your help in resolving this issue. The problem started yesterday when I tried to log in.";

const LONG_TEXT: &str = "I absolutely need your help with this critical situation! \
    I've been trying to figure out what's going wrong with my project for the past three hours, \
    and I'm completely stuck. The documentation doesn't seem to cover this specific use case, \
    and I've searched through all the forums without finding anything relevant. \
    I think maybe the problem might be related to how I configured the initial settings, \
    but I'm really not sure at this point. Could you please take a look and help me understand \
    what I'm doing wrong? This is quite urgent as I have a deadline tomorrow morning. \
    Thank you so much in advance for your assistance!";

fn bench_linguistic_extraction(c: &mut Criterion) {
    let extractor = LinguisticExtractor::default();

    let mut group = c.benchmark_group("linguistic_extraction");

    group.bench_with_input(
        BenchmarkId::new("short", "19 chars"),
        SHORT_TEXT,
        |b, text| b.iter(|| extractor.extract(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("medium", "180 chars"),
        MEDIUM_TEXT,
        |b, text| b.iter(|| extractor.extract(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("long", "800 chars"),
        LONG_TEXT,
        |b, text| b.iter(|| extractor.extract(black_box(text))),
    );

    group.finish();
}

fn bench_full_inference(c: &mut Criterion) {
    let engine = InferenceEngine::default();

    let mut group = c.benchmark_group("full_inference");

    group.bench_with_input(
        BenchmarkId::new("short", "19 chars"),
        SHORT_TEXT,
        |b, text| b.iter(|| engine.infer(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("medium", "180 chars"),
        MEDIUM_TEXT,
        |b, text| b.iter(|| engine.infer(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("long", "800 chars"),
        LONG_TEXT,
        |b, text| b.iter(|| engine.infer(black_box(text))),
    );

    group.finish();
}

fn bench_inference_with_baseline(c: &mut Criterion) {
    let engine = InferenceEngine::default();

    c.bench_function("inference_with_baseline", |b| {
        b.iter_batched(
            || {
                // Setup: create baseline with some history
                let mut baseline = Baseline::default();
                for _ in 0..20 {
                    baseline.add(&engine.extract_features(MEDIUM_TEXT));
                }
                baseline
            },
            |mut baseline| engine.infer_with_baseline(black_box(LONG_TEXT), &mut baseline, None),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_baseline_update(c: &mut Criterion) {
    let extractor = LinguisticExtractor::default();
    let features = extractor.extract(MEDIUM_TEXT);

    c.bench_function("baseline_update", |b| {
        b.iter_batched(
            || Baseline::new(50),
            |mut baseline| {
                baseline.add(black_box(&features));
                baseline
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_linguistic_extraction,
    bench_full_inference,
    bench_inference_with_baseline,
    bench_baseline_update,
);

criterion_main!(benches);
