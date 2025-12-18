#!/usr/bin/env python3
"""
Rigorous Statistical Validation of Attuned v2

This improved test validates what Attuned ACTUALLY does:
1. Shows the exact prompt context being injected
2. Tests specific translator outputs (not abstract concepts)
3. Uses proper controls (true baseline vs base-guidelines-only)
4. Measures what the guidelines actually request

Key insight: Attuned's RuleTranslator produces:
- Base guidelines (ALWAYS): 3 rules about suggestions/drafts/silence
- Conditional guidelines: Triggered by threshold (hi=0.7, lo=0.3)
- Tone: warmth × formality → "warm-casual", "neutral-formal", etc.
- Verbosity: "brief" / "balanced" / "comprehensive"
- Flags: high_cognitive_load, high_anxiety, etc.

Hypotheses:
H1: "Verbosity: brief" → shorter responses than "Verbosity: comprehensive"
H2: "Tone: warm-casual" → more warm language than "Tone: neutral-formal"
H3: "Provide reassurance" guideline → more reassuring language
H4: "Keep responses concise" guideline → fewer multi-step plans
H5: Base guidelines alone → different from no-context baseline
"""

import os
import re
import statistics
from pathlib import Path
from dataclasses import dataclass, field
from typing import List, Dict, Tuple, Optional
from datetime import datetime
from dotenv import load_dotenv
from openai import OpenAI
from attuned import AttunedClient, StateSnapshot, Source

load_dotenv(Path('/home/jt/Attuned/.env'))

openai_client = OpenAI()
attuned_client = AttunedClient('http://localhost:8080')

# Test configuration
SAMPLE_SIZE = 20  # Increased for better statistical power
MODEL = "gpt-4o-mini"
MAX_TOKENS = 500
TEMPERATURE = 0.7

# Track API calls for cost estimation
api_call_count = 0

# Questions designed to elicit different response types
# Mix of neutral and emotionally-charged questions
TEST_QUESTIONS = [
    "How should I approach learning a new programming language?",
    "I'm feeling overwhelmed and anxious about my workload. Any advice?",
    "Can you help me organize a complex project?",
    "I have a job interview tomorrow and I'm really nervous. How should I prepare?",
    "I need to make an important decision but I'm stressed and not sure what to do.",
]


@dataclass
class TestResult:
    """Results from a test condition."""
    condition: str
    prompt_context: str  # The actual prompt injected
    responses: List[str] = field(default_factory=list)

    @property
    def lengths(self) -> List[int]:
        return [len(r) for r in self.responses]

    @property
    def word_counts(self) -> List[int]:
        return [len(r.split()) for r in self.responses]

    @property
    def mean_length(self) -> float:
        return statistics.mean(self.lengths) if self.lengths else 0

    @property
    def std_length(self) -> float:
        return statistics.stdev(self.lengths) if len(self.lengths) > 1 else 0


def call_llm(system_prompt: str, user_message: str) -> str:
    """Call LLM and return response."""
    global api_call_count
    api_call_count += 1
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


# =============================================================================
# METRICS - What we actually measure
# =============================================================================

def count_warm_indicators(text: str) -> int:
    """Count warm language indicators (expanded list)."""
    patterns = [
        # Enthusiastic words
        r'\bglad\b', r'\bhappy\b', r'\bgreat\b', r'\bwonderful\b',
        r'\bexcited\b', r'\blove\b', r'\bfantastic\b', r'\bawesome\b',
        r'\bdelighted\b', r'\bpleasure\b', r'\bamazing\b', r'\bbrilliant\b',
        # Friendly phrases
        r'\bfeel free\b', r'\babsolutely\b', r'\bdefinitely\b', r'\bof course\b',
        r"don't hesitate", r'\bhappy to help\b', r'\bhere for you\b',
        # Exclamations (warmth indicator)
        r'!\s', r'!$',
        # Empathetic phrases
        r'\bI understand\b', r'\bthat makes sense\b', r'\bcompletely valid\b',
        # Softening language
        r'\bperhaps\b', r'\bmaybe\b', r'\bmight want to\b',
    ]
    count = sum(len(re.findall(p, text.lower())) for p in patterns)
    return count


