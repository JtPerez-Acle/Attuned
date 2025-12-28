"""
Attuned - Declare human state. Get appropriate AI behavior.

The universal behavioral layer for LLM applications.

Quick Start (Simple API):
    >>> from attuned import Attuned
    >>>
    >>> # Set any axes you care about (others default to neutral)
    >>> state = Attuned(
    ...     verbosity_preference=0.2,  # Brief responses
    ...     warmth=0.9,                # Warm and friendly
    ... )
    >>>
    >>> # Get prompt context - works with ANY LLM
    >>> print(state.prompt())
    >>>
    >>> # Use with OpenAI, Anthropic, Ollama, or any LLM
    >>> response = openai.chat.completions.create(
    ...     model="gpt-4o-mini",
    ...     messages=[
    ...         {"role": "system", "content": f"You are an assistant.\\n\\n{state.prompt()}"},
    ...         {"role": "user", "content": "How do I learn Python?"}
    ...     ]
    ... )

With integrations:
    >>> from attuned import Attuned
    >>> from attuned.integrations.openai import AttunedOpenAI
    >>>
    >>> client = AttunedOpenAI(state=Attuned(verbosity_preference=0.2))
    >>> response = client.chat("How do I learn Python?")

With presets:
    >>> from attuned import Attuned
    >>>
    >>> state = Attuned.presets.anxious_user()  # Pre-configured for anxious users
    >>> state = Attuned.presets.busy_executive()  # Brief, formal, direct
    >>> state = Attuned.presets.learning_student()  # Detailed, patient

Advanced (full control):
    >>> from attuned import StateSnapshot, RuleTranslator, Source
    >>>
    >>> snapshot = StateSnapshot.builder() \\
    ...     .user_id("user_123") \\
    ...     .source(Source.SelfReport) \\
    ...     .axis("warmth", 0.7) \\
    ...     .axis("cognitive_load", 0.9) \\
    ...     .build()
    >>>
    >>> translator = RuleTranslator()
    >>> context = translator.to_prompt_context(snapshot)
    >>> print(context.format_for_prompt())

For axis governance information:
    >>> from attuned import get_axis, get_all_axes
    >>>
    >>> axis = get_axis("cognitive_load")
    >>> print(axis.description)
    >>> print(axis.forbidden_uses)  # What this axis must NEVER be used for

Inference from text:
    >>> from attuned import infer, InferenceEngine
    >>>
    >>> # Quick inference
    >>> state = infer("I need this done ASAP!!!")
    >>> print(state.get("urgency_sensitivity").value)  # ~0.7
    >>>
    >>> # With engine (better for repeated inference)
    >>> engine = InferenceEngine()
    >>> state = engine.infer("I'm feeling overwhelmed...")
    >>>
    >>> # Full transparency - see how each estimate was derived
    >>> for estimate in state.all():
    ...     print(f"{estimate.axis}: {estimate.value:.2f}")
    ...     print(f"  Confidence: {estimate.confidence:.2f}")
    ...     print(f"  Source: {estimate.source}")
"""

from attuned._attuned import (
    # Core types
    StateSnapshot,
    StateSnapshotBuilder,
    Source,

    # Translator types
    PromptContext,
    Verbosity,
    RuleTranslator,
    Thresholds,

    # Axis types
    AxisDefinition,
    AxisCategory,

    # Inference types
    InferenceEngine,
    InferredState,
    AxisEstimate,
    InferenceSource,
    LinguisticFeatures,

    # HTTP client
    AttunedClient,

    # Module functions
    get_axis,
    is_valid_axis_name,
    get_axis_names,
    get_all_axes,

    # Inference functions
    infer,
    extract_features,
)

# Simple API - the primary interface
from attuned.core import Attuned, Presets

__version__ = "1.0.0"
__all__ = [
    # Simple API (recommended)
    "Attuned",
    "Presets",

    # Core types (advanced)
    "StateSnapshot",
    "StateSnapshotBuilder",
    "Source",

    # Translator types (advanced)
    "PromptContext",
    "Verbosity",
    "RuleTranslator",
    "Thresholds",

    # Axis types
    "AxisDefinition",
    "AxisCategory",

    # Inference types
    "InferenceEngine",
    "InferredState",
    "AxisEstimate",
    "InferenceSource",
    "LinguisticFeatures",

    # HTTP client
    "AttunedClient",

    # Module functions
    "get_axis",
    "is_valid_axis_name",
    "get_axis_names",
    "get_all_axes",

    # Inference functions
    "infer",
    "extract_features",

    # Constants
    "CANONICAL_AXES",
]

# Convenience constant: all axis definitions
CANONICAL_AXES = get_all_axes()
