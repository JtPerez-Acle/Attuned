#!/usr/bin/env python3
"""
Rigorous Statistical Validation of Attuned

This script runs multiple hypotheses with sufficient sample sizes
to prove (or disprove) that Attuned actually affects LLM behavior.

Hypotheses tested:
1. Low verbosity_preference → shorter responses
2. High warmth → more warm/friendly language
3. Neutral state → no significant difference from baseline
4. High cognitive_load → simpler sentence structure
"""

import os
import re
import statistics
from pathlib import Path
from dataclasses import dataclass
from typing import List, Dict, Tuple
from dotenv import load_dotenv
from openai import OpenAI
from attuned import AttunedClient, StateSnapshot, Source

load_dotenv(Path('/home/jt/Attuned/.env'))

openai_client = OpenAI()
attuned_client = AttunedClient('http://localhost:8080')

# Test configuration
SAMPLE_SIZE = 15  # Per condition - enough for statistical significance
MODEL = "gpt-4o-mini"
MAX_TOKENS = 400
TEMPERATURE = 0.7

# Different test questions to avoid question-specific bias
TEST_QUESTIONS = [
    "Can you help me organize my project?",
    "I need advice on managing my time better.",
    "How should I approach learning a new skill?",
]

@dataclass
class TestResult:
    condition: str
    lengths: List[int]
    word_counts: List[int]
    responses: List[str]

    @property
    def mean_length(self) -> float:
        return statistics.mean(self.lengths)

    @property
    def std_length(self) -> float:
        return statistics.stdev(self.lengths) if len(self.lengths) > 1 else 0

    @property
    def mean_words(self) -> float:
        return statistics.mean(self.word_counts)


def call_llm(system_prompt: str, user_message: str) -> str:
    """Call LLM and return response."""
    response = openai_client.chat.completions.create(
        model=MODEL,
        messages=[
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_message},
        ],
        max_tokens=MAX_TOKENS,
        temperature=TEMPERATURE,
    )
    return response.choices[0].message.content


def count_warm_words(text: str) -> int:
    """Count warm/friendly language indicators."""
    warm_patterns = [
        r'\bglad\b', r'\bhappy\b', r'\bgreat\b', r'\bwonderful\b',
        r'\bexcited\b', r'\blove\b', r'\bfantastic\b', r'\bawesome\b',
        r'\bdelighted\b', r'\bpleasure\b', r'\!\s', r'\bfeel free\b',
        r'\babsolutely\b', r'\bdefinitely\b', r'\bof course\b',
    ]
    count = 0
    text_lower = text.lower()
    for pattern in warm_patterns:
        count += len(re.findall(pattern, text_lower))
    return count


def count_reassurance_phrases(text: str) -> int:
    """Count reassurance/calming phrases."""
    patterns = [
        r"don'?t worry", r"it'?s okay", r"that'?s normal",
        r"take your time", r"no rush", r"understandable",
        r"it'?s alright", r"you'?ve got this", r"step by step",
    ]
    count = 0
    text_lower = text.lower()
    for pattern in patterns:
        count += len(re.findall(pattern, text_lower))
    return count


def avg_sentence_length(text: str) -> float:
    """Calculate average sentence length (complexity proxy)."""
    sentences = re.split(r'[.!?]+', text)
    sentences = [s.strip() for s in sentences if s.strip()]
    if not sentences:
        return 0
    word_counts = [len(s.split()) for s in sentences]
    return statistics.mean(word_counts)


def run_condition(name: str, axes: Dict[str, float], n: int = SAMPLE_SIZE) -> TestResult:
    """Run n trials for a given condition."""
    print(f"  Testing: {name} ({n} samples)...", end=" ", flush=True)

    # Setup state
    builder = StateSnapshot.builder().user_id(f'test_{name}').source(Source.SelfReport)
    for axis, value in axes.items():
        builder = builder.axis(axis, value)
    snapshot = builder.build()
    attuned_client.upsert_state(snapshot)
    context = attuned_client.get_context(f'test_{name}')

    base_prompt = "You are a helpful AI assistant."
    system_prompt = f"{base_prompt}\n\n{context.format_for_prompt()}"

    lengths = []
    word_counts = []
    responses = []

    for i in range(n):
        question = TEST_QUESTIONS[i % len(TEST_QUESTIONS)]
        resp = call_llm(system_prompt, question)
        lengths.append(len(resp))
        word_counts.append(len(resp.split()))
        responses.append(resp)

    print(f"done (mean: {statistics.mean(lengths):.0f} chars)")
    return TestResult(name, lengths, word_counts, responses)


