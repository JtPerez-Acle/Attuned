#!/usr/bin/env python3
"""
Dreaddit Dataset Validation Script for attuned-infer

This script validates our anxiety/stress inference features against the Dreaddit
dataset - a public Reddit corpus with ~3.5K human-labeled posts for stress detection.

Paper: Turcan & McKeown (2019) "Dreaddit: A Reddit Dataset for Stress Analysis"
       https://aclanthology.org/D19-6213/

Usage:
    pip install pandas numpy scipy scikit-learn datasets
    python scripts/validate_dreaddit.py
"""

import re
import json
import sys
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional
from collections import Counter

import numpy as np
import pandas as pd
from scipy import stats
from sklearn.linear_model import LogisticRegression
from sklearn.model_selection import train_test_split, cross_val_score
from sklearn.metrics import classification_report, confusion_matrix
from sklearn.preprocessing import StandardScaler

# Try to import datasets library for HuggingFace
try:
    from datasets import load_dataset
    HAS_DATASETS = True
except ImportError:
    HAS_DATASETS = False
    print("Warning: 'datasets' library not found. Install with: pip install datasets")


# =============================================================================
# Linguistic Feature Extraction (Python port of attuned-infer/src/features.rs)
# =============================================================================

# Word lists from attuned-infer
HEDGE_WORDS = [
    "maybe", "perhaps", "possibly", "probably", "might", "could",
    "sort of", "kind of", "i think", "i guess", "i suppose",
    "seems", "appears", "likely", "unlikely", "somewhat",
    "fairly", "rather", "quite", "a bit", "a little",
    "in my opinion", "i believe", "i feel", "not sure",
]

CERTAINTY_MARKERS = [
    "definitely", "certainly", "absolutely", "clearly", "obviously",
    "undoubtedly", "surely", "of course", "without doubt",
    "no question", "for sure", "guaranteed", "always", "never",
    "must", "will", "know for a fact",
]

URGENCY_WORDS = [
    "urgent", "asap", "immediately", "now", "hurry",
    "emergency", "critical", "deadline", "rush", "quick",
    "fast", "right away", "time-sensitive", "pressing",
]

POLITENESS_MARKERS = [
    "please", "thank you", "thanks", "appreciate",
    "sorry", "excuse me", "pardon", "would you mind",
]

FIRST_PERSON = ["i", "me", "my", "mine", "myself", "we", "us", "our", "ours"]

FILLER_WORDS = [
    "um", "uh", "like", "you know", "basically",
    "actually", "literally", "honestly", "well",
]

# NEW: Negative emotion words (from TASK-016 / DEEP_RESEARCH.md)
NEGATIVE_EMOTION_WORDS = [
    # Anxiety-specific (LIWC Anxiety category)
    "worried", "worry", "worries", "anxious", "anxiety", "nervous",
    "afraid", "fear", "scared", "panic", "stressed", "stress",
    "tense", "uneasy", "dread", "dreading",
    # General negative affect
    "upset", "frustrated", "annoyed", "angry", "mad",
    "sad", "depressed", "hopeless", "miserable",
    "terrible", "awful", "horrible", "worst",
    # Distress markers
    "struggling", "suffering", "overwhelmed", "exhausted",
    "desperate", "helpless", "stuck", "lost",
]

# NEW: Absolutist words (linked to anxious/depressive thinking)
ABSOLUTIST_WORDS = [
    "always", "never", "nothing", "everything", "completely",
    "totally", "absolutely", "entirely", "impossible", "definitely",
]


