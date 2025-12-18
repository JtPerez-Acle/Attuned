# Dreaddit Validation Report

Generated: 2025-12-16 19:53

## Dataset Statistics

- **Total posts**: 3,553
- **Stressed**: 1,857 (52.3%)
- **Not stressed**: 1,696 (47.7%)
- **Train/Test split**: 2,838 / 715

## Feature Correlations with Stress Label

| Feature | Correlation (r) | Effect Size (d) | p-value | Significant |
|---------|-----------------|-----------------|---------|-------------|
| lex_liwc_negemo | 0.395 | 0.861 | 5.45e-133 | Yes |
| lex_liwc_i | 0.387 | 0.841 | 1.34e-127 | Yes |
| first_person_ratio | 0.321 | 0.678 | 6.74e-86 | Yes |
| uncertainty_score_v2 | 0.309 | 0.651 | 1.39e-79 | Yes |
| lex_liwc_anx | 0.271 | 0.564 | 6.23e-61 | Yes |
| negative_emotion_density | 0.266 | 0.552 | 1.33e-58 | Yes |
| emotional_intensity | -0.119 | -0.240 | 9.81e-13 | Yes |
| word_count | 0.111 | 0.223 | 3.93e-11 | Yes |
| exclamation_ratio | -0.110 | -0.221 | 5.16e-11 | Yes |
| uncertainty_score | 0.103 | 0.208 | 6.56e-10 | Yes |
| lex_liwc_Exclam | -0.087 | -0.175 | 1.87e-07 | Yes |
| hedge_density | 0.075 | 0.151 | 6.73e-06 | Yes |
| absolutist_density | 0.068 | 0.136 | 5.32e-05 | Yes |
| question_ratio | 0.066 | 0.133 | 7.89e-05 | Yes |
| lex_liwc_QMark | 0.045 | 0.091 | 0.0070 | Yes |
| formality_score | -0.042 | -0.085 | 0.0117 | Yes |
| lex_liwc_certain | 0.033 | 0.066 | 0.0500 | Yes |
| caps_ratio | -0.027 | -0.054 | 0.1095 | No |
| lex_liwc_tentat | -0.022 | -0.044 | 0.1886 | No |
| complexity_score | -0.011 | -0.022 | 0.5170 | No |

### Key Findings

- **lex_liwc_negemo**: Stressed posts have higher values (r=0.395, d=0.861)
- **lex_liwc_i**: Stressed posts have higher values (r=0.387, d=0.841)
- **first_person_ratio**: Stressed posts have higher values (r=0.321, d=0.678)
- **uncertainty_score_v2**: Stressed posts have higher values (r=0.309, d=0.651)
- **lex_liwc_anx**: Stressed posts have higher values (r=0.271, d=0.564)

## Classification Results

| Model | Precision | Recall | F1 | Accuracy | CV F1 |
|-------|-----------|--------|-----|----------|-------|
| current_v1 | 0.576 | 0.588 | 0.582 | 0.564 | 0.583 |
| proposed_v2 | 0.662 | 0.696 | 0.679 | 0.660 | 0.683 |
| all_features | 0.653 | 0.724 | 0.686 | 0.659 | 0.703 |
| liwc_comparison | 0.722 | 0.759 | 0.740 | 0.724 | 0.738 |
| random_baseline | 0.500 | 0.500 | 0.500 | 0.500 | - |
| majority_baseline | 0.516 | 1.000 | 0.681 | 0.516 | - |

### Feature Importance (proposed_v2 model)

| Feature | Coefficient |
|---------|-------------|
| first_person_ratio | 0.718 |
| negative_emotion_density | 0.696 |
| question_ratio | 0.274 |
| hedge_density | 0.083 |

## Error Analysis

### False Positives (predicted stressed, actually not)

- Count: 116
- Avg hedge density: 0.151
- Avg question ratio: 0.086
- Avg negative emotion: 0.169
- Avg first person ratio: 0.114

### False Negatives (predicted not stressed, actually stressed)

- Count: 127
- Avg hedge density: 0.124
- Avg question ratio: 0.043
- Avg negative emotion: 0.031
- Avg first person ratio: 0.075

## Recommendations

Based on this validation:

1. **ADD negative emotion words**: Proposed v2 improves F1 by 16.7% over current implementation
2. **Negative emotion density** shows significant correlation (r=0.266) - confirms DEEP_RESEARCH.md recommendation
3. **First-person ratio** shows significant correlation (r=0.321) - confirms DEEP_RESEARCH.md recommendation

## References

- Turcan & McKeown (2019). "Dreaddit: A Reddit Dataset for Stress Analysis in Social Media" https://aclanthology.org/D19-6213/
- DEEP_RESEARCH.md - validation evidence summary
- attuned-infer/src/features.rs - current feature implementation