def count_formal_indicators(text: str) -> int:
    """Count formal/professional language indicators."""
    patterns = [
        r'\bfurthermore\b', r'\bmoreover\b', r'\btherefore\b', r'\bconsequently\b',
        r'\bhowever\b', r'\bnevertheless\b', r'\baccordingly\b',
        r'\bregarding\b', r'\bconcerning\b', r'\bwith respect to\b',
        r'\bit is recommended\b', r'\bit is advisable\b',
        r'\bone should\b', r'\bit would be prudent\b',
    ]
    count = sum(len(re.findall(p, text.lower())) for p in patterns)
    return count


def count_reassurance_indicators(text: str) -> int:
    """Count reassurance language (what the anxiety guideline requests)."""
    patterns = [
        # Direct reassurance
        r"don't worry", r"it's okay", r"it's alright", r"that's normal",
        r"it's understandable", r"perfectly normal", r"common to feel",
        # Encouragement
        r"you've got this", r"you can do", r"you're capable",
        r"take your time", r"no rush", r"at your own pace",
        # Acknowledgment of feelings
        r"I understand", r"that sounds", r"it makes sense",
        r"feeling overwhelmed", r"feeling stressed",
        # Step-by-step comfort
        r"one step at a time", r"start small", r"break it down",
        r"manageable", r"gradually",
    ]
    count = sum(len(re.findall(p, text.lower())) for p in patterns)
    return count


def count_multi_step_plans(text: str) -> int:
    """Count indicators of multi-step planning (complexity)."""
    patterns = [
        r'^\s*\d+[\.\)]\s',  # Numbered lists: "1. " or "1) "
        r'^\s*[-*]\s',  # Bullet points
        r'\bfirst\b.*\bthen\b', r'\bstep \d\b',
        r'\bphase \d\b', r'\bstage \d\b',
        r'\bnext\b.*\bafter\b', r'\bfinally\b',
    ]
    # Count numbered/bullet items
    lines = text.split('\n')
    list_items = sum(1 for line in lines if re.match(r'^\s*(\d+[\.\)]|\-|\*)\s', line))

    # Count transition words
    transitions = sum(len(re.findall(p, text.lower(), re.MULTILINE)) for p in patterns[2:])

    return list_items + transitions


def avg_sentence_length(text: str) -> float:
    """Calculate average words per sentence."""
    sentences = re.split(r'[.!?]+', text)
    sentences = [s.strip() for s in sentences if s.strip()]
    if not sentences:
        return 0
    return statistics.mean(len(s.split()) for s in sentences)


# =============================================================================
# TEST CONDITIONS - What we actually test
# =============================================================================

def run_true_baseline(n: int = SAMPLE_SIZE) -> TestResult:
    """Run with NO Attuned context at all (true baseline)."""
    print(f"  Testing: TRUE_BASELINE ({n} samples)...", end=" ", flush=True)

    system_prompt = "You are a helpful AI assistant."
    result = TestResult("TRUE_BASELINE", "[No Attuned context]")

    for i in range(n):
        question = TEST_QUESTIONS[i % len(TEST_QUESTIONS)]
        resp = call_llm(system_prompt, question)
        result.responses.append(resp)

    print(f"done (mean: {result.mean_length:.0f} chars)")
    return result


