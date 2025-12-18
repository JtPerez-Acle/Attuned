"""
LiteLLM integration for Attuned.

LiteLLM provides a unified interface to 100+ LLM providers.
This integration gives you Attuned conditioning across all of them.

Supported providers include:
    - OpenAI (gpt-4, gpt-3.5-turbo, etc.)
    - Anthropic (claude-3-opus, claude-3-sonnet, etc.)
    - Google (gemini-pro, palm, etc.)
    - Azure OpenAI
    - AWS Bedrock
    - Ollama (local models)
    - Mistral
    - Cohere
    - And 90+ more...

Example:
    >>> from attuned import Attuned
    >>> from attuned.integrations.litellm import AttunedLiteLLM
    >>>
    >>> state = Attuned(verbosity_preference=0.2, warmth=0.9)
    >>> client = AttunedLiteLLM(state=state)
    >>>
    >>> # Same code works with any provider
    >>> response = client.chat("gpt-4o-mini", "How do I learn Python?")
    >>> response = client.chat("claude-3-sonnet-20240229", "How do I learn Python?")
    >>> response = client.chat("ollama/llama2", "How do I learn Python?")
    >>> response = client.chat("gemini/gemini-pro", "How do I learn Python?")
"""

from typing import Optional, List, Dict, Any, TYPE_CHECKING

if TYPE_CHECKING:
    from attuned.core import Attuned


class AttunedLiteLLM:
    """
    Universal LLM client with Attuned behavioral conditioning.

    Uses LiteLLM to provide access to 100+ LLM providers through a single interface.

    Args:
        state: Default Attuned state for all requests.
        base_system_prompt: Base system prompt to prepend (default: "You are a helpful assistant.")
    """

    def __init__(
        self,
        state: Optional["Attuned"] = None,
        base_system_prompt: str = "You are a helpful assistant.",
    ):
        self._default_state = state
        self._base_system_prompt = base_system_prompt

        try:
            import litellm
            self._litellm = litellm
        except ImportError:
            raise ImportError(
                "LiteLLM package not installed. Install with: pip install litellm"
            )

    def chat(
        self,
        model: str,
        message: str,
        state: Optional["Attuned"] = None,
        **kwargs,
    ) -> str:
        """
        Send a chat message to any supported LLM.

        Args:
            model: The model identifier (e.g., "gpt-4o-mini", "claude-3-sonnet-20240229", "ollama/llama2")
            message: The user message.
            state: Override the default state for this request.
            **kwargs: Additional arguments passed to LiteLLM.

        Returns:
            The assistant's response text.

        Model naming conventions:
            - OpenAI: "gpt-4", "gpt-4o-mini", "gpt-3.5-turbo"
            - Anthropic: "claude-3-opus-20240229", "claude-3-sonnet-20240229"
            - Google: "gemini/gemini-pro"
            - Ollama: "ollama/llama2", "ollama/mistral"
            - Azure: "azure/<deployment_name>"
            - AWS Bedrock: "bedrock/anthropic.claude-v2"
        """
        effective_state = state or self._default_state

        system_prompt = self._base_system_prompt
        if effective_state is not None:
            system_prompt = f"{self._base_system_prompt}\n\n{effective_state.prompt()}"

        response = self._litellm.completion(
            model=model,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": message},
            ],
            **kwargs,
        )
        return response.choices[0].message.content

    def chat_messages(
        self,
        model: str,
        messages: List[Dict[str, str]],
        state: Optional["Attuned"] = None,
        **kwargs,
    ) -> str:
        """
        Send multiple messages to any supported LLM.

        Args:
            model: The model identifier.
            messages: List of message dicts with "role" and "content".
            state: Override the default state for this request.
            **kwargs: Additional arguments passed to LiteLLM.

        Returns:
            The assistant's response text.
        """
        effective_state = state or self._default_state

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

        response = self._litellm.completion(
            model=model,
            messages=processed_messages,
            **kwargs,
        )
        return response.choices[0].message.content

    @staticmethod
    def supported_models() -> List[str]:
        """
        Get a sample of supported model identifiers.

        This is not exhaustive - LiteLLM supports 100+ providers.
        See https://docs.litellm.ai/docs/providers for full list.
        """
        return [
            # OpenAI
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-3.5-turbo",
            # Anthropic
            "claude-3-5-sonnet-20241022",
            "claude-3-opus-20240229",
            "claude-3-sonnet-20240229",
            "claude-3-haiku-20240307",
            # Google
            "gemini/gemini-pro",
            "gemini/gemini-1.5-pro",
            # Ollama (local)
            "ollama/llama2",
            "ollama/mistral",
            "ollama/codellama",
            # Mistral
            "mistral/mistral-large-latest",
            "mistral/mistral-medium",
            # Cohere
            "cohere/command-r-plus",
            "cohere/command-r",
        ]
