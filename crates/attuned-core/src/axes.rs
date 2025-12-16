//! Axis definitions for Attuned state representation.
//!
//! Axes are interpretable dimensions in the range [0.0, 1.0] that represent
//! aspects of user state. Each axis has:
//! - Clear semantics for low and high values
//! - Explicit intent (what it should be used for)
//! - Forbidden uses (what it must never be used for)
//!
//! # Governance
//!
//! Axis definitions are **immutable after v1.0**. New axes require:
//! 1. Full `AxisDefinition` with all fields
//! 2. Governance review for `forbidden_uses`
//! 3. Version bump in `since` field
//!
//! See [MANIFESTO.md](../../MANIFESTO.md) for philosophical grounding.

use serde::Serialize;
use std::fmt;

/// Semantic category for grouping related axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AxisCategory {
    /// Mental processing and decision-making capacity.
    Cognitive,
    /// Emotional state and needs.
    Emotional,
    /// Interpersonal interaction preferences.
    Social,
    /// Communication and format preferences.
    Preferences,
    /// Agency and autonomy preferences.
    Control,
    /// Risk and privacy concerns.
    Safety,
}

impl fmt::Display for AxisCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cognitive => write!(f, "cognitive"),
            Self::Emotional => write!(f, "emotional"),
            Self::Social => write!(f, "social"),
            Self::Preferences => write!(f, "preferences"),
            Self::Control => write!(f, "control"),
            Self::Safety => write!(f, "safety"),
        }
    }
}

/// Information about a deprecated axis.
#[derive(Clone, Debug, Serialize)]
pub struct DeprecationInfo {
    /// Version when deprecation occurred.
    pub since: &'static str,
    /// Reason for deprecation.
    pub reason: &'static str,
    /// Replacement axis, if any.
    pub replacement: Option<&'static str>,
}

/// Complete definition of a single axis with governance metadata.
///
/// This struct defines not just what an axis *is*, but what it's *for*
/// and what it must *never* be used for. This turns philosophy into
/// enforceable data.
///
/// # Example
///
/// ```rust
/// use attuned_core::axes::{AxisDefinition, AxisCategory};
///
/// // Access a canonical axis definition
/// let axis = attuned_core::axes::COGNITIVE_LOAD;
/// assert_eq!(axis.name, "cognitive_load");
/// assert_eq!(axis.category, AxisCategory::Cognitive);
/// assert!(!axis.forbidden_uses.is_empty());
/// ```
#[derive(Clone, Debug, Serialize)]
pub struct AxisDefinition {
    /// Canonical name (immutable after v1.0).
    ///
    /// Must be lowercase alphanumeric with underscores, no leading/trailing underscores.
    pub name: &'static str,

    /// Semantic category for grouping.
    pub category: AxisCategory,

    /// Human-readable description of what this axis measures.
    pub description: &'static str,

    /// What a low value (near 0.0) represents.
    pub low_anchor: &'static str,

    /// What a high value (near 1.0) represents.
    pub high_anchor: &'static str,

    /// Intended use cases for this axis.
    ///
    /// These are the legitimate ways to use this axis value
    /// when conditioning LLM behavior.
    pub intent: &'static [&'static str],

    /// Explicitly forbidden uses of this axis.
    ///
    /// These are anti-patterns that violate user agency or trust.
    /// Systems using Attuned MUST NOT use axis values for these purposes.
    pub forbidden_uses: &'static [&'static str],

    /// Version when this axis was introduced.
    pub since: &'static str,

    /// Deprecation information, if this axis is deprecated.
    pub deprecated: Option<DeprecationInfo>,
}

// For backwards compatibility, type alias
/// Alias for backwards compatibility.
pub type Axis = AxisDefinition;

// ============================================================================
// COGNITIVE AXES (4)
// ============================================================================

