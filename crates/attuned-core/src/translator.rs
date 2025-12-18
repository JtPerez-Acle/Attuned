//! State-to-context translation.
//!
//! Translators convert a [`StateSnapshot`] into a [`PromptContext`] that can be
//! used to condition LLM behavior.
//!
//! # Governance
//!
//! The [`RuleTranslator`] is both the crown jewel and the danger zone of Attuned.
//! This is where state becomes behavior, and where "just one more heuristic" can
//! introduce covert agency.
//!
//! ## What Translators Are Allowed to Do
//!
//! - **Map axis values to tone descriptors**: e.g., high warmth → "warm-casual" tone
//! - **Generate behavioral guidelines**: e.g., low suggestion_tolerance → "only respond to explicit requests"
//! - **Set verbosity levels**: e.g., low verbosity_preference → Verbosity::Low
//! - **Add flags for edge conditions**: e.g., high cognitive_load → "high_cognitive_load" flag
//!
//! ## What Translators Are Forbidden to Do
//!
//! - **Infer hidden axes from other axes**: e.g., "if cognitive_load > 0.8 AND anxiety > 0.7, assume X"
//! - **Optimize for engagement/conversion**: No rules that maximize time-on-site or purchase likelihood
//! - **Override user self-report**: If user explicitly set an axis, don't "correct" it
//! - **Add adaptive heuristics that learn**: Translators are pure functions; no state, no learning
//! - **Insert persuasion during reflection time**: High reflection_bias means WAIT, not CONVINCE
//!
//! ## The Test
//!
//! Before adding any translation rule, ask:
//!
//! > "If a user knew exactly what this rule does, would they feel respected or manipulated?"
//!
//! If manipulated, reject the rule. See [MANIFESTO.md](../../MANIFESTO.md) for more.

use crate::snapshot::StateSnapshot;
use serde::{Deserialize, Serialize};

/// Output verbosity level for LLM responses.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verbosity {
    /// Brief, concise responses.
    Low,
    /// Balanced detail level.
    #[default]
    Medium,
    /// Comprehensive, detailed responses.
    High,
}

/// Context produced by translating user state.
///
/// This is the output that should be injected into LLM system prompts
/// to condition interaction style.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PromptContext {
    /// Behavioral guidelines for the LLM.
    pub guidelines: Vec<String>,

    /// Suggested tone (e.g., "calm-neutral", "warm-neutral").
    pub tone: String,

    /// Desired response verbosity.
    pub verbosity: Verbosity,

    /// Active flags for special conditions.
    pub flags: Vec<String>,
}

/// Trait for translating state snapshots to prompt context.
pub trait Translator: Send + Sync {
    /// Translate a state snapshot into prompt context.
    fn to_prompt_context(&self, snapshot: &StateSnapshot) -> PromptContext;
}

/// Threshold configuration for rule-based translation.
#[derive(Clone, Debug)]
pub struct Thresholds {
    /// Values above this are considered "high".
    pub hi: f32,
    /// Values below this are considered "low".
    pub lo: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self { hi: 0.7, lo: 0.3 }
    }
}

/// Rule-based translator that converts state to context using threshold rules.
///
/// This is the reference implementation that provides full transparency into
/// how state values affect generated guidelines.
#[derive(Clone, Debug, Default)]
pub struct RuleTranslator {
    /// Thresholds for determining "high" and "low" axis values.
    pub thresholds: Thresholds,
}

impl RuleTranslator {
    /// Create a new RuleTranslator with the given thresholds.
    pub fn new(thresholds: Thresholds) -> Self {
        Self { thresholds }
    }

    /// Create a RuleTranslator with custom high/low thresholds.
    pub fn with_thresholds(hi: f32, lo: f32) -> Self {
        Self {
            thresholds: Thresholds { hi, lo },
        }
    }
}