@dataclass
class LinguisticFeatures:
    """Port of attuned-infer's LinguisticFeatures struct."""
    # Basic counts
    char_count: int = 0
    word_count: int = 0
    sentence_count: int = 0

    # Complexity
    avg_word_length: float = 0.0
    avg_sentence_length: float = 0.0
    long_word_ratio: float = 0.0
    reading_grade_level: float = 0.0

    # Punctuation/emphasis
    exclamation_ratio: float = 0.0
    question_ratio: float = 0.0
    caps_word_count: int = 0
    caps_ratio: float = 0.0

    # Linguistic markers
    hedge_count: int = 0
    hedge_density: float = 0.0
    certainty_count: int = 0
    contraction_ratio: float = 0.0
    politeness_count: int = 0
    first_person_ratio: float = 0.0
    urgency_word_count: int = 0
    imperative_count: int = 0
    filler_ratio: float = 0.0

    # NEW features (from TASK-016)
    negative_emotion_count: int = 0
    negative_emotion_density: float = 0.0
    absolutist_count: int = 0
    absolutist_density: float = 0.0

    def uncertainty_score(self) -> float:
        """Compute uncertainty score (anxiety proxy) - matches Rust implementation."""
        hedge_component = min(self.hedge_density * 2.0, 1.0) * 0.6
        question_component = min(self.question_ratio * 3.0, 1.0) * 0.4
        return hedge_component + question_component

    def uncertainty_score_v2(self) -> float:
        """Enhanced uncertainty score with negative emotions and first-person."""
        base = self.uncertainty_score()
        neg_emotion = min(self.negative_emotion_density * 2.0, 1.0)
        first_person = min(self.first_person_ratio * 2.0, 1.0)

        # Weighted combination as proposed in TASK-016
        return (
            base * 0.4 +
            neg_emotion * 0.4 +
            first_person * 0.2
        )

    def emotional_intensity(self) -> float:
        """Compute emotional intensity - matches Rust implementation."""
        exclaim = min(self.exclamation_ratio * 5.0, 1.0) * 0.5
        caps = min(self.caps_ratio * 3.0, 1.0) * 0.5
        return exclaim + caps

    def formality_score(self) -> float:
        """Compute formality score - matches Rust implementation."""
        contraction_penalty = self.contraction_ratio * 0.4
        complexity_bonus = min(self.reading_grade_level / 12.0, 1.0) * 0.4
        filler_penalty = self.filler_ratio * 0.2
        return max(0.0, min(1.0, 0.5 + complexity_bonus - contraction_penalty - filler_penalty))

    def complexity_score(self) -> float:
        """Compute complexity score - matches Rust implementation."""
        grade = min(self.reading_grade_level / 12.0, 1.0) * 0.4
        sentence_len = min(self.avg_sentence_length / 25.0, 1.0) * 0.3
        long_words = self.long_word_ratio * 0.3
        return grade + sentence_len + long_words