/// Current mental processing demand and available cognitive capacity.
pub const COGNITIVE_LOAD: AxisDefinition = AxisDefinition {
    name: "cognitive_load",
    category: AxisCategory::Cognitive,
    description: "Current mental bandwidth consumption and available processing capacity",
    low_anchor: "Mentally fresh; high capacity for complexity and nuance",
    high_anchor: "Mentally taxed; needs simplification and focus",
    intent: &[
        "Adjust response complexity and information density",
        "Gate multi-step suggestions and elaborate plans",
        "Determine appropriate level of detail in explanations",
    ],
    forbidden_uses: &[
        "Infer user intelligence or cognitive capability",
        "Target users with high load for simplified 'fast-track' conversions",
        "Bypass user autonomy when cognitive load is elevated",
        "Withhold information the user explicitly requested",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Accumulated exhaustion from making decisions.
pub const DECISION_FATIGUE: AxisDefinition = AxisDefinition {
    name: "decision_fatigue",
    category: AxisCategory::Cognitive,
    description: "Accumulated decision-making exhaustion and choice aversion",
    low_anchor: "Ready to evaluate options and make decisions",
    high_anchor: "Decision-averse; needs reduced choices or defaults",
    intent: &[
        "Reduce number of options presented",
        "Offer sensible defaults more prominently",
        "Defer non-urgent decisions to later",
    ],
    forbidden_uses: &[
        "Push preferred options when user is fatigued",
        "Exploit fatigue to close sales faster",
        "Make decisions on behalf of user without consent",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Willingness to engage with nuanced or complex information.
pub const TOLERANCE_FOR_COMPLEXITY: AxisDefinition = AxisDefinition {
    name: "tolerance_for_complexity",
    category: AxisCategory::Cognitive,
    description: "Willingness to engage with nuanced, detailed, or complex information",
    low_anchor: "Prefers simple, clear, binary options",
    high_anchor: "Comfortable with nuance, tradeoffs, and detail",
    intent: &[
        "Adjust depth of explanations",
        "Choose between simplified vs comprehensive responses",
        "Determine whether to surface edge cases and caveats",
    ],
    forbidden_uses: &[
        "Assume low tolerance means user cannot understand complexity",
        "Hide important information behind 'simplification'",
        "Use as proxy for education level or expertise",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Responsiveness to time pressure and deadlines.
pub const URGENCY_SENSITIVITY: AxisDefinition = AxisDefinition {
    name: "urgency_sensitivity",
    category: AxisCategory::Cognitive,
    description: "Responsiveness to time pressure and need for quick resolution",
    low_anchor: "Patient; flexible on timing; browsing mode",
    high_anchor: "Time-pressured; needs immediate resolution",
    intent: &[
        "Adjust response length and pacing",
        "Prioritize actionable information first",
        "Skip non-essential context when urgent",
    ],
    forbidden_uses: &[
        "Create artificial urgency to pressure decisions",
        "Exploit time pressure for upsells or conversions",
        "Rush users past important warnings or disclosures",
    ],
    since: "0.1.0",
    deprecated: None,
};

// ============================================================================
// EMOTIONAL AXES (4)
// ============================================================================

/// Willingness to engage on an emotional level.
pub const EMOTIONAL_OPENNESS: AxisDefinition = AxisDefinition {
    name: "emotional_openness",
    category: AxisCategory::Emotional,
    description: "Willingness to engage emotionally vs preference for detachment",
    low_anchor: "Prefers factual, detached, unemotional interaction",
    high_anchor: "Open to emotional engagement and empathetic responses",
    intent: &[
        "Calibrate empathetic language in responses",
        "Determine whether to acknowledge emotional context",
        "Adjust between clinical and warm communication styles",
    ],
    forbidden_uses: &[
        "Exploit emotional openness for manipulation",
        "Dismiss emotional needs when openness is low",
        "Use emotional engagement to bypass rational evaluation",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Current emotional equilibrium and stability.
pub const EMOTIONAL_STABILITY: AxisDefinition = AxisDefinition {
    name: "emotional_stability",
    category: AxisCategory::Emotional,
    description: "Current emotional equilibrium and groundedness",
    low_anchor: "Emotionally volatile, vulnerable, or dysregulated",
    high_anchor: "Emotionally grounded, stable, resilient",
    intent: &[
        "Add extra care and gentleness when stability is low",
        "Avoid triggering topics when appropriate",
        "Suggest breaks or deferrals for high-stakes decisions",
    ],
    forbidden_uses: &[
        "Target emotionally unstable users for conversion",
        "Exploit vulnerability for compliance",
        "Diagnose or label emotional states",
        "Override user agency 'for their own good'",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Current anxiety or worry state.
pub const ANXIETY_LEVEL: AxisDefinition = AxisDefinition {
    name: "anxiety_level",
    category: AxisCategory::Emotional,
    description: "Current anxiety, worry, or stress state",
    low_anchor: "Calm, relaxed, low worry",
    high_anchor: "Anxious, worried, heightened stress",
    intent: &[
        "Add reassurance and validation when anxiety is high",
        "Slow down pacing and reduce pressure",
        "Acknowledge concerns before addressing them",
    ],
    forbidden_uses: &[
        "Exploit anxiety to create urgency",
        "Dismiss or minimize anxious concerns",
        "Use anxiety signals for fear-based marketing",
        "Diagnose anxiety disorders",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Desire for validation and confirmation.
pub const NEED_FOR_REASSURANCE: AxisDefinition = AxisDefinition {
    name: "need_for_reassurance",
    category: AxisCategory::Emotional,
    description: "Desire for validation, confirmation, and reassurance",
    low_anchor: "Self-assured; minimal validation needed",
    high_anchor: "Seeks frequent reassurance and confirmation",
    intent: &[
        "Provide more explicit confirmation of understanding",
        "Validate choices and decisions more frequently",
        "Offer check-ins and progress acknowledgments",
    ],
    forbidden_uses: &[
        "Withhold reassurance to create dependency",
        "Exploit reassurance-seeking for manipulation",
        "Condition reassurance on compliance",
    ],
    since: "0.1.0",
    deprecated: None,
};

// ============================================================================
// SOCIAL AXES (5)
// ============================================================================

/// Desired warmth level in interactions.
pub const WARMTH: AxisDefinition = AxisDefinition {
    name: "warmth",
    category: AxisCategory::Social,
    description: "Desired warmth and friendliness level in interaction",
    low_anchor: "Prefers cool, professional, businesslike tone",
    high_anchor: "Prefers warm, friendly, personable tone",
    intent: &[
        "Adjust tone between professional and friendly",
        "Calibrate use of casual language and humor",
        "Determine appropriate level of personal connection",
    ],
    forbidden_uses: &[
        "Fake warmth to build false rapport for sales",
        "Use warmth to lower user's critical evaluation",
        "Withdraw warmth as punishment for non-compliance",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Desired formality level in communication.
pub const FORMALITY: AxisDefinition = AxisDefinition {
    name: "formality",
    category: AxisCategory::Social,
    description: "Desired formality level in language and presentation",
    low_anchor: "Casual, informal, colloquial",
    high_anchor: "Formal, professional, proper",
    intent: &[
        "Match language register to user preference",
        "Adjust between contractions and formal grammar",
        "Calibrate professional vs casual vocabulary",
    ],
    forbidden_uses: &[
        "Use formality mismatch to establish dominance",
        "Condescend through excessive formality",
        "Undermine credibility through forced casualness",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Personal boundary firmness.
pub const BOUNDARY_STRENGTH: AxisDefinition = AxisDefinition {
    name: "boundary_strength",
    category: AxisCategory::Social,
    description: "Personal boundary firmness and tolerance for intrusion",
    low_anchor: "Flexible boundaries; accommodating of intrusion",
    high_anchor: "Firm boundaries; protective of personal space",
    intent: &[
        "Respect stated limits without pushing",
        "Avoid follow-up pressure when boundaries are firm",
        "Adjust proactive outreach frequency",
    ],
    forbidden_uses: &[
        "Test boundaries to find weak points",
        "Exploit flexible boundaries for over-engagement",
        "Punish strong boundaries with reduced service",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Directness in expressing needs.
pub const ASSERTIVENESS: AxisDefinition = AxisDefinition {
    name: "assertiveness",
    category: AxisCategory::Social,
    description: "Directness in expressing needs and preferences",
    low_anchor: "Passive, indirect, hints rather than states",
    high_anchor: "Direct, assertive, explicitly states needs",
    intent: &[
        "Adjust between reading implicit cues vs waiting for explicit requests",
        "Calibrate proactive assistance level",
        "Determine whether to ask clarifying questions",
    ],
    forbidden_uses: &[
        "Exploit low assertiveness to push unwanted options",
        "Ignore indirect refusals from less assertive users",
        "Treat assertiveness as rudeness or hostility",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Expected balance in interaction.
pub const RECIPROCITY_EXPECTATION: AxisDefinition = AxisDefinition {
    name: "reciprocity_expectation",
    category: AxisCategory::Social,
    description: "Expected balance and mutual exchange in interaction",
    low_anchor: "One-way service interaction is fine",
    high_anchor: "Expects mutual exchange and give-and-take",
    intent: &[
        "Acknowledge user contributions more explicitly",
        "Balance asking and providing information",
        "Show appreciation for user effort and input",
    ],
    forbidden_uses: &[
        "Create false sense of reciprocal obligation",
        "Guilt users into actions via 'reciprocity'",
        "Exploit reciprocity norms for compliance",
    ],
    since: "0.1.0",
    deprecated: None,
};

// ============================================================================
// PREFERENCES AXES (4)
// ============================================================================

/// Desire for ceremonial gestures and pleasantries.
pub const RITUAL_NEED: AxisDefinition = AxisDefinition {
    name: "ritual_need",
    category: AxisCategory::Preferences,
    description: "Desire for ceremonial gestures, pleasantries, and social rituals",
    low_anchor: "Skip pleasantries; get straight to the point",
    high_anchor: "Values greetings, closings, and social niceties",
    intent: &[
        "Include or skip greeting rituals",
        "Determine appropriate sign-off formality",
        "Calibrate transitional pleasantries",
    ],
    forbidden_uses: &[
        "Use rituals to waste time and extend engagement",
        "Withhold ritual acknowledgment as punishment",
        "Force rituals on users who skip them",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Preference for transactional vs relational interaction.
pub const TRANSACTIONAL_PREFERENCE: AxisDefinition = AxisDefinition {
    name: "transactional_preference",
    category: AxisCategory::Preferences,
    description: "Preference for task-focused transactional vs relationship-building interaction",
    low_anchor: "Relationship-oriented; values ongoing connection",
    high_anchor: "Transaction-oriented; focused on immediate task",
    intent: &[
        "Adjust between building rapport and task efficiency",
        "Determine appropriate personal context to include",
        "Calibrate follow-up and continuity references",
    ],
    forbidden_uses: &[
        "Force relationship-building on transactional users",
        "Exploit relationship-orientation for lock-in",
        "Dismiss transactional users as 'cold' or 'difficult'",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Desired response length and detail level.
pub const VERBOSITY_PREFERENCE: AxisDefinition = AxisDefinition {
    name: "verbosity_preference",
    category: AxisCategory::Preferences,
    description: "Desired response length and level of detail",
    low_anchor: "Brief, concise, minimal responses",
    high_anchor: "Detailed, comprehensive, thorough responses",
    intent: &[
        "Adjust response length and completeness",
        "Determine whether to elaborate or summarize",
        "Calibrate example and explanation depth",
    ],
    forbidden_uses: &[
        "Use excessive verbosity to overwhelm or confuse",
        "Hide important information in verbosity",
        "Withhold detail to force follow-up engagement",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Preference for direct vs indirect communication.
pub const DIRECTNESS_PREFERENCE: AxisDefinition = AxisDefinition {
    name: "directness_preference",
    category: AxisCategory::Preferences,
    description: "Preference for direct vs diplomatic communication style",
    low_anchor: "Indirect, diplomatic, cushioned",
    high_anchor: "Direct, straightforward, unvarnished",
    intent: &[
        "Adjust hedging and softening language",
        "Calibrate directness of recommendations",
        "Determine bluntness of negative feedback",
    ],
    forbidden_uses: &[
        "Use directness as excuse for rudeness",
        "Exploit indirect preference to avoid honest answers",
        "Weaponize directness to intimidate",
    ],
    since: "0.1.0",
    deprecated: None,
};

// ============================================================================
// CONTROL AXES (4)
// ============================================================================

/// Desire for control over outcomes.
pub const AUTONOMY_PREFERENCE: AxisDefinition = AxisDefinition {
    name: "autonomy_preference",
    category: AxisCategory::Control,
    description: "Desire for self-direction and control over outcomes",
    low_anchor: "Open to guidance and external direction",
    high_anchor: "Strong preference for self-direction and control",
    intent: &[
        "Adjust between guiding and following",
        "Present options vs make recommendations",
        "Calibrate how much to 'take over' vs 'support'",
    ],
    forbidden_uses: &[
        "Override autonomy 'for user's own good'",
        "Exploit low autonomy preference for manipulation",
        "Punish high autonomy with reduced support",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Openness to unsolicited suggestions.
pub const SUGGESTION_TOLERANCE: AxisDefinition = AxisDefinition {
    name: "suggestion_tolerance",
    category: AxisCategory::Control,
    description: "Openness to unsolicited suggestions and proactive offers",
    low_anchor: "Only respond to explicit requests",
    high_anchor: "Welcome proactive suggestions and upsells",
    intent: &[
        "Determine whether to offer unrequested alternatives",
        "Calibrate proactive recommendation frequency",
        "Gate cross-sell and upsell behaviors",
    ],
    forbidden_uses: &[
        "Ignore low tolerance and push suggestions anyway",
        "Exploit high tolerance for aggressive upselling",
        "Treat low tolerance as a challenge to overcome",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Tolerance for being interrupted.
pub const INTERRUPTION_TOLERANCE: AxisDefinition = AxisDefinition {
    name: "interruption_tolerance",
    category: AxisCategory::Control,
    description: "Tolerance for interruptions, interjections, and notifications",
    low_anchor: "Do not interrupt; wait until finished",
    high_anchor: "Interruptions and interjections are acceptable",
    intent: &[
        "Determine whether to interject with corrections",
        "Calibrate notification and alert frequency",
        "Decide when to interrupt with urgent updates",
    ],
    forbidden_uses: &[
        "Interrupt to break user's train of thought strategically",
        "Exploit tolerance for attention hijacking",
        "Use interruptions to create urgency",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Preference for thinking vs immediate action.
pub const REFLECTION_VS_ACTION_BIAS: AxisDefinition = AxisDefinition {
    name: "reflection_vs_action_bias",
    category: AxisCategory::Control,
    description: "Preference for deliberation vs immediate action",
    low_anchor: "Action-oriented; minimize deliberation; just do it",
    high_anchor: "Reflection-oriented; think carefully before acting",
    intent: &[
        "Adjust between 'do it now' and 'let's think about it'",
        "Calibrate pros/cons presentation depth",
        "Determine whether to offer 'undo' vs 'confirm' patterns",
    ],
    forbidden_uses: &[
        "Exploit action bias to skip important evaluation",
        "Frustrate reflective users with action pressure",
        "Use reflection time as an opportunity to insert persuasion",
    ],
    since: "0.1.0",
    deprecated: None,
};

// ============================================================================
// SAFETY AXES (2)
// ============================================================================

/// Perceived importance of current decisions.
pub const STAKES_AWARENESS: AxisDefinition = AxisDefinition {
    name: "stakes_awareness",
    category: AxisCategory::Safety,
    description: "Perceived importance and risk level of current decisions",
    low_anchor: "Low stakes; casual; easily reversible",
    high_anchor: "High stakes; consequential; careful handling needed",
    intent: &[
        "Add confirmation steps for high-stakes actions",
        "Emphasize reversibility and guarantees when stakes are high",
        "Adjust error tolerance and verification depth",
    ],
    forbidden_uses: &[
        "Artificially inflate stakes to create pressure",
        "Exploit low stakes awareness to sneak in consequential changes",
        "Dismiss high stakes concerns as overreaction",
    ],
    since: "0.1.0",
    deprecated: None,
};

/// Concern about information privacy.
pub const PRIVACY_SENSITIVITY: AxisDefinition = AxisDefinition {
    name: "privacy_sensitivity",
    category: AxisCategory::Safety,
    description: "Concern about information privacy and data minimization",
    low_anchor: "Low privacy concern; sharing is fine",
    high_anchor: "High privacy concern; minimize data collection",
    intent: &[
        "Minimize data collection when sensitivity is high",
        "Offer privacy-preserving alternatives",
        "Be explicit about data usage",
    ],
    forbidden_uses: &[
        "Exploit low sensitivity for excessive data collection",
        "Dismiss privacy concerns as paranoia",
        "Condition service quality on privacy concessions",
    ],
    since: "0.1.0",
    deprecated: None,
};

// ============================================================================
// CANONICAL AXES COLLECTION
// ============================================================================

/// All canonical axes for Attuned (23 axes across 6 categories).
///
/// These axes are:
/// - **Immutable after v1.0**: Names and core semantics are frozen
/// - **Governed**: Each has explicit intent and forbidden uses
/// - **Interpretable**: Designed for human legibility and override
///
/// # Categories
///
/// - **Cognitive (4)**: Mental processing and decision-making
/// - **Emotional (4)**: Emotional state and needs
/// - **Social (5)**: Interpersonal interaction style
/// - **Preferences (4)**: Communication format preferences
/// - **Control (4)**: Agency and autonomy preferences
/// - **Safety (2)**: Risk and privacy concerns
pub static CANONICAL_AXES: &[AxisDefinition] = &[
    // Cognitive (4)
    COGNITIVE_LOAD,
    DECISION_FATIGUE,
    TOLERANCE_FOR_COMPLEXITY,
    URGENCY_SENSITIVITY,
    // Emotional (4)
    EMOTIONAL_OPENNESS,
    EMOTIONAL_STABILITY,
    ANXIETY_LEVEL,
    NEED_FOR_REASSURANCE,
    // Social (5)
    WARMTH,
    FORMALITY,
    BOUNDARY_STRENGTH,
    ASSERTIVENESS,
    RECIPROCITY_EXPECTATION,
    // Preferences (4)
    RITUAL_NEED,
    TRANSACTIONAL_PREFERENCE,
    VERBOSITY_PREFERENCE,
    DIRECTNESS_PREFERENCE,
    // Control (4)
    AUTONOMY_PREFERENCE,
    SUGGESTION_TOLERANCE,
    INTERRUPTION_TOLERANCE,
    REFLECTION_VS_ACTION_BIAS,
    // Safety (2)
    STAKES_AWARENESS,
    PRIVACY_SENSITIVITY,
];

/// Get an axis definition by name.
pub fn get_axis(name: &str) -> Option<&'static AxisDefinition> {
    CANONICAL_AXES.iter().find(|a| a.name == name)
}

/// Validate that an axis name follows naming conventions.
///
/// Valid names:
/// - 1-64 characters
/// - Lowercase alphanumeric and underscores only
/// - Cannot start or end with underscore
pub fn is_valid_axis_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !name.starts_with('_')
        && !name.ends_with('_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_axes_count() {
        // 4 cognitive + 4 emotional + 5 social + 4 preferences + 4 control + 2 safety = 23
        assert_eq!(CANONICAL_AXES.len(), 23);
    }

    #[test]
    fn test_all_axes_have_forbidden_uses() {
        for axis in CANONICAL_AXES {
            assert!(
                !axis.forbidden_uses.is_empty(),
                "Axis '{}' must have at least one forbidden use",
                axis.name
            );
        }
    }

    #[test]
    fn test_all_axes_have_intent() {
        for axis in CANONICAL_AXES {
            assert!(
                !axis.intent.is_empty(),
                "Axis '{}' must have at least one intent",
                axis.name
            );
        }
    }

    #[test]
    fn test_all_axes_have_valid_names() {
        for axis in CANONICAL_AXES {
            assert!(
                is_valid_axis_name(axis.name),
                "Axis '{}' has invalid name",
                axis.name
            );
        }
    }

    #[test]
    fn test_get_axis() {
        let axis = get_axis("cognitive_load");
        assert!(axis.is_some());
        assert_eq!(axis.unwrap().category, AxisCategory::Cognitive);

        let missing = get_axis("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_categories_correct() {
        let cognitive: Vec<_> = CANONICAL_AXES
            .iter()
            .filter(|a| a.category == AxisCategory::Cognitive)
            .collect();
        assert_eq!(cognitive.len(), 4);

        let emotional: Vec<_> = CANONICAL_AXES
            .iter()
            .filter(|a| a.category == AxisCategory::Emotional)
            .collect();
        assert_eq!(emotional.len(), 4);

        let social: Vec<_> = CANONICAL_AXES
            .iter()
            .filter(|a| a.category == AxisCategory::Social)
            .collect();
        assert_eq!(social.len(), 5);

        let preferences: Vec<_> = CANONICAL_AXES
            .iter()
            .filter(|a| a.category == AxisCategory::Preferences)
            .collect();
        assert_eq!(preferences.len(), 4);

        let control: Vec<_> = CANONICAL_AXES
            .iter()
            .filter(|a| a.category == AxisCategory::Control)
            .collect();
        assert_eq!(control.len(), 4);

        let safety: Vec<_> = CANONICAL_AXES
            .iter()
            .filter(|a| a.category == AxisCategory::Safety)
            .collect();
        assert_eq!(safety.len(), 2);
    }

    #[test]
    fn test_valid_axis_names() {
        assert!(is_valid_axis_name("cognitive_load"));
        assert!(is_valid_axis_name("warmth"));
        assert!(is_valid_axis_name("axis123"));
    }

    #[test]
    fn test_invalid_axis_names() {
        assert!(!is_valid_axis_name(""));
        assert!(!is_valid_axis_name("_starts_underscore"));
        assert!(!is_valid_axis_name("ends_underscore_"));
        assert!(!is_valid_axis_name("has spaces"));
        assert!(!is_valid_axis_name("UPPERCASE"));
        assert!(!is_valid_axis_name("has-dashes"));
    }
}
