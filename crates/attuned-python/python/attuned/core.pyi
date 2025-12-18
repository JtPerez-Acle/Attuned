"""Type stubs for attuned.core."""

from typing import Optional, List, Dict, Any

from attuned import PromptContext

AXIS_NAMES: List[str]


class Attuned:
    """
    Universal LLM behavioral layer.

    Declare human state using any of the 23 axes. Get a prompt context
    that conditions LLM behavior appropriately.

    Args:
        user_id: Optional user identifier for tracking/storage
        **axes: Any axis values (0.0-1.0). Unset axes default to 0.5 (neutral).

    Axes (23 total):
        Cognitive: cognitive_load, decision_fatigue, tolerance_for_complexity, urgency_sensitivity
        Emotional: emotional_openness, emotional_stability, anxiety_level, need_for_reassurance
        Social: warmth, formality, boundary_strength, assertiveness, reciprocity_expectation
        Preferences: ritual_need, transactional_preference, verbosity_preference, directness_preference
        Control: autonomy_preference, suggestion_tolerance, interruption_tolerance, reflection_vs_action_bias
        Safety: stakes_awareness, privacy_sensitivity
    """

    presets: "Presets"

    def __init__(self, user_id: str = "default", **axes: float) -> None: ...
    def prompt(self) -> str: ...
    def context(self) -> PromptContext: ...

    @staticmethod
    def axes() -> List[str]: ...
    @staticmethod
    def axis_info(name: str) -> Dict[str, Any]: ...

    def __repr__(self) -> str: ...


class Presets:
    """Pre-configured Attuned states for common scenarios."""

    @staticmethod
    def anxious_user() -> Attuned:
        """User feeling anxious - warm, reassuring, not overwhelming."""
        ...

    @staticmethod
    def busy_executive() -> Attuned:
        """Time-pressed professional - brief, formal, direct."""
        ...

    @staticmethod
    def learning_student() -> Attuned:
        """Someone learning - detailed, patient, educational."""
        ...

    @staticmethod
    def casual_chat() -> Attuned:
        """Relaxed conversation - warm, casual, balanced."""
        ...

    @staticmethod
    def high_stakes() -> Attuned:
        """Important decision - careful, thorough, formal."""
        ...

    @staticmethod
    def overwhelmed() -> Attuned:
        """User is overwhelmed - minimal, supportive, no pressure."""
        ...

    @staticmethod
    def privacy_conscious() -> Attuned:
        """User values privacy - minimal data, respectful."""
        ...