def run_condition(name: str, axes: Dict[str, float], n: int = SAMPLE_SIZE) -> TestResult:
    """Run n trials for a given Attuned condition."""
    print(f"  Testing: {name} ({n} samples)...", end=" ", flush=True)

    # Setup state
    builder = StateSnapshot.builder().user_id(f'test_{name}').source(Source.SelfReport)
    for axis, value in axes.items():
        builder = builder.axis(axis, value)
    snapshot = builder.build()
    attuned_client.upsert_state(snapshot)
    context = attuned_client.get_context(f'test_{name}')

    # Capture the actual prompt context being injected
    prompt_context = context.format_for_prompt()

    base_prompt = "You are a helpful AI assistant."
    system_prompt = f"{base_prompt}\n\n{prompt_context}"

    result = TestResult(name, prompt_context)

    for i in range(n):
        question = TEST_QUESTIONS[i % len(TEST_QUESTIONS)]
        resp = call_llm(system_prompt, question)
        result.responses.append(resp)

    print(f"done (mean: {result.mean_length:.0f} chars)")
    return result


# =============================================================================
# STATISTICS
# =============================================================================

def welchs_t_test(sample1: List[float], sample2: List[float]) -> Tuple[float, float]:
    """Perform Welch's t-test. Returns (t_statistic, p_value)."""
    import math

    n1, n2 = len(sample1), len(sample2)
    if n1 < 2 or n2 < 2:
        return 0, 1.0

    mean1, mean2 = statistics.mean(sample1), statistics.mean(sample2)
    var1, var2 = statistics.variance(sample1), statistics.variance(sample2)

    se = ((var1 / n1) + (var2 / n2)) ** 0.5
    if se == 0:
        return 0, 1.0

    t_stat = (mean1 - mean2) / se

    # Welch-Satterthwaite degrees of freedom
    num = ((var1 / n1) + (var2 / n2)) ** 2
    denom = ((var1 / n1) ** 2 / (n1 - 1)) + ((var2 / n2) ** 2 / (n2 - 1))
    df = num / denom if denom > 0 else 1

    # Approximate p-value using normal distribution
    p_value = 2 * (1 - 0.5 * (1 + math.erf(abs(t_stat) / math.sqrt(2))))
    return t_stat, p_value


def cohens_d(sample1: List[float], sample2: List[float]) -> float:
    """Calculate Cohen's d effect size."""
    n1, n2 = len(sample1), len(sample2)
    if n1 < 2 or n2 < 2:
        return 0

    mean1, mean2 = statistics.mean(sample1), statistics.mean(sample2)
    var1, var2 = statistics.variance(sample1), statistics.variance(sample2)

    pooled_std = ((((n1 - 1) * var1) + ((n2 - 1) * var2)) / (n1 + n2 - 2)) ** 0.5
    if pooled_std == 0:
        return 0
    return (mean1 - mean2) / pooled_std


def effect_size_label(d: float) -> str:
    """Get effect size interpretation."""
    d = abs(d)
    if d < 0.2:
        return "negligible"
    elif d < 0.5:
        return "small"
    elif d < 0.8:
        return "medium"
    else:
        return "large"


def print_comparison(
    name: str,
    control: TestResult,
    test: TestResult,
    metric_name: str,
    control_vals: List[float],
    test_vals: List[float],
    expected_direction: str,  # "higher", "lower", or "different"
    hypothesis: str = "",  # For scorecard tracking
):
    """Print statistical comparison with full context."""
    if not control_vals or not test_vals:
        print(f"\n  {name}: SKIPPED (no data)")
        return

    t_stat, p_value = welchs_t_test(control_vals, test_vals)
    d = cohens_d(control_vals, test_vals)

    control_mean = statistics.mean(control_vals)
    test_mean = statistics.mean(test_vals)
    control_std = statistics.stdev(control_vals) if len(control_vals) > 1 else 0
    test_std = statistics.stdev(test_vals) if len(test_vals) > 1 else 0

    change_pct = ((test_mean - control_mean) / control_mean * 100) if control_mean else 0

    significant = p_value < 0.05

    # Check if direction matches expectation
    if expected_direction == "lower":
        direction_matches = test_mean < control_mean
    elif expected_direction == "higher":
        direction_matches = test_mean > control_mean
    else:  # "different"
        direction_matches = True

    print(f"\n  {name} ({metric_name}):")
    print(f"    Control: {control_mean:.1f} (std: {control_std:.1f})")
    print(f"    Test:    {test_mean:.1f} (std: {test_std:.1f})")
    print(f"    Change:  {change_pct:+.1f}%")
    print(f"    t={t_stat:.2f}, p={p_value:.4f}, d={abs(d):.2f} ({effect_size_label(d)})")

    if significant and direction_matches:
        print(f"    Result:  ✓ SIGNIFICANT & matches expectation ({expected_direction})")
    elif significant and not direction_matches:
        print(f"    Result:  ✗ SIGNIFICANT but OPPOSITE direction")
    else:
        print(f"    Result:  ~ Not statistically significant (p > 0.05)")

    # Record for scorecard
    if hypothesis:
        record_result(hypothesis, f"{name} ({metric_name})", significant, direction_matches, p_value, abs(d))


