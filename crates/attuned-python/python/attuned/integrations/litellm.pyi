"""Type stubs for attuned.integrations.litellm."""

from typing import Optional, List, Dict, Any

from attuned import Attuned


class AttunedLiteLLM:
    """
    Universal LLM client with Attuned behavioral conditioning.

    Uses LiteLLM to provide access to 100+ LLM providers through a single interface.
    """

    def __init__(
        self,
        state: Optional[Attuned] = None,
        base_system_prompt: str = "You are a helpful assistant.",
    ) -> None: ...

    def chat(
        self,
        model: str,
        message: str,
        state: Optional[Attuned] = None,
        **kwargs: Any,
    ) -> str: ...

    def chat_messages(
        self,
        model: str,
        messages: List[Dict[str, str]],
        state: Optional[Attuned] = None,
        **kwargs: Any,
    ) -> str: ...
