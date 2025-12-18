"""
OpenAI integration for Attuned.

This is a thin wrapper that injects Attuned context into OpenAI API calls.
You can always use `state.prompt()` directly if you prefer more control.

Example:
    >>> from attuned import Attuned
    >>> from attuned.integrations.openai import AttunedOpenAI
    >>>
    >>> state = Attuned(verbosity_preference=0.2, warmth=0.9)
    >>> client = AttunedOpenAI(state=state)
    >>> response = client.chat("How do I learn Python?")
    >>> print(response)

Or with dynamic state:
    >>> client = AttunedOpenAI()
    >>> response = client.chat(
    ...     "I'm stressed about my deadline",
    ...     state=Attuned.presets.anxious_user()
    ... )
"""

from typing import Optional, List, Dict, Any, TYPE_CHECKING

if TYPE_CHECKING:
    from attuned.core import Attuned


class AttunedOpenAI:
    """
    OpenAI client with Attuned behavioral conditioning.

    Args:
        state: Default Attuned state for all requests.
        base_system_prompt: Base system prompt to prepend (default: "You are a helpful assistant.")
        model: Default model (default: "gpt-4o-mini")
        client: Optional OpenAI client instance. If not provided, creates one.
    """

    def __init__(
        self,
        state: Optional["Attuned"] = None,
        base_system_prompt: str = "You are a helpful assistant.",
        model: str = "gpt-4o-mini",
        client: Optional[Any] = None,
    ):
        self._default_state = state
        self._base_system_prompt = base_system_prompt
        self._model = model

        if client is not None:
            self._client = client
        else:
            try:
                from openai import OpenAI
                self._client = OpenAI()
            except ImportError:
                raise ImportError(
                    "OpenAI package not installed. Install with: pip install openai"
                )

    def chat(
        self,
        message: str,
        state: Optional["Attuned"] = None,
        model: Optional[str] = None,
        **kwargs,
    ) -> str:
        """
        Send a chat message with Attuned conditioning.

        Args:
            message: The user message.
            state: Override the default state for this request.
            model: Override the default model for this request.
            **kwargs: Additional arguments passed to OpenAI API.

        Returns:
            The assistant's response text.
        """
        effective_state = state or self._default_state
        effective_model = model or self._model

        system_prompt = self._base_system_prompt
        if effective_state is not None:
            system_prompt = f"{self._base_system_prompt}\n\n{effective_state.prompt()}"

        response = self._client.chat.completions.create(
            model=effective_model,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": message},
            ],
            **kwargs,
        )
        return response.choices[0].message.content

    def chat_messages(
        self,
        messages: List[Dict[str, str]],
        state: Optional["Attuned"] = None,
        model: Optional[str] = None,
        **kwargs,
    ) -> str:
        """
        Send multiple messages with Attuned conditioning.

        The Attuned context is prepended to the system message.

        Args:
            messages: List of message dicts with "role" and "content".
            state: Override the default state for this request.
            model: Override the default model for this request.
            **kwargs: Additional arguments passed to OpenAI API.

        Returns:
            The assistant's response text.
        """
        effective_state = state or self._default_state
        effective_model = model or self._model

        # Inject Attuned context into system message
        processed_messages = []
        has_system = False

        for msg in messages:
            if msg.get("role") == "system":
                has_system = True
                content = msg["content"]
                if effective_state is not None:
                    content = f"{content}\n\n{effective_state.prompt()}"
                processed_messages.append({"role": "system", "content": content})
            else:
                processed_messages.append(msg)

        # Add system message if none exists
        if not has_system and effective_state is not None:
            processed_messages.insert(0, {
                "role": "system",
                "content": f"{self._base_system_prompt}\n\n{effective_state.prompt()}"
            })

        response = self._client.chat.completions.create(
            model=effective_model,
            messages=processed_messages,
            **kwargs,
        )
        return response.choices[0].message.content


def attune(state: "Attuned"):
    """
    Decorator to add Attuned context to a function that calls OpenAI.

    Example:
        >>> from attuned import Attuned
        >>> from attuned.integrations.openai import attune
        >>>
        >>> @attune(Attuned(verbosity_preference=0.2))
        ... def ask(prompt):
        ...     return openai.chat.completions.create(
        ...         model="gpt-4o-mini",
        ...         messages=[{"role": "user", "content": prompt}]
        ...     )
    """
    def decorator(func):
        def wrapper(*args, **kwargs):
            # Inject state into kwargs for the function to use
            kwargs["_attuned_state"] = state
            kwargs["_attuned_prompt"] = state.prompt()
            return func(*args, **kwargs)
        return wrapper
    return decorator
