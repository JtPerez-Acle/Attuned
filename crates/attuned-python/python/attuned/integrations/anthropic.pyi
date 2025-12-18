"""Type stubs for attuned.integrations.anthropic."""

from typing import Optional, List, Dict, Any

from attuned import Attuned


class AttunedAnthropic:
    """Anthropic client with Attuned behavioral conditioning."""

    def __init__(
        self,
        state: Optional[Attuned] = None,
        base_system_prompt: str = "You are a helpful assistant.",
        model: str = "claude-3-5-sonnet-20241022",
        client: Optional[Any] = None,
    ) -> None: ...

    def chat(
        self,
        message: str,
        state: Optional[Attuned] = None,
        model: Optional[str] = None,
        max_tokens: int = 1024,
        **kwargs: Any,
    ) -> str: ...

    def chat_messages(
        self,
        messages: List[Dict[str, str]],
        state: Optional[Attuned] = None,
        model: Optional[str] = None,
        max_tokens: int = 1024,
        **kwargs: Any,
    ) -> str: ...