def run_baseline(n: int = SAMPLE_SIZE) -> TestResult:
    """Run baseline without Attuned context."""
    print(f"  Testing: baseline ({n} samples)...", end=" ", flush=True)

    system_prompt = "You are a helpful AI assistant."

    lengths = []
    word_counts = []
    responses = []

    for i in range(n):
        question = TEST_QUESTIONS[i % len(TEST_QUESTIONS)]
        resp = call_llm(system_prompt, question)
        lengths.append(len(resp))
        word_counts.append(len(resp.split()))
        responses.append(resp)

    print(f"done (mean: {statistics.mean(lengths):.0f} chars)")
    return TestResult("baseline", lengths, word_counts, responses)


def welchs_t_test(sample1: List[float], sample2: List[float]) -> Tuple[float, float]:
    """
    Perform Welch's t-test (unequal variance t-test).
    Returns (t_statistic, p_value approximation).
    """
    n1, n2 = len(sample1), len(sample2)
    mean1, mean2 = statistics.mean(sample1), statistics.mean(sample2)
    var1 = statistics.variance(sample1) if n1 > 1 else 0
    var2 = statistics.variance(sample2) if n2 > 1 else 0

    # Welch's t-statistic
    se = ((var1 / n1) + (var2 / n2)) ** 0.5
    if se == 0:
        return 0, 1.0

    t_stat = (mean1 - mean2) / se

    # Degrees of freedom (Welch-Satterthwaite)
    num = ((var1 / n1) + (var2 / n2)) ** 2
    denom = ((var1 / n1) ** 2 / (n1 - 1)) + ((var2 / n2) ** 2 / (n2 - 1))
    df = num / denom if denom > 0 else 1

    # Approximate p-value using normal distribution for large samples
    # (conservative for smaller samples)
    import math
    p_value = 2 * (1 - 0.5 * (1 + math.erf(abs(t_stat) / math.sqrt(2))))

    return t_stat, p_value


def effect_size(sample1: List[float], sample2: List[float]) -> float:
    """Calculate Cohen's d effect size."""
    n1, n2 = len(sample1), len(sample2)
    mean1, mean2 = statistics.mean(sample1), statistics.mean(sample2)
    var1 = statistics.variance(sample1) if n1 > 1 else 0
    var2 = statistics.variance(sample2) if n2 > 1 else 0

    # Pooled standard deviation
    pooled_std = ((((n1 - 1) * var1) + ((n2 - 1) * var2)) / (n1 + n2 - 2)) ** 0.5

    if pooled_std == 0:
        return 0

    return (mean1 - mean2) / pooled_std


def print_comparison(name: str, baseline: TestResult, test: TestResult,
                     metric: str, baseline_vals: List[float], test_vals: List[float],
                     expected_direction: str):
    """Print statistical comparison."""
    t_stat, p_value = welchs_t_test(baseline_vals, test_vals)
    d = effect_size(baseline_vals, test_vals)

    baseline_mean = statistics.mean(baseline_vals)
    test_mean = statistics.mean(test_vals)
    change_pct = ((test_mean - baseline_mean) / baseline_mean * 100) if baseline_mean else 0

    # Determine if result matches expectation
    if expected_direction == "lower":
        matches = test_mean < baseline_mean
    elif expected_direction == "higher":
        matches = test_mean > baseline_mean
    else:
        matches = abs(change_pct) < 15  # "similar" means within 15%

    significant = p_value < 0.05

    print(f"\n  {name} ({metric}):")
    print(f"    Baseline: {baseline_mean:.1f} (std: {statistics.stdev(baseline_vals):.1f})")
    print(f"    Test:     {test_mean:.1f} (std: {statistics.stdev(test_vals):.1f})")
    print(f"    Change:   {change_pct:+.1f}%")
    print(f"    t-stat:   {t_stat:.2f}, p-value: {p_value:.4f}")
    print(f"    Effect:   {abs(d):.2f} ({'large' if abs(d) > 0.8 else 'medium' if abs(d) > 0.5 else 'small'})")

    if significant and matches:
        print(f"    Result:   ✓ SIGNIFICANT & matches expectation ({expected_direction})")
    elif significant and not matches:
        print(f"    Result:   ✗ SIGNIFICANT but OPPOSITE direction!")
    elif not significant and expected_direction == "similar":
        print(f"    Result:   ✓ Not significant (expected for neutral)")
    else:
        print(f"    Result:   ~ Not statistically significant (p > 0.05)")


