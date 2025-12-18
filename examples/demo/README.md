# Attuned Split-Screen Demo

> "Your LLM can't tell when users are frustrated. Now it can."

An interactive demo that shows the difference between a vanilla LLM and one conditioned with Attuned.

## Quick Start

```bash
# From the project root
cd examples/demo

# Install dependencies (if not already installed)
uv pip install streamlit litellm

# Run the demo
streamlit run streamlit_app.py
```

Or use the run script:

```bash
./run_demo.sh
```

## What This Demo Shows

1. **Type a message** (or click a sample prompt)
2. **See axes detected** in real-time from your message
3. **Compare responses** side-by-side:
   - Left: Vanilla LLM (no state awareness)
   - Right: Attuned LLM (same model, state-aware)
4. **Adjust sliders** to see how different states change behavior
5. **Copy integration code** to add Attuned to your project

## The "Aha Moment"

Try typing: *"I've been trying to figure this out for 2 hours and I'm losing my mind"*

Watch the axes light up, then compare:
- **Vanilla**: 400 tokens of detailed steps
- **Attuned**: "That's exhausting. Let's fix this fast - what are you seeing?"

## Requirements

- Python 3.11+
- Streamlit 1.28+
- LiteLLM 1.0+
- Attuned (installed from this repo)
- API key for OpenAI or Anthropic

## Self-Serve

Bring your own API key - it's used only for the current session and never stored.