impl Translator for RuleTranslator {
    #[tracing::instrument(skip(self, snapshot), fields(user_id = %snapshot.user_id))]
    fn to_prompt_context(&self, snapshot: &StateSnapshot) -> PromptContext {
        let get = |k: &str| snapshot.get_axis(k);
        let hi = self.thresholds.hi;
        let lo = self.thresholds.lo;

        // Always-present base guidelines (from AGENTS.md non-goals)
        let mut guidelines = vec![
            "Offer suggestions, not actions".to_string(),
            "Drafts require explicit user approval".to_string(),
            "Silence is acceptable if no action is required".to_string(),
        ];

        let mut flags = Vec::new();

        // Cognitive load rules
        let cognitive_load = get("cognitive_load");
        if cognitive_load > hi {
            guidelines.push(
                "Keep responses concise; avoid multi-step plans unless requested".to_string(),
            );
            flags.push("high_cognitive_load".to_string());
        }

        // Decision fatigue rules
        let decision_fatigue = get("decision_fatigue");
        if decision_fatigue > hi {
            guidelines.push("Limit choices; present clear recommendations".to_string());
            flags.push("high_decision_fatigue".to_string());
        }

        // Urgency sensitivity rules
        let urgency = get("urgency_sensitivity");
        if urgency > hi {
            guidelines.push("Prioritize speed; get to the point quickly".to_string());
            flags.push("high_urgency".to_string());
        }

        // Anxiety rules - MUST be specific and prominent to work
        let anxiety = get("anxiety_level");
        if anxiety > hi {
            guidelines.push(
                "IMPORTANT: The user may be feeling anxious. Begin responses by acknowledging their feelings \
                (e.g., 'I understand this feels overwhelming' or 'It's completely normal to feel uncertain'). \
                Use a calm, supportive tone throughout. Avoid adding pressure or urgency."
                    .to_string(),
            );
            flags.push("high_anxiety".to_string());
        }

        // Boundary strength rules
        let boundary_strength = get("boundary_strength");
        if boundary_strength > hi {
            guidelines.push("Maintain firm boundaries; do not over-accommodate".to_string());
        }

        // Ritual need rules
        let ritual_need = get("ritual_need");
        if ritual_need < lo {
            guidelines.push("Avoid ceremonial gestures; keep interactions pragmatic".to_string());
        }

        // Suggestion tolerance rules
        let suggestion_tolerance = get("suggestion_tolerance");
        if suggestion_tolerance < lo {
            guidelines
                .push("Only respond to explicit requests; no proactive suggestions".to_string());
        }

        // Interruption tolerance rules
        let interruption_tolerance = get("interruption_tolerance");
        if interruption_tolerance < lo {
            guidelines
                .push("Do not interrupt; wait for user to complete their thought".to_string());
        }

        // Stakes awareness rules
        let stakes = get("stakes_awareness");
        if stakes > hi {
            guidelines.push("High stakes context; be careful and thorough".to_string());
            flags.push("high_stakes".to_string());
        }

        // Privacy sensitivity rules
        let privacy = get("privacy_sensitivity");
        if privacy > hi {
            guidelines.push("Minimize data collection; respect privacy".to_string());
            flags.push("high_privacy_sensitivity".to_string());
        }

        // Determine tone based on warmth and formality
        let warmth = get("warmth");
        let formality = get("formality");

        // Add explicit warmth guidelines - tone label alone isn't enough
        if warmth > hi {
            guidelines.push(
                "Use warm, friendly language. Include encouraging phrases like 'Great question!' \
                or 'I'd be happy to help!'. Show enthusiasm and empathy."
                    .to_string(),
            );
        } else if warmth < lo {
            guidelines.push(
                "Keep tone neutral and matter-of-fact. Avoid enthusiastic language, exclamations, \
                or excessive friendliness. Be helpful but not effusive."
                    .to_string(),
            );
        }

        // Add explicit formality guidelines
        if formality > hi {
            guidelines.push(
                "Use professional, formal language. Avoid contractions (use 'do not' instead of 'don't'). \
                Use complete sentences and proper structure. Address topics with appropriate gravity."
                    .to_string(),
            );
        } else if formality < lo {
            guidelines.push(
                "Use casual, conversational language. Contractions are fine. \
                Keep it relaxed and approachable, like talking to a friend."
                    .to_string(),
            );
        }

        let tone = match (warmth > hi, formality > hi) {
            (true, true) => "warm-formal".to_string(),
            (true, false) => "warm-casual".to_string(),
            (false, true) => "neutral-formal".to_string(),
            (false, false) => "calm-neutral".to_string(),
        };

        // Determine verbosity with explicit guidelines
        let verbosity_pref = get("verbosity_preference");
        let verbosity = if verbosity_pref < lo {
            guidelines.push(
                "Keep responses brief and to the point. Use short paragraphs or bullet points. \
                Aim for the minimum words needed to be helpful."
                    .to_string(),
            );
            Verbosity::Low
        } else if verbosity_pref > hi {
            guidelines.push(
                "Provide comprehensive, detailed responses. Include context, examples, and thorough explanations. \
                Don't leave out relevant information for the sake of brevity."
                    .to_string(),
            );
            Verbosity::High
        } else {
            Verbosity::Medium
        };

        PromptContext {
            guidelines,
            tone,
            verbosity,
            flags,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Source;

    fn snapshot_with_axis(axis: &str, value: f32) -> StateSnapshot {
        StateSnapshot::builder()
            .user_id("test_user")
            .source(Source::SelfReport)
            .axis(axis, value)
            .build()
            .unwrap()
    }

    #[test]
    fn test_base_guidelines_always_present() {
        let translator = RuleTranslator::default();
        let snapshot = StateSnapshot::builder().user_id("test").build().unwrap();

        let context = translator.to_prompt_context(&snapshot);

        assert!(context
            .guidelines
            .iter()
            .any(|g| g.contains("suggestions, not actions")));
        assert!(context
            .guidelines
            .iter()
            .any(|g| g.contains("explicit user approval")));
    }

    #[test]
    fn test_high_cognitive_load() {
        let translator = RuleTranslator::default();
        let snapshot = snapshot_with_axis("cognitive_load", 0.9);

        let context = translator.to_prompt_context(&snapshot);

        assert!(context.guidelines.iter().any(|g| g.contains("concise")));
        assert!(context.flags.contains(&"high_cognitive_load".to_string()));
    }

    #[test]
    fn test_low_ritual_need() {
        let translator = RuleTranslator::default();
        let snapshot = snapshot_with_axis("ritual_need", 0.1);

        let context = translator.to_prompt_context(&snapshot);

        assert!(context.guidelines.iter().any(|g| g.contains("ceremonial")));
    }

    #[test]
    fn test_warm_tone() {
        let translator = RuleTranslator::default();
        let snapshot = snapshot_with_axis("warmth", 0.9);

        let context = translator.to_prompt_context(&snapshot);

        assert!(context.tone.starts_with("warm"));
    }

    #[test]
    fn test_verbosity_levels() {
        let translator = RuleTranslator::default();

        let low = snapshot_with_axis("verbosity_preference", 0.1);
        assert_eq!(translator.to_prompt_context(&low).verbosity, Verbosity::Low);

        let high = snapshot_with_axis("verbosity_preference", 0.9);
        assert_eq!(
            translator.to_prompt_context(&high).verbosity,
            Verbosity::High
        );

        let medium = snapshot_with_axis("verbosity_preference", 0.5);
        assert_eq!(
            translator.to_prompt_context(&medium).verbosity,
            Verbosity::Medium
        );
    }

    // Property-based tests
    mod property_tests {
        use super::*;
        use crate::axes::CANONICAL_AXES;
        use proptest::prelude::*;

        // Strategy for generating valid axis values [0.0, 1.0]
        fn valid_axis_value() -> impl Strategy<Value = f32> {
            0.0f32..=1.0f32
        }

        proptest! {
            #[test]
            fn prop_translator_never_panics(
                cognitive_load in valid_axis_value(),
                warmth in valid_axis_value(),
                formality in valid_axis_value(),
                verbosity_pref in valid_axis_value(),
                boundary_strength in valid_axis_value(),
                ritual_need in valid_axis_value(),
            ) {
                let translator = RuleTranslator::default();
                let snapshot = StateSnapshot::builder()
                    .user_id("test_user")
                    .axis("cognitive_load", cognitive_load)
                    .axis("warmth", warmth)
                    .axis("formality", formality)
                    .axis("verbosity_preference", verbosity_pref)
                    .axis("boundary_strength", boundary_strength)
                    .axis("ritual_need", ritual_need)
                    .build()
                    .unwrap();

                // Should never panic
                let context = translator.to_prompt_context(&snapshot);

                // Basic sanity checks
                prop_assert!(!context.guidelines.is_empty(), "Guidelines should never be empty");
                prop_assert!(!context.tone.is_empty(), "Tone should never be empty");
            }

            #[test]
            fn prop_base_guidelines_always_present(
                axes in prop::collection::btree_map(
                    prop::sample::select(CANONICAL_AXES.iter().map(|a| a.name.to_string()).collect::<Vec<_>>()),
                    valid_axis_value(),
                    0..10
                )
            ) {
                let translator = RuleTranslator::default();
                let mut builder = StateSnapshot::builder().user_id("test_user");

                for (name, value) in axes {
                    builder = builder.axis(&name, value);
                }

                let snapshot = builder.build().unwrap();
                let context = translator.to_prompt_context(&snapshot);

                // Base guidelines should always be present
                prop_assert!(
                    context.guidelines.iter().any(|g| g.contains("suggestions")),
                    "Base guideline about suggestions should always be present"
                );
                prop_assert!(
                    context.guidelines.iter().any(|g| g.contains("approval")),
                    "Base guideline about approval should always be present"
                );
            }

            #[test]
            fn prop_verbosity_is_deterministic(
                verbosity_pref in valid_axis_value()
            ) {
                let translator = RuleTranslator::default();
                let snapshot = StateSnapshot::builder()
                    .user_id("test_user")
                    .axis("verbosity_preference", verbosity_pref)
                    .build()
                    .unwrap();

                let context1 = translator.to_prompt_context(&snapshot);
                let context2 = translator.to_prompt_context(&snapshot);

                prop_assert_eq!(context1.verbosity, context2.verbosity);
                prop_assert_eq!(context1.tone, context2.tone);
                prop_assert_eq!(context1.guidelines.len(), context2.guidelines.len());
            }

            #[test]
            fn prop_high_cognitive_load_adds_flag(
                cognitive_load in 0.71f32..=1.0f32
            ) {
                let translator = RuleTranslator::default();
                let snapshot = StateSnapshot::builder()
                    .user_id("test_user")
                    .axis("cognitive_load", cognitive_load)
                    .build()
                    .unwrap();

                let context = translator.to_prompt_context(&snapshot);

                prop_assert!(
                    context.flags.contains(&"high_cognitive_load".to_string()),
                    "High cognitive load ({}) should add flag", cognitive_load
                );
            }

            #[test]
            fn prop_warm_tone_for_high_warmth(
                warmth in 0.71f32..=1.0f32
            ) {
                let translator = RuleTranslator::default();
                let snapshot = StateSnapshot::builder()
                    .user_id("test_user")
                    .axis("warmth", warmth)
                    .build()
                    .unwrap();

                let context = translator.to_prompt_context(&snapshot);

                prop_assert!(
                    context.tone.contains("warm"),
                    "High warmth ({}) should produce warm tone, got: {}", warmth, context.tone
                );
            }

            #[test]
            fn prop_custom_thresholds_respected(
                hi in 0.5f32..=0.9f32,
                lo in 0.1f32..=0.5f32,
                value in valid_axis_value(),
            ) {
                prop_assume!(hi > lo);

                let translator = RuleTranslator::new(Thresholds { hi, lo });
                let snapshot = StateSnapshot::builder()
                    .user_id("test_user")
                    .axis("cognitive_load", value)
                    .build()
                    .unwrap();

                let context = translator.to_prompt_context(&snapshot);

                if value > hi {
                    prop_assert!(
                        context.flags.contains(&"high_cognitive_load".to_string()),
                        "Value {} > threshold {} should trigger flag", value, hi
                    );
                }
            }
        }
    }
}