class LinguisticExtractor:
    """Port of attuned-infer's LinguisticExtractor."""

    def __init__(self, long_word_threshold: int = 6, min_caps_word_length: int = 2):
        self.long_word_threshold = long_word_threshold
        self.min_caps_word_length = min_caps_word_length

    def extract(self, text: str) -> LinguisticFeatures:
        """Extract all linguistic features from text."""
        if not text or not text.strip():
            return LinguisticFeatures()

        f = LinguisticFeatures()

        # Basic counts
        f.char_count = len(text)
        words = self._tokenize_words(text)
        f.word_count = len(words)
        sentences = self._count_sentences(text)
        f.sentence_count = max(sentences, 1)

        if f.word_count == 0:
            return f

        # Word-level stats
        word_lengths = [len(w) for w in words]
        f.avg_word_length = np.mean(word_lengths) if word_lengths else 0.0
        f.avg_sentence_length = f.word_count / f.sentence_count
        f.long_word_ratio = sum(1 for l in word_lengths if l > self.long_word_threshold) / f.word_count

        # Readability (Flesch-Kincaid grade level)
        syllables = sum(self._syllables_in_word(w) for w in words)
        if f.word_count > 0 and f.sentence_count > 0:
            f.reading_grade_level = (
                0.39 * (f.word_count / f.sentence_count) +
                11.8 * (syllables / f.word_count) -
                15.59
            )
            f.reading_grade_level = max(0.0, f.reading_grade_level)

        # Punctuation
        exclamations = text.count('!')
        questions = text.count('?')
        f.exclamation_ratio = exclamations / f.sentence_count
        f.question_ratio = questions / f.sentence_count

        # CAPS detection
        caps_words = [w for w in words if w.isupper() and len(w) >= self.min_caps_word_length]
        f.caps_word_count = len(caps_words)
        f.caps_ratio = f.caps_word_count / f.word_count

        # Linguistic markers
        text_lower = text.lower()

        f.hedge_count = self._count_matches(text_lower, HEDGE_WORDS)
        f.hedge_density = f.hedge_count / f.sentence_count

        f.certainty_count = self._count_matches(text_lower, CERTAINTY_MARKERS)
        f.politeness_count = self._count_matches(text_lower, POLITENESS_MARKERS)
        f.urgency_word_count = self._count_matches(text_lower, URGENCY_WORDS)

        # Contractions
        contractions = len(re.findall(r"\b\w+'\w+\b", text_lower))
        f.contraction_ratio = contractions / f.word_count

        # First person
        words_lower = [w.lower() for w in words]
        first_person_count = sum(1 for w in words_lower if w in FIRST_PERSON)
        f.first_person_ratio = first_person_count / f.word_count

        # Imperatives (rough heuristic: sentence-initial verbs)
        f.imperative_count = self._count_imperatives(text)

        # Fillers
        filler_count = self._count_matches(text_lower, FILLER_WORDS)
        f.filler_ratio = filler_count / f.word_count

        # NEW: Negative emotions
        f.negative_emotion_count = self._count_matches(text_lower, NEGATIVE_EMOTION_WORDS)
        f.negative_emotion_density = f.negative_emotion_count / f.sentence_count

        # NEW: Absolutist words
        f.absolutist_count = self._count_matches(text_lower, ABSOLUTIST_WORDS)
        f.absolutist_density = f.absolutist_count / f.sentence_count

        return f

    def _tokenize_words(self, text: str) -> list[str]:
        """Simple word tokenization."""
        return re.findall(r"\b[a-zA-Z]+(?:'[a-zA-Z]+)?\b", text)

    def _count_sentences(self, text: str) -> int:
        """Count sentences by terminal punctuation."""
        return len(re.findall(r'[.!?]+', text)) or 1

    def _syllables_in_word(self, word: str) -> int:
        """Estimate syllable count."""
        word = word.lower()
        vowels = "aeiouy"
        count = 0
        prev_vowel = False

        for char in word:
            is_vowel = char in vowels
            if is_vowel and not prev_vowel:
                count += 1
            prev_vowel = is_vowel

        # Adjustments
        if word.endswith('e') and count > 1:
            count -= 1
        if word.endswith('le') and len(word) > 2 and word[-3] not in vowels:
            count += 1

        return max(1, count)

    def _count_matches(self, text: str, word_list: list[str]) -> int:
        """Count occurrences of words/phrases in text."""
        count = 0
        for phrase in word_list:
            # Use word boundaries for single words, simple contains for phrases
            if ' ' in phrase:
                count += text.count(phrase)
            else:
                count += len(re.findall(rf'\b{re.escape(phrase)}\b', text))
        return count

    def _count_imperatives(self, text: str) -> int:
        """Rough heuristic for imperative sentences."""
        imperative_starters = [
            "do", "don't", "please", "let", "make", "take", "give",
            "get", "go", "come", "tell", "help", "stop", "start",
            "try", "send", "call", "check", "look", "see", "read",
        ]
        sentences = re.split(r'[.!?]+', text)
        count = 0
        for sent in sentences:
            words = sent.strip().lower().split()
            if words and words[0] in imperative_starters:
                count += 1
        return count


# =============================================================================
# Dataset Loading
# =============================================================================

def load_dreaddit_dataset() -> pd.DataFrame:
    """Load the Dreaddit dataset from HuggingFace or local cache."""
    if not HAS_DATASETS:
        raise ImportError("Please install datasets: pip install datasets")

    print("Loading Dreaddit dataset from HuggingFace (andreagasparini/dreaddit)...")
    dataset = load_dataset("andreagasparini/dreaddit")

    # Combine train and test for full analysis
    train_df = pd.DataFrame(dataset['train'])
    test_df = pd.DataFrame(dataset['test'])

    # Add split indicator
    train_df['split'] = 'train'
    test_df['split'] = 'test'

    df = pd.concat([train_df, test_df], ignore_index=True)

    print(f"Loaded {len(df)} posts ({len(train_df)} train, {len(test_df)} test)")
    print(f"Dataset includes LIWC features: {[c for c in df.columns if 'liwc' in c][:5]}...")
    return df


# =============================================================================
# Analysis Functions
# =============================================================================

def extract_all_features(df: pd.DataFrame, text_col: str = 'text') -> pd.DataFrame:
    """Extract linguistic features for all posts."""
    extractor = LinguisticExtractor()

    features_list = []
    for i, text in enumerate(df[text_col]):
        if i % 500 == 0:
            print(f"  Processing post {i}/{len(df)}...")
        features = extractor.extract(str(text))
        features_list.append(asdict(features))

    features_df = pd.DataFrame(features_list)
    return pd.concat([df.reset_index(drop=True), features_df], axis=1)


