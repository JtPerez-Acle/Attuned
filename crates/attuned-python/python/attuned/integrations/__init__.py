"""
Attuned integrations for popular LLM libraries.

These are thin convenience wrappers. The core `Attuned.prompt()` output
works with ANY LLM - these just reduce boilerplate.

Available integrations:
    - openai: OpenAI API wrapper
    - anthropic: Anthropic API wrapper
    - litellm: Universal wrapper (100+ providers)

Example:
    >>> from attuned import Attuned
    >>> from attuned.integrations import openai
    >>>
    >>> state = Attuned(verbosity_preference=0.2, warmth=0.9)
    >>> client = openai.AttunedOpenAI(state=state)
    >>> response = client.chat("How do I learn Python?")
"""

from . import openai
from . import anthropic
from . import litellm

__all__ = ["openai", "anthropic", "litellm"]
