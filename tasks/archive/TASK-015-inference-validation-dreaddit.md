# TASK-015: Validate Inference Against Dreaddit Dataset

## Status
- [ ] Not Started
- [ ] In Progress
- [x] Completed
- [ ] Blocked

**Priority**: High
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: Validation
**Depends On**: attuned-infer crate (completed)
**Blocks**: TASK-016 (refinements should be informed by validation)

## Task Description
Validate our anxiety/stress inference against the Dreaddit dataset - a public Reddit corpus with 3.5K human-labeled posts for stress detection. This establishes ground truth for our feature mappings before making refinements.

## Background
Deep research (DEEP_RESEARCH.md) identified Dreaddit as an ideal validation dataset:
- 3,553 Reddit posts labeled "stressed" vs "not stressed"
- Human annotations with reasonable inter-rater agreement
- Public and citable (Turcan & McKeown, ACL 2019)

Paper: https://aclanthology.org/D19-6213/
Dataset: https://github.com/EdinburghNLP/dreaddit (or via HuggingFace)

## Requirements

### 1. Dataset Acquisition
- Download Dreaddit dataset
- Parse into usable format (text + binary stress label)
- Document data split (train/test if provided)

### 2. Feature Extraction
Run `LinguisticExtractor` on all posts, compute:
- `uncertainty_score()` (our anxiety proxy)
- `hedge_density`
- `question_ratio`
- Individual feature counts

### 3. Correlation Analysis
For each feature, compute:
- Point-biserial correlation with stress label
- Effect size (Cohen's d between stressed/not-stressed groups)
- Statistical significance (p-value)

### 4. Classification Experiment
- Train simple logistic regression on our features
- Compare to:
  - Random baseline
  - Majority class baseline
  - Published Dreaddit baselines (if available)
- Report precision, recall, F1 for stress class

### 5. Error Analysis
- Sample false positives (predicted stressed, actually not)
- Sample false negatives (predicted not stressed, actually stressed)
- Identify patterns: what signals are we missing?

## Expected Outputs

### Validation Report (`tasks/reports/dreaddit-validation.md`)
```markdown
# Dreaddit Validation Report

## Dataset Statistics
- Total posts: X
- Stressed: X (Y%)
- Not stressed: X (Y%)

## Feature Correlations with Stress Label

| Feature | Correlation (r) | Effect Size (d) | p-value |
|---------|-----------------|-----------------|---------|
| uncertainty_score | X.XX | X.XX | <0.001 |
| hedge_density | X.XX | X.XX | X.XXX |
| ...

## Classification Results

| Model | Precision | Recall | F1 | Accuracy |
|-------|-----------|--------|-----|----------|
| Random baseline | 0.50 | 0.50 | 0.50 | 0.50 |
| Our features (LR) | X.XX | X.XX | X.XX | X.XX |
| Published baseline | X.XX | X.XX | X.XX | X.XX |

## Error Analysis
[Patterns identified in misclassifications]

## Recommendations
[Which features to keep, drop, or add based on findings]
```

## Implementation Notes

### Quick Validation Script
Could be Python (faster to prototype) or Rust. Python recommended for:
- Easy dataset loading (pandas, huggingface datasets)
- Statistical libraries (scipy, sklearn)
- Quick iteration

```python
# Pseudocode
from attuned_infer import LinguisticExtractor  # via PyO3 or reimplement

for post in dreaddit:
    features = extract_features(post.text)
    results.append({
        'stress_label': post.label,
        'uncertainty': features.uncertainty_score(),
        'hedge_density': features.hedge_density,
        ...
    })

# Compute correlations
scipy.stats.pointbiserialr(results['uncertainty'], results['stress_label'])
```

### Alternative: Pure Rust
If keeping in Rust:
- Add `csv` crate for dataset parsing
- Use `statrs` crate for statistics
- Create `examples/validate_dreaddit.rs`

## Success Criteria
- [ ] Dataset acquired and parsed
- [ ] All features extracted for all posts
- [ ] Correlation table computed
- [ ] Classification F1 > 0.55 (better than random)
- [ ] Validation report written with recommendations
- [ ] Clear signal on which features work vs don't

## References
- Turcan & McKeown (2019). "Dreaddit: A Reddit Dataset for Stress Analysis in Social Media"
- DEEP_RESEARCH.md - validation evidence summary
- attuned-infer/src/features.rs - current feature implementation