def compute_correlations(df: pd.DataFrame, label_col: str = 'label') -> pd.DataFrame:
    """Compute point-biserial correlations between features and stress label."""
    # Our custom features
    feature_cols = [
        'uncertainty_score', 'uncertainty_score_v2',
        'hedge_density', 'question_ratio', 'first_person_ratio',
        'negative_emotion_density', 'absolutist_density',
        'emotional_intensity', 'formality_score', 'complexity_score',
        'exclamation_ratio', 'caps_ratio', 'word_count',
    ]

    # Also include LIWC features from the dataset for comparison
    liwc_comparison = [
        'lex_liwc_negemo',  # LIWC negative emotion
        'lex_liwc_anx',     # LIWC anxiety
        'lex_liwc_tentat',  # LIWC tentative (hedge words)
        'lex_liwc_i',       # LIWC first-person I
        'lex_liwc_certain', # LIWC certainty
        'lex_liwc_Exclam',  # LIWC exclamation
        'lex_liwc_QMark',   # LIWC question marks
    ]
    feature_cols.extend([c for c in liwc_comparison if c in df.columns])

    # Compute derived scores
    extractor = LinguisticExtractor()
    df = df.copy()

    # Add computed scores
    df['uncertainty_score'] = df.apply(
        lambda r: LinguisticFeatures(
            hedge_density=r['hedge_density'],
            question_ratio=r['question_ratio']
        ).uncertainty_score(),
        axis=1
    )
    df['uncertainty_score_v2'] = df.apply(
        lambda r: LinguisticFeatures(
            hedge_density=r['hedge_density'],
            question_ratio=r['question_ratio'],
            negative_emotion_density=r['negative_emotion_density'],
            first_person_ratio=r['first_person_ratio']
        ).uncertainty_score_v2(),
        axis=1
    )
    df['emotional_intensity'] = df.apply(
        lambda r: LinguisticFeatures(
            exclamation_ratio=r['exclamation_ratio'],
            caps_ratio=r['caps_ratio']
        ).emotional_intensity(),
        axis=1
    )
    df['formality_score'] = df.apply(
        lambda r: LinguisticFeatures(
            contraction_ratio=r['contraction_ratio'],
            reading_grade_level=r['reading_grade_level'],
            filler_ratio=r['filler_ratio']
        ).formality_score(),
        axis=1
    )
    df['complexity_score'] = df.apply(
        lambda r: LinguisticFeatures(
            reading_grade_level=r['reading_grade_level'],
            avg_sentence_length=r['avg_sentence_length'],
            long_word_ratio=r['long_word_ratio']
        ).complexity_score(),
        axis=1
    )

    results = []
    for col in feature_cols:
        if col not in df.columns:
            continue

        # Point-biserial correlation
        valid = df[[col, label_col]].dropna()
        if len(valid) < 10:
            continue

        r, p = stats.pointbiserialr(valid[label_col], valid[col])

        # Effect size (Cohen's d)
        stressed = valid[valid[label_col] == 1][col]
        not_stressed = valid[valid[label_col] == 0][col]

        pooled_std = np.sqrt(
            ((len(stressed) - 1) * stressed.std()**2 +
             (len(not_stressed) - 1) * not_stressed.std()**2) /
            (len(stressed) + len(not_stressed) - 2)
        )

        cohens_d = (stressed.mean() - not_stressed.mean()) / pooled_std if pooled_std > 0 else 0

        results.append({
            'feature': col,
            'correlation_r': r,
            'p_value': p,
            'cohens_d': cohens_d,
            'stressed_mean': stressed.mean(),
            'not_stressed_mean': not_stressed.mean(),
            'significant': p < 0.05,
        })

    return pd.DataFrame(results).sort_values('correlation_r', ascending=False, key=abs)


