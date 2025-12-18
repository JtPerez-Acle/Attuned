"""
Anthropic integration for Attuned.

This is a thin wrapper that injects Attuned context into Anthropic API calls.
You can always use `state.prompt()` directly if you prefer more control.

Example:
    >>> from attuned import Attuned
    >>> from attuned.integrations.anthropic import AttunedAnthropic
    >>>
    >>> state = Attuned(verbosity_preference=0.2, warmth=0.9)
    >>> client = AttunedAnthropic(state=state)
    >>> response = client.message("How do I learn Python?")
    >>> print(response)
"""

from typing import Optional, List, Dict, Any, TYPE_CHECKING

if TYPE_CHECKING:
    from attuned.core import Attuned


class AttunedAnthropic:
    """
    Anthropic client with Attuned behavioral conditioning.

    Args:
        state: Default Attuned state for all requests.
        base_system_prompt: Base system prompt to prepend (default: "You are a helpful assistant.")
        model: Default model (default: "claude-3-5-sonnet-20241022")
        client: Optional Anthropic client instance. If not provided, creates one.
    """

    def __init__(
        self,
        state: Optional["Attuned"] = None,
        base_system_prompt: str = "You are a helpful assistant.",
        model: str = "claude-3-5-sonnet-20241022",
        client: Optional[Any] = None,
    ):
        self._default_state = state
        self._base_system_prompt = base_system_prompt
        self._model = model

        if client is not None:
            self._client = client
        else:
            try:
                from anthropic import Anthropic
                self._client = Anthropic()
            except ImportError:
                raise ImportError(
                    "Anthropic package not installed. Install with: pip install anthropic"
                )

    def message(
        self,
        message: str,
        state: Optional["Attuned"] = None,
        model: Optional[str] = None,
        max_tokens: int = 1024,
        **kwargs,
    ) -> str:
        """
        Send a message with Attuned conditioning.

        Args:
            message: The user message.
            state: Override the default state for this request.
            model: Override the default model for this request.
            max_tokens: Maximum tokens in response.
            **kwargs: Additional arguments passed to Anthropic API.

        Returns:
            The assistant's response text.
        """
        effective_state = state or self._default_state
        effective_model = model or self._model

        system_prompt = self._base_system_prompt
        if effective_state is not None:
            system_prompt = f"{self._base_system_prompt}\n\n{effective_state.prompt()}"

        response = self._client.messages.create(
            model=effective_model,
            max_tokens=max_tokens,
            system=system_prompt,
            messages=[
                {"role": "user", "content": message},
            ],
            **kwargs,
        )
        return response.content[0].text

    def messages(
        self,
        messages: List[Dict[str, str]],
        state: Optional["Attuned"] = None,
        model: Optional[str] = None,
        max_tokens: int = 1024,
        **kwargs,
    ) -> str:
        """
        Send multiple messages with Attuned conditioning.

        Args:
            messages: List of message dicts with "role" and "content".
            state: Override the default state for this request.
            model: Override the default model for this request.
            max_tokens: Maximum tokens in response.
            **kwargs: Additional arguments passed to Anthropic API.

        Returns:
            The assistant's response text.
        """
        effective_state = state or self._default_state
        effective_model = model or self._model

        system_prompt = self._base_system_prompt
        if effective_state is not None:
            system_prompt = f"{self._base_system_prompt}\n\n{effective_state.prompt()}"

        response = self._client.messages.create(
            model=effective_model,
            max_tokens=max_tokens,
            system=system_prompt,
            messages=messages,
            **kwargs,
        )
        return response.content[0].text
