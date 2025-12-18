"""Type stubs for attuned.integrations.openai."""

from typing import Optional, List, Dict, Any, Callable, TypeVar

from attuned import Attuned

F = TypeVar("F", bound=Callable[..., Any])


class AttunedOpenAI:
    """OpenAI client with Attuned behavioral conditioning."""

    def __init__(
        self,
        state: Optional[Attuned] = None,
        base_system_prompt: str = "You are a helpful assistant.",
        model: str = "gpt-4o-mini",
        client: Optional[Any] = None,
    ) -> None: ...

    def chat(
        self,
        message: str,
        state: Optional[Attuned] = None,
        model: Optional[str] = None,
        **kwargs: Any,
    ) -> str: ...

    def chat_messages(
        self,
        messages: List[Dict[str, str]],
        state: Optional[Attuned] = None,
        model: Optional[str] = None,
        **kwargs: Any,
    ) -> str: ...


def attune(state: Attuned) -> Callable[[F], F]: ...