def run_classification(df: pd.DataFrame, label_col: str = 'label') -> dict:
    """Run logistic regression classification experiment."""
    # Feature sets to compare (only use features from extract_all_features)
    feature_sets = {
        'current_v1': ['hedge_density', 'question_ratio'],
        'proposed_v2': ['hedge_density', 'question_ratio', 'negative_emotion_density', 'first_person_ratio'],
        'all_features': [
            'hedge_density', 'question_ratio', 'negative_emotion_density',
            'first_person_ratio', 'absolutist_density', 'exclamation_ratio',
            'caps_ratio', 'contraction_ratio', 'word_count'
        ],
        'liwc_comparison': [
            'lex_liwc_negemo', 'lex_liwc_anx', 'lex_liwc_tentat', 'lex_liwc_i'
        ],
    }

    results = {}

    # Split data
    train_df = df[df['split'] == 'train']
    test_df = df[df['split'] == 'test']

    for name, features in feature_sets.items():
        # Prepare data
        X_train = train_df[features].fillna(0)
        y_train = train_df[label_col]
        X_test = test_df[features].fillna(0)
        y_test = test_df[label_col]

        # Scale features
        scaler = StandardScaler()
        X_train_scaled = scaler.fit_transform(X_train)
        X_test_scaled = scaler.transform(X_test)

        # Train model
        model = LogisticRegression(max_iter=1000, random_state=42)
        model.fit(X_train_scaled, y_train)

        # Evaluate
        y_pred = model.predict(X_test_scaled)
        report = classification_report(y_test, y_pred, output_dict=True)

        # Cross-validation on full data
        X_all = df[features].fillna(0)
        y_all = df[label_col]
        X_all_scaled = StandardScaler().fit_transform(X_all)
        cv_scores = cross_val_score(
            LogisticRegression(max_iter=1000, random_state=42),
            X_all_scaled, y_all, cv=5, scoring='f1'
        )

        results[name] = {
            'features': features,
            'test_precision': report['1']['precision'],
            'test_recall': report['1']['recall'],
            'test_f1': report['1']['f1-score'],
            'test_accuracy': report['accuracy'],
            'cv_f1_mean': cv_scores.mean(),
            'cv_f1_std': cv_scores.std(),
            'coefficients': dict(zip(features, model.coef_[0])),
        }

    # Add baselines
    results['random_baseline'] = {
        'features': [],
        'test_precision': 0.5,
        'test_recall': 0.5,
        'test_f1': 0.5,
        'test_accuracy': 0.5,
    }

    majority_class = y_train.mode()[0]
    majority_acc = (y_test == majority_class).mean()
    results['majority_baseline'] = {
        'features': [],
        'test_precision': majority_acc if majority_class == 1 else 0,
        'test_recall': 1.0 if majority_class == 1 else 0,
        'test_f1': 2 * majority_acc / (1 + majority_acc) if majority_class == 1 else 0,
        'test_accuracy': majority_acc,
    }

    return results


def analyze_errors(df: pd.DataFrame, label_col: str = 'label', text_col: str = 'text') -> dict:
    """Analyze classification errors to identify patterns."""
    # Use proposed_v2 features for error analysis
    features = ['hedge_density', 'question_ratio', 'negative_emotion_density', 'first_person_ratio']

    test_df = df[df['split'] == 'test'].copy()

    X = test_df[features].fillna(0)
    y = test_df[label_col]

    scaler = StandardScaler()
    X_scaled = scaler.fit_transform(X)

    model = LogisticRegression(max_iter=1000, random_state=42)
    model.fit(X_scaled, y)

    test_df['predicted'] = model.predict(X_scaled)
    test_df['prob_stressed'] = model.predict_proba(X_scaled)[:, 1]

    # Find false positives and false negatives
    fp = test_df[(test_df['predicted'] == 1) & (test_df[label_col] == 0)]
    fn = test_df[(test_df['predicted'] == 0) & (test_df[label_col] == 1)]

    # Sample examples
    fp_samples = fp.nlargest(5, 'prob_stressed')[[text_col] + features + ['prob_stressed']]
    fn_samples = fn.nsmallest(5, 'prob_stressed')[[text_col] + features + ['prob_stressed']]

    # Analyze patterns in errors
    fp_patterns = {
        'avg_hedge_density': fp['hedge_density'].mean(),
        'avg_question_ratio': fp['question_ratio'].mean(),
        'avg_negative_emotion': fp['negative_emotion_density'].mean(),
        'avg_first_person': fp['first_person_ratio'].mean(),
        'count': len(fp),
    }

    fn_patterns = {
        'avg_hedge_density': fn['hedge_density'].mean(),
        'avg_question_ratio': fn['question_ratio'].mean(),
        'avg_negative_emotion': fn['negative_emotion_density'].mean(),
        'avg_first_person': fn['first_person_ratio'].mean(),
        'count': len(fn),
    }

    return {
        'false_positive_patterns': fp_patterns,
        'false_negative_patterns': fn_patterns,
        'false_positive_samples': fp_samples.to_dict('records')[:5],
        'false_negative_samples': fn_samples.to_dict('records')[:5],
    }