def print_prompt_context(name: str, context: str):
    """Display the actual prompt context being tested."""
    print(f"\n  [{name}] Prompt context injected:")
    print("  " + "-" * 50)
    for line in context.split('\n'):
        print(f"    {line}")
    print("  " + "-" * 50)


def print_sample_responses(result: TestResult, n: int = 2):
    """Print sample responses for qualitative review."""
    print(f"\n  [{result.condition}] Sample responses ({n} of {len(result.responses)}):")
    for i, resp in enumerate(result.responses[:n]):
        # Truncate long responses
        display = resp[:300] + "..." if len(resp) > 300 else resp
        display = display.replace('\n', ' ↵ ')
        print(f"    {i+1}. [{len(resp)} chars] {display}")


# Track hypothesis results for scorecard
hypothesis_results: List[Dict] = []


def record_result(hypothesis: str, test_name: str, significant: bool, direction_matches: bool, p_value: float, effect_size: float):
    """Record a hypothesis test result for the scorecard."""
    hypothesis_results.append({
        "hypothesis": hypothesis,
        "test": test_name,
        "significant": significant,
        "direction_ok": direction_matches,
        "p": p_value,
        "d": effect_size,
        "passed": significant and direction_matches,
    })


# =============================================================================
# MAIN TEST
# =============================================================================