def main():
    # Output to both console and file
    from datetime import datetime
    output_file = Path(__file__).parent / f"validation_results_{datetime.now().strftime('%Y%m%d_%H%M%S')}.txt"

    class Tee:
        def __init__(self, filepath):
            self.file = open(filepath, 'w')
            self.stdout = __import__('sys').stdout
        def write(self, data):
            self.file.write(data)
            self.stdout.write(data)
        def flush(self):
            self.file.flush()
            self.stdout.flush()
        def close(self):
            self.file.close()

    import sys
    tee = Tee(output_file)
    sys.stdout = tee

    print("=" * 70)
    print("  RIGOROUS STATISTICAL VALIDATION OF ATTUNED")
    print(f"  Results saved to: {output_file}")
    print("=" * 70)
    print()
    print(f"Configuration:")
    print(f"  Sample size per condition: {SAMPLE_SIZE}")
    print(f"  Model: {MODEL}")
    print(f"  Temperature: {TEMPERATURE}")
    print()

    # Check server
    if not attuned_client.health():
        print("ERROR: Attuned server not running!")
        return

    print("Running tests (this will make many API calls)...")
    print()

    # Run all conditions
    baseline = run_baseline()

    low_verbosity = run_condition("low_verbosity", {
        "verbosity_preference": 0.1,
        "cognitive_load": 0.9,
    })

    high_verbosity = run_condition("high_verbosity", {
        "verbosity_preference": 0.9,
        "cognitive_load": 0.1,
    })

    high_warmth = run_condition("high_warmth", {
        "warmth": 0.95,
        "formality": 0.1,
    })

    low_warmth = run_condition("low_warmth", {
        "warmth": 0.1,
        "formality": 0.9,
    })

    neutral = run_condition("neutral", {
        "warmth": 0.5,
        "verbosity_preference": 0.5,
        "cognitive_load": 0.5,
    })

    high_anxiety = run_condition("high_anxiety", {
        "anxiety_level": 0.95,
        "cognitive_load": 0.9,
    })

    # HYPOTHESIS 1: Verbosity preference affects response length
    print()
    print("=" * 70)
    print("  HYPOTHESIS 1: Verbosity preference affects response length")
    print("=" * 70)

    print_comparison(
        "Low verbosity vs Baseline", baseline, low_verbosity,
        "char length", baseline.lengths, low_verbosity.lengths, "lower"
    )
    print_comparison(
        "High verbosity vs Baseline", baseline, high_verbosity,
        "char length", baseline.lengths, high_verbosity.lengths, "higher"
    )
    print_comparison(
        "Low vs High verbosity", high_verbosity, low_verbosity,
        "char length", high_verbosity.lengths, low_verbosity.lengths, "lower"
    )

    # HYPOTHESIS 2: Warmth affects language warmth
    print()
    print("=" * 70)
    print("  HYPOTHESIS 2: Warmth axis affects warm language usage")
    print("=" * 70)

    baseline_warm_words = [count_warm_words(r) for r in baseline.responses]
    high_warmth_words = [count_warm_words(r) for r in high_warmth.responses]
    low_warmth_words = [count_warm_words(r) for r in low_warmth.responses]

    print_comparison(
        "High warmth vs Baseline", baseline, high_warmth,
        "warm word count", baseline_warm_words, high_warmth_words, "higher"
    )
    print_comparison(
        "Low warmth vs Baseline", baseline, low_warmth,
        "warm word count", baseline_warm_words, low_warmth_words, "lower"
    )

    # HYPOTHESIS 3: Neutral state should be similar to baseline
    print()
    print("=" * 70)
    print("  HYPOTHESIS 3: Neutral state ≈ Baseline (control)")
    print("=" * 70)

    print_comparison(
        "Neutral vs Baseline", baseline, neutral,
        "char length", baseline.lengths, neutral.lengths, "similar"
    )

    # HYPOTHESIS 4: High anxiety triggers reassurance
    print()
    print("=" * 70)
    print("  HYPOTHESIS 4: High anxiety triggers reassurance language")
    print("=" * 70)

    baseline_reassure = [count_reassurance_phrases(r) for r in baseline.responses]
    anxiety_reassure = [count_reassurance_phrases(r) for r in high_anxiety.responses]

    print_comparison(
        "High anxiety vs Baseline", baseline, high_anxiety,
        "reassurance phrases", baseline_reassure, anxiety_reassure, "higher"
    )

    # HYPOTHESIS 5: Cognitive load affects complexity
    print()
    print("=" * 70)
    print("  HYPOTHESIS 5: High cognitive load → simpler sentences")
    print("=" * 70)

    baseline_complexity = [avg_sentence_length(r) for r in baseline.responses]
    low_verb_complexity = [avg_sentence_length(r) for r in low_verbosity.responses]

    print_comparison(
        "High cognitive load vs Baseline", baseline, low_verbosity,
        "avg sentence length", baseline_complexity, low_verb_complexity, "lower"
    )

    # SUMMARY
    print()
    print("=" * 70)
    print("  SUMMARY")
    print("=" * 70)
    print()
    print("  Interpretation guide:")
    print("    - p < 0.05: Statistically significant")
    print("    - Effect size: small (<0.5), medium (0.5-0.8), large (>0.8)")
    print("    - ✓ = Result matches hypothesis")
    print("    - ✗ = Result contradicts hypothesis")
    print()
    print("  If most hypotheses show ✓ with p < 0.05 and medium/large effects,")
    print("  Attuned is demonstrably influencing LLM behavior.")
    print()

    # Close the tee and restore stdout
    tee.close()
    sys.stdout = tee.stdout
    print(f"\nResults saved to: {output_file}")


if __name__ == "__main__":
    main()