def generate_report(
    df: pd.DataFrame,
    correlations: pd.DataFrame,
    classification: dict,
    errors: dict,
    output_path: Path
) -> str:
    """Generate markdown validation report."""

    # Dataset stats
    stressed_count = (df['label'] == 1).sum()
    not_stressed_count = (df['label'] == 0).sum()
    stressed_pct = stressed_count / len(df) * 100

    report = f"""# Dreaddit Validation Report

Generated: {pd.Timestamp.now().strftime('%Y-%m-%d %H:%M')}

## Dataset Statistics

- **Total posts**: {len(df):,}
- **Stressed**: {stressed_count:,} ({stressed_pct:.1f}%)
- **Not stressed**: {not_stressed_count:,} ({100-stressed_pct:.1f}%)
- **Train/Test split**: {len(df[df['split']=='train']):,} / {len(df[df['split']=='test']):,}

## Feature Correlations with Stress Label

| Feature | Correlation (r) | Effect Size (d) | p-value | Significant |
|---------|-----------------|-----------------|---------|-------------|
"""

    for _, row in correlations.iterrows():
        sig = "Yes" if row['significant'] else "No"
        p_str = f"{row['p_value']:.2e}" if row['p_value'] < 0.001 else f"{row['p_value']:.4f}"
        report += f"| {row['feature']} | {row['correlation_r']:.3f} | {row['cohens_d']:.3f} | {p_str} | {sig} |\n"

    report += """
### Key Findings

"""
    # Analyze top correlations
    top_features = correlations.head(5)
    for _, row in top_features.iterrows():
        direction = "higher" if row['correlation_r'] > 0 else "lower"
        report += f"- **{row['feature']}**: Stressed posts have {direction} values (r={row['correlation_r']:.3f}, d={row['cohens_d']:.3f})\n"

    report += """
## Classification Results

| Model | Precision | Recall | F1 | Accuracy | CV F1 |
|-------|-----------|--------|-----|----------|-------|
"""

    for name, results in classification.items():
        cv_f1 = results.get('cv_f1_mean', '-')
        cv_str = f"{cv_f1:.3f}" if isinstance(cv_f1, float) else cv_f1
        report += f"| {name} | {results['test_precision']:.3f} | {results['test_recall']:.3f} | {results['test_f1']:.3f} | {results['test_accuracy']:.3f} | {cv_str} |\n"

    # Feature importance
    if 'proposed_v2' in classification and 'coefficients' in classification['proposed_v2']:
        report += """
### Feature Importance (proposed_v2 model)

| Feature | Coefficient |
|---------|-------------|
"""
        for feat, coef in sorted(classification['proposed_v2']['coefficients'].items(),
                                  key=lambda x: abs(x[1]), reverse=True):
            report += f"| {feat} | {coef:.3f} |\n"

    report += """
## Error Analysis

### False Positives (predicted stressed, actually not)
"""
    fp = errors['false_positive_patterns']
    report += f"""
- Count: {fp['count']}
- Avg hedge density: {fp['avg_hedge_density']:.3f}
- Avg question ratio: {fp['avg_question_ratio']:.3f}
- Avg negative emotion: {fp['avg_negative_emotion']:.3f}
- Avg first person ratio: {fp['avg_first_person']:.3f}

### False Negatives (predicted not stressed, actually stressed)
"""
    fn = errors['false_negative_patterns']
    report += f"""
- Count: {fn['count']}
- Avg hedge density: {fn['avg_hedge_density']:.3f}
- Avg question ratio: {fn['avg_question_ratio']:.3f}
- Avg negative emotion: {fn['avg_negative_emotion']:.3f}
- Avg first person ratio: {fn['avg_first_person']:.3f}

## Recommendations

Based on this validation:

"""
    # Generate recommendations based on results
    v1_f1 = classification.get('current_v1', {}).get('test_f1', 0)
    v2_f1 = classification.get('proposed_v2', {}).get('test_f1', 0)

    if v2_f1 > v1_f1:
        improvement = (v2_f1 - v1_f1) / v1_f1 * 100
        report += f"1. **ADD negative emotion words**: Proposed v2 improves F1 by {improvement:.1f}% over current implementation\n"

    neg_emotion_corr = correlations[correlations['feature'] == 'negative_emotion_density']
    if len(neg_emotion_corr) > 0 and neg_emotion_corr.iloc[0]['significant']:
        r = neg_emotion_corr.iloc[0]['correlation_r']
        report += f"2. **Negative emotion density** shows significant correlation (r={r:.3f}) - confirms DEEP_RESEARCH.md recommendation\n"

    first_person_corr = correlations[correlations['feature'] == 'first_person_ratio']
    if len(first_person_corr) > 0 and first_person_corr.iloc[0]['significant']:
        r = first_person_corr.iloc[0]['correlation_r']
        report += f"3. **First-person ratio** shows significant correlation (r={r:.3f}) - confirms DEEP_RESEARCH.md recommendation\n"

    report += """
## References

- Turcan & McKeown (2019). "Dreaddit: A Reddit Dataset for Stress Analysis in Social Media" https://aclanthology.org/D19-6213/
- DEEP_RESEARCH.md - validation evidence summary
- attuned-infer/src/features.rs - current feature implementation
"""

    # Save report
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(report)

    return report


