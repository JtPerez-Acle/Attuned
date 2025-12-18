# Attuned

**Declare human state. Get appropriate AI behavior.**

Attuned is the behavioral layer for LLM applications. Set user context, get conditioned responses. Works with any LLM.

```bash
pip install attuned
```

## Quick Start

```python
from attuned import Attuned

# Declare user state - set what you need, rest defaults to neutral
state = Attuned(
    verbosity_preference=0.2,  # Brief responses
    warmth=0.9,                # Warm and friendly
)

# Get prompt context - works with ANY LLM
system_prompt = f"You are an assistant.\n\n{state.prompt()}"

# Use with OpenAI
response = openai.chat.completions.create(
    model="gpt-4o-mini",
    messages=[
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": "How do I learn Python?"}
    ]
)

# Or Anthropic
response = anthropic.messages.create(
    model="claude-3-5-sonnet",
    system=system_prompt,
    messages=[{"role": "user", "content": "How do I learn Python?"}]
)

# Or Ollama, Mistral, Gemini, or any LLM that accepts a system prompt
```

## Why Attuned?

Without Attuned:
```python
# Hand-crafted prompts. Does "be concise" work? Who knows.
system = "You are a helpful assistant. Be concise. Be friendly."
```

With Attuned:
```python
# Statistically validated. 68% shorter responses with brief. Proven.
state = Attuned(verbosity_preference=0.2, warmth=0.9)
system = f"You are a helpful assistant.\n\n{state.prompt()}"
```

**Validation results:**
- `verbosity_preference=0.2` → 68% shorter responses (p<0.0001, d=2.5)
- `warmth=0.9` → 5x more warm language (p<0.0001, d=1.3)
- `cognitive_load=0.9` → 81% fewer multi-step plans (p<0.0001, d=2.0)

## Axes (23 Available)

Set any axes you care about. Unset axes default to 0.5 (neutral, no effect).

| Category | Axes |
|----------|------|
| Cognitive | `cognitive_load`, `decision_fatigue`, `tolerance_for_complexity`, `urgency_sensitivity` |
| Emotional | `emotional_openness`, `emotional_stability`, `anxiety_level`, `need_for_reassurance` |
| Social | `warmth`, `formality`, `boundary_strength`, `assertiveness`, `reciprocity_expectation` |
| Preferences | `ritual_need`, `transactional_preference`, `verbosity_preference`, `directness_preference` |
| Control | `autonomy_preference`, `suggestion_tolerance`, `interruption_tolerance`, `reflection_vs_action_bias` |
| Safety | `stakes_awareness`, `privacy_sensitivity` |

## Presets

Common patterns out of the box:

```python
from attuned import Attuned

# Anxious user - warm, reassuring, not overwhelming
state = Attuned.presets.anxious_user()

# Busy executive - brief, formal, direct
state = Attuned.presets.busy_executive()

# Learning student - detailed, patient, educational
state = Attuned.presets.learning_student()

# Casual chat - warm, casual, balanced
state = Attuned.presets.casual_chat()

# High stakes - careful, thorough, formal
state = Attuned.presets.high_stakes()

# Overwhelmed - minimal, supportive, no pressure
state = Attuned.presets.overwhelmed()
```

## Integrations (Optional)

Thin wrappers for less boilerplate. The core `state.prompt()` works with anything.

### OpenAI
```python
from attuned import Attuned
from attuned.integrations.openai import AttunedOpenAI

state = Attuned(verbosity_preference=0.2, warmth=0.9)
client = AttunedOpenAI(state=state)
response = client.chat("How do I learn Python?")
```

### Anthropic
```python
from attuned import Attuned
from attuned.integrations.anthropic import AttunedAnthropic

state = Attuned(verbosity_preference=0.2, warmth=0.9)
client = AttunedAnthropic(state=state)
response = client.message("How do I learn Python?")
```

### LiteLLM (100+ providers)
```python
from attuned import Attuned
from attuned.integrations.litellm import AttunedLiteLLM

state = Attuned(verbosity_preference=0.2, warmth=0.9)
client = AttunedLiteLLM(state=state)

# Same code, any provider
response = client.chat("gpt-4o-mini", "Hello")
response = client.chat("claude-3-sonnet-20240229", "Hello")
response = client.chat("ollama/llama2", "Hello")
response = client.chat("gemini/gemini-pro", "Hello")
```

## What Attuned Produces

When you call `state.prompt()`, you get text like this:

```
## Interaction Guidelines
- Offer suggestions, not actions
- Drafts require explicit user approval
- Silence is acceptable if no action is required
- Use warm, friendly language. Include encouraging phrases like 'Great question!'
- Keep responses brief and to the point.

Tone: warm-casual
Verbosity: brief
```

This is injected into the LLM's system prompt. That's it. No magic. Just validated prompt engineering.

## Advanced Usage

For full control, use the underlying types:

```python
from attuned import StateSnapshot, RuleTranslator, Source

snapshot = StateSnapshot.builder() \
    .user_id("user_123") \
    .source(Source.SelfReport) \
    .axis("warmth", 0.7) \
    .axis("cognitive_load", 0.9) \
    .build()

translator = RuleTranslator()
context = translator.to_prompt_context(snapshot)
print(context.format_for_prompt())
```

## HTTP Client (Server Mode)

For distributed deployments:

```python
from attuned import AttunedClient, StateSnapshot

client = AttunedClient("http://localhost:8080")
client.upsert_state(snapshot)
context = client.get_context("user_123")
```

## Governance

Every axis has governance metadata:

```python
from attuned import get_axis

axis = get_axis("cognitive_load")
print(axis.intent)         # What this axis is FOR
print(axis.forbidden_uses) # What it must NEVER be used for
```

Attuned is designed to respect users:
- Never optimizes for engagement/conversion
- Never executes actions (produces context, not commands)
- Self-report always overrides inference
- Full transparency into translation rules

## License

MIT