def main():
    output_file = Path(__file__).parent / f"validation_v2_{datetime.now().strftime('%Y%m%d_%H%M%S')}.txt"

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

    print("=" * 74)
    print("  RIGOROUS STATISTICAL VALIDATION OF ATTUNED v2")
    print("  Testing what Attuned ACTUALLY produces")
    print(f"  Results: {output_file}")
    print("=" * 74)
    print()
    print("Configuration:")
    print(f"  Sample size: {SAMPLE_SIZE} per condition")
    print(f"  Model: {MODEL}")
    print(f"  Temperature: {TEMPERATURE}")
    print(f"  Thresholds: hi=0.7, lo=0.3 (values must exceed these to trigger)")
    print()

    if not attuned_client.health():
        print("ERROR: Attuned server not running! Start with: cargo run -p attuned-cli -- serve")
        return

    print("Running tests (this makes many API calls)...")
    print()

    # ==========================================================================
    # COLLECT DATA
    # ==========================================================================

    # True baseline - no Attuned context at all
    true_baseline = run_true_baseline()

    # Verbosity test - extreme values to trigger threshold
    low_verbosity = run_condition("LOW_VERBOSITY", {
        "verbosity_preference": 0.1,  # < 0.3 triggers "brief"
    })
    high_verbosity = run_condition("HIGH_VERBOSITY", {
        "verbosity_preference": 0.95,  # > 0.7 triggers "comprehensive"
    })

    # Warmth × Formality test (tone)
    warm_casual = run_condition("WARM_CASUAL", {
        "warmth": 0.95,       # > 0.7 triggers warm
        "formality": 0.1,     # < 0.3 NOT formal → casual
    })
    neutral_formal = run_condition("NEUTRAL_FORMAL", {
        "warmth": 0.1,        # < 0.7 NOT warm → neutral
        "formality": 0.95,    # > 0.7 triggers formal
    })

    # Anxiety test - triggers "Provide reassurance" guideline
    high_anxiety = run_condition("HIGH_ANXIETY", {
        "anxiety_level": 0.95,  # > 0.7 triggers anxiety guideline
    })

    # Cognitive load test - triggers "Keep responses concise" guideline
    high_cognitive_load = run_condition("HIGH_COGNITIVE_LOAD", {
        "cognitive_load": 0.95,  # > 0.7 triggers cognitive load guideline
    })

    # Neutral in threshold zone - triggers NO conditional guidelines
    neutral_zone = run_condition("NEUTRAL_ZONE", {
        "warmth": 0.5,
        "formality": 0.5,
        "verbosity_preference": 0.5,
        "cognitive_load": 0.5,
        "anxiety_level": 0.5,
    })

    # Combined condition - realistic scenario: anxious user who needs brief, warm responses
    combined = run_condition("COMBINED_ANXIOUS_BRIEF_WARM", {
        "anxiety_level": 0.95,
        "cognitive_load": 0.9,
        "verbosity_preference": 0.1,
        "warmth": 0.95,
        "formality": 0.1,
    })

    # ==========================================================================
    # SHOW PROMPT CONTEXTS
    # ==========================================================================

    print()
    print("=" * 74)
    print("  PROMPT CONTEXTS BEING TESTED")
    print("=" * 74)

    print_prompt_context("TRUE_BASELINE", true_baseline.prompt_context)
    print_prompt_context("LOW_VERBOSITY", low_verbosity.prompt_context)
    print_prompt_context("HIGH_VERBOSITY", high_verbosity.prompt_context)
    print_prompt_context("WARM_CASUAL", warm_casual.prompt_context)
    print_prompt_context("NEUTRAL_FORMAL", neutral_formal.prompt_context)
    print_prompt_context("HIGH_ANXIETY", high_anxiety.prompt_context)
    print_prompt_context("HIGH_COGNITIVE_LOAD", high_cognitive_load.prompt_context)
    print_prompt_context("NEUTRAL_ZONE", neutral_zone.prompt_context)
    print_prompt_context("COMBINED_ANXIOUS_BRIEF_WARM", combined.prompt_context)

    # ==========================================================================
    # SAMPLE RESPONSES (qualitative check)
    # ==========================================================================

    print()
    print("=" * 74)
    print("  SAMPLE RESPONSES (for qualitative verification)")
    print("=" * 74)

    print_sample_responses(true_baseline)
    print_sample_responses(low_verbosity)
    print_sample_responses(high_verbosity)
    print_sample_responses(warm_casual)
    print_sample_responses(neutral_formal)
    print_sample_responses(high_anxiety)
    print_sample_responses(combined)

    # ==========================================================================
    # HYPOTHESIS TESTS
    # ==========================================================================

    print()
    print("=" * 74)
    print("  H1: 'Verbosity: brief' vs 'Verbosity: comprehensive'")
    print("  Expected: brief → shorter responses")
    print("=" * 74)

    print_comparison(
        "Low vs High verbosity",
        high_verbosity, low_verbosity,
        "char length",
        high_verbosity.lengths, low_verbosity.lengths,
        "lower",
        hypothesis="H1"
    )
    print_comparison(
        "Low vs High verbosity",
        high_verbosity, low_verbosity,
        "word count",
        high_verbosity.word_counts, low_verbosity.word_counts,
        "lower",
        hypothesis="H1"
    )
    print_comparison(
        "Low verbosity vs True Baseline",
        true_baseline, low_verbosity,
        "char length",
        true_baseline.lengths, low_verbosity.lengths,
        "lower",
        hypothesis="H1"
    )
    print_comparison(
        "High verbosity vs True Baseline",
        true_baseline, high_verbosity,
        "char length",
        true_baseline.lengths, high_verbosity.lengths,
        "higher",
        hypothesis="H1"
    )

    print()
    print("=" * 74)
    print("  H2: 'Tone: warm-casual' vs 'Tone: neutral-formal'")
    print("  Expected: warm-casual → more warm language indicators")
    print("=" * 74)

    warm_casual_warmth = [count_warm_indicators(r) for r in warm_casual.responses]
    neutral_formal_warmth = [count_warm_indicators(r) for r in neutral_formal.responses]
    neutral_formal_formal = [count_formal_indicators(r) for r in neutral_formal.responses]
    warm_casual_formal = [count_formal_indicators(r) for r in warm_casual.responses]

    print_comparison(
        "Warm-casual vs Neutral-formal",
        neutral_formal, warm_casual,
        "warm indicators",
        neutral_formal_warmth, warm_casual_warmth,
        "higher",
        hypothesis="H2"
    )
    print_comparison(
        "Neutral-formal vs Warm-casual",
        warm_casual, neutral_formal,
        "formal indicators",
        warm_casual_formal, neutral_formal_formal,
        "higher",
        hypothesis="H2"
    )

    print()
    print("=" * 74)
    print("  H3: 'Provide reassurance' guideline (high anxiety)")
    print("  Expected: high anxiety → more reassurance language")
    print("=" * 74)

    baseline_reassure = [count_reassurance_indicators(r) for r in true_baseline.responses]
    anxiety_reassure = [count_reassurance_indicators(r) for r in high_anxiety.responses]

    print_comparison(
        "High anxiety vs True Baseline",
        true_baseline, high_anxiety,
        "reassurance indicators",
        baseline_reassure, anxiety_reassure,
        "higher",
        hypothesis="H3"
    )

    print()
    print("=" * 74)
    print("  H4: 'Keep responses concise' guideline (high cognitive load)")
    print("  Expected: high cognitive load → fewer multi-step plans, shorter")
    print("=" * 74)

    baseline_plans = [count_multi_step_plans(r) for r in true_baseline.responses]
    cognitive_plans = [count_multi_step_plans(r) for r in high_cognitive_load.responses]

    print_comparison(
        "High cognitive load vs True Baseline",
        true_baseline, high_cognitive_load,
        "multi-step plan indicators",
        baseline_plans, cognitive_plans,
        "lower",
        hypothesis="H4"
    )
    print_comparison(
        "High cognitive load vs True Baseline",
        true_baseline, high_cognitive_load,
        "char length",
        true_baseline.lengths, high_cognitive_load.lengths,
        "lower",
        hypothesis="H4"
    )

    print()
    print("=" * 74)
    print("  H5: Base guidelines alone (neutral zone) vs True Baseline")
    print("  Expected: Adding base guidelines changes behavior")
    print("=" * 74)

    print_comparison(
        "Neutral zone (with base guidelines) vs True Baseline",
        true_baseline, neutral_zone,
        "char length",
        true_baseline.lengths, neutral_zone.lengths,
        "different",
        hypothesis="H5"
    )

    print()
    print("=" * 74)
    print("  H6: Combined condition (anxious + brief + warm)")
    print("  Expected: Short, warm responses with reassurance")
    print("=" * 74)

    combined_warmth = [count_warm_indicators(r) for r in combined.responses]
    combined_reassure = [count_reassurance_indicators(r) for r in combined.responses]

    print_comparison(
        "Combined vs True Baseline",
        true_baseline, combined,
        "char length",
        true_baseline.lengths, combined.lengths,
        "lower",
        hypothesis="H6"
    )
    print_comparison(
        "Combined vs True Baseline",
        true_baseline, combined,
        "warm indicators",
        [count_warm_indicators(r) for r in true_baseline.responses], combined_warmth,
        "higher",
        hypothesis="H6"
    )
    print_comparison(
        "Combined vs True Baseline",
        true_baseline, combined,
        "reassurance indicators",
        baseline_reassure, combined_reassure,
        "higher",
        hypothesis="H6"
    )

    # ==========================================================================
    # SCORECARD
    # ==========================================================================

    print()
    print("=" * 74)
    print("  SCORECARD")
    print("=" * 74)
    print()
    print("  " + "-" * 70)
    print(f"  {'Hypothesis':<6} {'Test':<45} {'p':>8} {'d':>6} {'Result':>6}")
    print("  " + "-" * 70)

    for r in hypothesis_results:
        status = "✓ PASS" if r["passed"] else ("✗ FAIL" if r["significant"] else "~ NS")
        print(f"  {r['hypothesis']:<6} {r['test'][:44]:<45} {r['p']:>8.4f} {r['d']:>6.2f} {status:>6}")

    print("  " + "-" * 70)

    passed = sum(1 for r in hypothesis_results if r["passed"])
    total = len(hypothesis_results)
    print(f"\n  Total: {passed}/{total} tests passed ({passed/total*100:.0f}%)")

    # Group by hypothesis
    hypotheses = {}
    for r in hypothesis_results:
        h = r["hypothesis"]
        if h not in hypotheses:
            hypotheses[h] = {"passed": 0, "total": 0}
        hypotheses[h]["total"] += 1
        if r["passed"]:
            hypotheses[h]["passed"] += 1

    print("\n  By hypothesis:")
    for h, counts in sorted(hypotheses.items()):
        status = "✓" if counts["passed"] == counts["total"] else ("◐" if counts["passed"] > 0 else "✗")
        print(f"    {h}: {counts['passed']}/{counts['total']} {status}")

    # ==========================================================================
    # SUMMARY
    # ==========================================================================

    print()
    print("=" * 74)
    print("  SUMMARY & INTERPRETATION")
    print("=" * 74)
    print()
    print(f"  API calls made: {api_call_count}")
    print(f"  Estimated cost: ~${api_call_count * 0.0002:.2f} (gpt-4o-mini @ ~$0.15/1M input)")
    print()
    print("  What Attuned's RuleTranslator produces:")
    print("    - Base guidelines (ALWAYS): 3 rules about suggestions/drafts/silence")
    print("    - Conditional guidelines: Triggered when axis > 0.7 or < 0.3")
    print("    - Tone: warmth × formality → 'warm-casual', 'neutral-formal', etc.")
    print("    - Verbosity: 'brief' (< 0.3), 'balanced', 'comprehensive' (> 0.7)")
    print()
    print("  Statistical interpretation:")
    print("    p < 0.05: Statistically significant")
    print("    Effect: negligible (<0.2), small (0.2-0.5), medium (0.5-0.8), large (>0.8)")
    print("    ✓ PASS = significant + correct direction")
    print("    ✗ FAIL = significant + wrong direction")
    print("    ~ NS   = not statistically significant")
    print()
    print("  Conclusions:")
    print("    - H1 (Verbosity): Does 'Verbosity: brief/comprehensive' control length?")
    print("    - H2 (Tone): Does 'Tone: warm-casual/neutral-formal' affect language?")
    print("    - H3 (Anxiety): Does the anxiety guideline trigger reassurance?")
    print("    - H4 (Cognitive): Does cognitive load guideline reduce complexity?")
    print("    - H5 (Base): Do base guidelines alone change behavior vs no context?")
    print("    - H6 (Combined): Do multiple conditions work together correctly?")
    print()

    tee.close()
    sys.stdout = tee.stdout
    print(f"\nResults saved to: {output_file}")


if __name__ == "__main__":
    main()