# =============================================================================
# Main
# =============================================================================

def main():
    print("=" * 60)
    print("Dreaddit Validation for attuned-infer")
    print("=" * 60)

    # Load dataset
    print("\n[1/5] Loading dataset...")
    try:
        df = load_dreaddit_dataset()
    except Exception as e:
        print(f"Error loading dataset: {e}")
        print("\nTo install dependencies: pip install pandas numpy scipy scikit-learn datasets")
        sys.exit(1)

    # Extract features
    print("\n[2/5] Extracting linguistic features...")
    df = extract_all_features(df)

    # Save intermediate results
    cache_path = Path("tasks/reports/dreaddit_features.csv")
    cache_path.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(cache_path, index=False)
    print(f"  Cached features to {cache_path}")

    # Compute correlations
    print("\n[3/5] Computing correlations...")
    correlations = compute_correlations(df)
    print("\nTop correlations with stress label:")
    print(correlations[['feature', 'correlation_r', 'cohens_d', 'p_value']].head(10).to_string(index=False))

    # Classification experiment
    print("\n[4/5] Running classification experiments...")
    classification = run_classification(df)
    print("\nClassification results:")
    for name, results in classification.items():
        print(f"  {name}: F1={results['test_f1']:.3f}, Accuracy={results['test_accuracy']:.3f}")

    # Error analysis
    print("\n[5/5] Analyzing errors...")
    errors = analyze_errors(df)
    print(f"  False positives: {errors['false_positive_patterns']['count']}")
    print(f"  False negatives: {errors['false_negative_patterns']['count']}")

    # Generate report
    report_path = Path("tasks/reports/dreaddit-validation.md")
    report = generate_report(df, correlations, classification, errors, report_path)
    print(f"\nReport saved to: {report_path}")

    # Print summary
    print("\n" + "=" * 60)
    print("VALIDATION SUMMARY")
    print("=" * 60)

    v1_f1 = classification.get('current_v1', {}).get('test_f1', 0)
    v2_f1 = classification.get('proposed_v2', {}).get('test_f1', 0)

    print(f"\nCurrent implementation (hedge + question): F1 = {v1_f1:.3f}")
    print(f"Proposed (+ negative_emotion + first_person): F1 = {v2_f1:.3f}")

    if v2_f1 > v1_f1:
        print(f"\n>>> Proposed changes IMPROVE performance by {(v2_f1-v1_f1)/v1_f1*100:.1f}%")
    else:
        print(f"\n>>> Proposed changes show {(v2_f1-v1_f1)/v1_f1*100:.1f}% change")

    print("\nKey recommendations:")
    neg_emotion_sig = correlations[correlations['feature'] == 'negative_emotion_density']['significant'].iloc[0]
    first_person_sig = correlations[correlations['feature'] == 'first_person_ratio']['significant'].iloc[0]

    print(f"  - Add negative emotion words: {'RECOMMENDED' if neg_emotion_sig else 'UNCERTAIN'}")
    print(f"  - Use first-person ratio: {'RECOMMENDED' if first_person_sig else 'UNCERTAIN'}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
