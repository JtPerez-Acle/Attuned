"""
Attuned - Universal LLM behavioral layer.

Declare human state. Get appropriate AI behavior.

Quick Start:
    >>> from attuned import Attuned
    >>>
    >>> # Set any axes you care about (others default to neutral)
    >>> state = Attuned(
    ...     verbosity_preference=0.2,  # Brief responses
    ...     warmth=0.9,                # Warm and friendly
    ...     cognitive_load=0.8,        # User is overwhelmed
    ... )
    >>>
    >>> # Get the prompt context - works with ANY LLM
    >>> print(state.prompt())
    >>>
    >>> # Use with OpenAI, Anthropic, Ollama, or any LLM
    >>> response = openai.chat.completions.create(
    ...     model="gpt-4o-mini",
    ...     messages=[
    ...         {"role": "system", "content": f"You are an assistant.\\n\\n{state.prompt()}"},
    ...         {"role": "user", "content": user_message}
    ...     ]
    ... )
"""

from typing import Optional, Dict, Any
from attuned._attuned import (
    StateSnapshot,
    Source,
    RuleTranslator,
    PromptContext,
    get_all_axes,
)


# All 23 canonical axes
AXIS_NAMES = [axis.name for axis in get_all_axes()]


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

    Example:
        >>> state = Attuned(verbosity_preference=0.2, warmth=0.9)
        >>> print(state.prompt())
        ## Interaction Guidelines
        - Keep responses brief...
        - Use warm, friendly language...

        Tone: warm-casual
        Verbosity: brief
    """

    def __init__(self, user_id: str = "default", **axes: float):
        self._user_id = user_id
        self._axes: Dict[str, float] = {}
        self._translator = RuleTranslator()
        self._context: Optional[PromptContext] = None

        # Validate and store axes
        for name, value in axes.items():
            if name not in AXIS_NAMES:
                valid_axes = ", ".join(AXIS_NAMES[:5]) + "..."
                raise ValueError(
                    f"Unknown axis: '{name}'. Valid axes include: {valid_axes}. "
                    f"See Attuned.axes() for full list."
                )
            if not 0.0 <= value <= 1.0:
                raise ValueError(f"Axis value must be between 0.0 and 1.0, got {value} for '{name}'")
            self._axes[name] = value

    def prompt(self) -> str:
        """
        Get the prompt context as a string.

        This is the core output - inject this into any LLM's system prompt.

        Returns:
            Formatted prompt context ready for injection.

        Example:
            >>> state = Attuned(verbosity_preference=0.2)
            >>> system_prompt = f"You are an assistant.\\n\\n{state.prompt()}"
        """
        if self._context is None:
            self._context = self._build_context()
        return self._context.format_for_prompt()

    def context(self) -> PromptContext:
        """
        Get the structured PromptContext object.

        Use this if you need programmatic access to guidelines, tone, etc.

        Returns:
            PromptContext with guidelines, tone, verbosity, and flags.
        """
        if self._context is None:
            self._context = self._build_context()
        return self._context

    def _build_context(self) -> PromptContext:
        """Build the prompt context from current axes."""
        builder = StateSnapshot.builder().user_id(self._user_id).source(Source.SelfReport)
        for name, value in self._axes.items():
            builder = builder.axis(name, value)
        snapshot = builder.build()
        return self._translator.to_prompt_context(snapshot)

    @staticmethod
    def axes() -> list:
        """
        Get all available axis names.

        Returns:
            List of all 23 canonical axis names.
        """
        return AXIS_NAMES.copy()

    @staticmethod
    def axis_info(name: str) -> dict:
        """
        Get detailed information about an axis.

        Args:
            name: The axis name.

        Returns:
            Dictionary with description, category, intent, and forbidden_uses.
        """
        from attuned._attuned import get_axis
        axis = get_axis(name)
        return {
            "name": axis.name,
            "description": axis.description,
            "category": str(axis.category),
            "intent": axis.intent,
            "forbidden_uses": axis.forbidden_uses,
        }

    def __repr__(self) -> str:
        axes_str = ", ".join(f"{k}={v:.2f}" for k, v in self._axes.items())
        return f"Attuned({axes_str})" if axes_str else "Attuned()"


# =============================================================================
# PRESETS - Common patterns out of the box
# =============================================================================

class Presets:
    """
    Pre-configured Attuned states for common scenarios.

    Example:
        >>> state = Attuned.presets.anxious_user()
        >>> print(state.prompt())
    """

    @staticmethod
    def anxious_user() -> Attuned:
        """User feeling anxious - warm, reassuring, not overwhelming."""
        return Attuned(
            anxiety_level=0.9,
            warmth=0.9,
            cognitive_load=0.8,
            verbosity_preference=0.3,
            urgency_sensitivity=0.2,
        )

    @staticmethod
    def busy_executive() -> Attuned:
        """Time-pressed professional - brief, formal, direct."""
        return Attuned(
            verbosity_preference=0.1,
            formality=0.9,
            urgency_sensitivity=0.9,
            cognitive_load=0.9,
            decision_fatigue=0.8,
        )

    @staticmethod
    def learning_student() -> Attuned:
        """Someone learning - detailed, patient, educational."""
        return Attuned(
            verbosity_preference=0.9,
            warmth=0.7,
            tolerance_for_complexity=0.7,  # Can handle detailed explanations
            cognitive_load=0.3,  # Not overwhelmed, has mental bandwidth
        )

    @staticmethod
    def casual_chat() -> Attuned:
        """Relaxed conversation - warm, casual, balanced."""
        return Attuned(
            warmth=0.9,
            formality=0.1,
            ritual_need=0.2,
        )

    @staticmethod
    def high_stakes() -> Attuned:
        """Important decision - careful, thorough, formal."""
        return Attuned(
            stakes_awareness=0.95,
            formality=0.8,
            verbosity_preference=0.7,
            decision_fatigue=0.7,
        )

    @staticmethod
    def overwhelmed() -> Attuned:
        """User is overwhelmed - minimal, supportive, no pressure."""
        return Attuned(
            cognitive_load=0.95,
            anxiety_level=0.8,
            verbosity_preference=0.1,
            decision_fatigue=0.9,
            warmth=0.8,
            suggestion_tolerance=0.2,
        )

    @staticmethod
    def privacy_conscious() -> Attuned:
        """User values privacy - minimal data, respectful."""
        return Attuned(
            privacy_sensitivity=0.95,
            boundary_strength=0.8,
            formality=0.6,
        )


# Attach presets to Attuned class
Attuned.presets = Presets
