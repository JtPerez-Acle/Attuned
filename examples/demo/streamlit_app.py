#!/usr/bin/env python3
"""
Attuned Split-Screen Demo

"Your LLM can't tell when users are frustrated. Now it can."

A split-screen interactive demo that makes Attuned's value proposition
undeniable in 30 seconds.
"""

import streamlit as st
from typing import Optional
import re

# Must be first Streamlit command
st.set_page_config(
    page_title="Attuned Demo",
    page_icon="🎯",
    layout="wide",
    initial_sidebar_state="collapsed",
)


# =============================================================================
# AXIS INFERENCE (Heuristic MVP)
# =============================================================================

def infer_axes_from_message(message: str) -> dict:
    """
    Infer axes from user message using simple heuristics.

    This is a demo-grade implementation. The real inference engine
    in attuned-infer uses validated NLP features.
    """
    text = message.lower()
    words = text.split()

    # Phrase-based detection (check these first - they're strong signals)
    high_anxiety_phrases = [
        'losing my mind', 'going crazy', 'freaking out', 'can\'t take it',
        'at my wit\'s end', 'pulling my hair out', 'so frustrated',
        'don\'t know what to do', 'completely lost', 'no idea what',
    ]
    phrase_anxiety = sum(0.25 for p in high_anxiety_phrases if p in text)

    # Word-based anxiety detection
    anxiety_words = [
        'worried', 'anxious', 'stressed', 'frustrated', 'help',
        'panic', 'overwhelmed', 'scared', 'nervous', 'confused', 'stuck'
    ]
    word_anxiety = 0.12 * sum(1 for w in anxiety_words if w in text)

    anxiety = min(0.95, 0.25 + phrase_anxiety + word_anxiety)

    # Urgency detection
    urgency_phrases = ['right now', 'right away', 'as soon as possible']
    urgency_words = ['urgent', 'asap', 'quickly', 'immediately', 'deadline', 'hurry', 'fast', 'critical']
    urgency = min(0.95, 0.25 + 0.2 * sum(1 for p in urgency_phrases if p in text) +
                  0.15 * sum(1 for w in urgency_words if w in text))

    # Frustration markers (escalate anxiety)
    frustration_markers = [
        'hours', 'days', 'tried everything', 'nothing works', 'so tired',
        'give up', 'hate', 'terrible', 'awful', 'ridiculous', 'been trying'
    ]
    frustration = sum(1 for w in frustration_markers if w in text)
    if frustration > 0:
        anxiety = min(0.95, anxiety + 0.12 * frustration)

    # Cognitive load (longer messages with complex structure suggest overwhelm)
    word_count = len(words)
    has_multiple_topics = text.count(' and ') + text.count(',') > 2
    cognitive = min(0.95, 0.35 + 0.012 * word_count + (0.2 if has_multiple_topics else 0))
    # High anxiety also increases cognitive load
    if anxiety > 0.6:
        cognitive = min(0.95, cognitive + 0.15)

    # If user seems distressed, they want brevity
    verbosity = 0.2 if anxiety > 0.6 or frustration > 0 else 0.5

    # Warmth defaults high when distressed
    warmth = 0.8 if anxiety > 0.5 else 0.5

    # Formality: casual language = low formality
    casual_markers = ['hey', 'hi', 'yo', 'gonna', 'wanna', 'kinda', 'lol', 'haha']
    formality = 0.3 if any(m in text for m in casual_markers) else 0.5

    return {
        'anxiety_level': round(anxiety, 2),
        'cognitive_load': round(cognitive, 2),
        'verbosity_preference': round(verbosity, 2),
        'warmth': round(warmth, 2),
        'formality': round(formality, 2),
        'urgency_sensitivity': round(urgency, 2),
    }


# =============================================================================
# LLM INTEGRATION
# =============================================================================

def call_llm(
    api_key: str,
    model: str,
    system_prompt: str,
    user_message: str,
) -> tuple[str, int]:
    """
    Call an LLM via LiteLLM. Returns (response_text, token_count).
    """
    try:
        import litellm
        litellm.api_key = api_key

        # Set API key based on model
        if 'gpt' in model or 'o1' in model:
            import os
            os.environ['OPENAI_API_KEY'] = api_key
        elif 'claude' in model:
            import os
            os.environ['ANTHROPIC_API_KEY'] = api_key

        response = litellm.completion(
            model=model,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_message},
            ],
            max_tokens=500,
            temperature=0.7,
        )

        text = response.choices[0].message.content
        tokens = response.usage.completion_tokens if response.usage else len(text.split())
        return text, tokens

    except Exception as e:
        return f"Error: {str(e)}", 0


def generate_responses(
    api_key: str,
    model: str,
    base_prompt: str,
    user_message: str,
    axes: dict,
) -> tuple[tuple[str, int], tuple[str, int]]:
    """Generate both vanilla and Attuned responses."""

    # Import attuned
    try:
        from attuned import Attuned
    except ImportError:
        return (
            ("Error: attuned package not installed. Run: pip install -e crates/attuned-python", 0),
            ("Error: attuned package not installed. Run: pip install -e crates/attuned-python", 0),
        )

    # Vanilla response
    vanilla_prompt = base_prompt if base_prompt else "You are a helpful assistant."
    vanilla_response = call_llm(api_key, model, vanilla_prompt, user_message)

    # Attuned response
    state = Attuned(**axes)
    attuned_prompt = f"{vanilla_prompt}\n\n{state.prompt()}"
    attuned_response = call_llm(api_key, model, attuned_prompt, user_message)

    return vanilla_response, attuned_response


# =============================================================================
# CODE SNIPPET GENERATOR
# =============================================================================

def generate_code_snippet(axes: dict) -> str:
    """Generate copy-paste integration code."""
    # Filter to non-default axes
    non_default = {k: v for k, v in axes.items() if abs(v - 0.5) > 0.05}

    if not non_default:
        return '''from attuned import Attuned

state = Attuned()  # Default neutral state
system = f"{your_prompt}\\n\\n{state.prompt()}"'''

    axes_str = ",\n    ".join(f"{k}={v}" for k, v in non_default.items())

    return f'''from attuned import Attuned

state = Attuned(
    {axes_str},
)

system = f"{{your_prompt}}\\n\\n{{state.prompt()}}"'''


# =============================================================================
# SAMPLE PROMPTS
# =============================================================================

SAMPLE_PROMPTS = {
    "Frustrated User": "I've been trying to figure this out for 2 hours and I'm losing my mind",
    "Anxious Decision": "I'm not sure if I should accept this job offer, there are so many factors to consider and I keep going back and forth",
    "Overwhelmed Student": "I have three exams next week and I haven't started studying for any of them, I don't even know where to begin",
    "Busy Executive": "Give me the bottom line on Q3 performance",
    "Curious Learner": "I'd love to understand how neural networks actually work, from the ground up",
}


# =============================================================================
# MAIN APP
# =============================================================================

def main():
    # Header
    st.markdown("""
    <h1 style='text-align: center; margin-bottom: 0.5rem;'>
        The Problem With Every AI Assistant
    </h1>
    """, unsafe_allow_html=True)

    # The problem - make them feel it
    st.markdown("""
    <div style='background: #fff3e0; border-left: 4px solid #ff9800; padding: 1rem 1.5rem; margin: 1rem 0;'>
        <p style='margin: 0; font-size: 1.1em;'>
            <strong>You:</strong> <em>"I've been trying to figure this out for 2 hours and I'm losing my mind"</em>
        </p>
        <p style='margin: 0.5rem 0 0 0; font-size: 1.1em;'>
            <strong>ChatGPT:</strong> "Great question! Here's a comprehensive guide. First, let's understand the fundamentals..."
            <span style='color: #666;'>[proceeds to write 400 words]</span>
        </p>
    </div>
    """, unsafe_allow_html=True)

    st.markdown("""
    **The LLM has no idea you're frustrated.** It responds the same way whether you're relaxed and curious,
    or stressed and desperate for a quick answer. It can't read the room.

    This is why AI assistants feel robotic. They're not dumb - they just can't see *you*.
    """)

    # The solution - show, don't tell
    st.markdown("### What if the LLM could read the room?")

    # Side-by-side example - the "aha moment" before any user action
    example_col1, example_col2 = st.columns(2)

    with example_col1:
        st.markdown("""
        <div style='background: #ffebee; border: 2px solid #ef5350; border-radius: 8px; padding: 1rem; height: 100%;'>
            <p style='margin: 0 0 0.5rem 0; font-weight: bold; color: #c62828;'>Without Attuned:</p>
            <p style='margin: 0; font-size: 0.95em; color: #333;'>
                "Great question! Let me walk you through this step by step.
                <br><br>
                First, you'll want to understand the core concepts. The key thing to know is that...
                <br><br>
                Step 1: Navigate to your settings panel and locate the configuration section...
                <br><br>
                Step 2: Once you've found that, you'll need to..."
            </p>
            <p style='margin: 0.5rem 0 0 0; color: #999; font-size: 0.85em;'>~350 tokens, ignores emotional state</p>
        </div>
        """, unsafe_allow_html=True)

    with example_col2:
        st.markdown("""
        <div style='background: #e8f5e9; border: 2px solid #66bb6a; border-radius: 8px; padding: 1rem; height: 100%;'>
            <p style='margin: 0 0 0.5rem 0; font-weight: bold; color: #2e7d32;'>With Attuned:</p>
            <p style='margin: 0; font-size: 0.95em; color: #333;'>
                "That sounds exhausting. Let's get this fixed quickly.
                <br><br>
                What are you seeing on your screen right now? Just describe it briefly and I'll give you the exact fix."
            </p>
            <p style='margin: 0.5rem 0 0 0; color: #999; font-size: 0.85em;'>~45 tokens, acknowledges frustration, gets to the point</p>
        </div>
        """, unsafe_allow_html=True)

    st.markdown("""
    <p style='text-align: center; margin-top: 1rem; color: #666;'>
        <strong>Same model. Same system prompt. The only difference is Attuned.</strong>
    </p>
    """, unsafe_allow_html=True)

    # Now the invitation to try
    st.markdown("""
    <div style='background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                padding: 1.5rem; border-radius: 10px; margin: 1rem 0; color: white; text-align: center;'>
        <h3 style='margin-top: 0; color: white;'>Try it yourself</h3>
        <p style='margin-bottom: 0;'>
            Enter your API key below and send any message.
            Watch how Attuned detects your state and adapts the response.
        </p>
    </div>
    """, unsafe_allow_html=True)

    st.markdown("---")

    # Configuration row
    col_key, col_model = st.columns(2)

    with col_key:
        api_key = st.text_input(
            "Your API Key",
            type="password",
            placeholder="sk-... (OpenAI) or sk-ant-... (Anthropic)",
            help="Your API key. Used only for this session - never stored or logged.",
        )

    with col_model:
        model = st.selectbox(
            "Model",
            options=[
                "gpt-4o-mini",
                "gpt-4o",
                "gpt-4-turbo",
                "claude-3-5-sonnet-20241022",
                "claude-3-5-haiku-20241022",
            ],
            index=0,
            help="Pick any model - Attuned works with all of them.",
        )

    # Check if API key is provided
    if not api_key:
        st.warning("Enter your API key above to try the demo. Your key is used only for this session.")

    # Optional: custom system prompt
    with st.expander("Advanced: Custom System Prompt"):
        base_prompt = st.text_area(
            "Base system prompt (same for both sides)",
            value="You are a helpful assistant.",
            height=80,
        )

    st.markdown("---")

    # Sample prompts with better labeling
    st.markdown("**Quick start - click a scenario:**")
    sample_cols = st.columns(len(SAMPLE_PROMPTS))
    for i, (name, prompt) in enumerate(SAMPLE_PROMPTS.items()):
        with sample_cols[i]:
            if st.button(name, use_container_width=True, help=prompt[:50] + "..."):
                st.session_state.user_message = prompt

    # User input
    user_message = st.text_area(
        "Or type your own message:",
        value=st.session_state.get('user_message', ''),
        height=100,
        placeholder="Try something emotional: frustrated, anxious, overwhelmed, or just busy...",
    )

    # Initialize session state for axes
    if 'axes' not in st.session_state:
        st.session_state.axes = {
            'anxiety_level': 0.5,
            'cognitive_load': 0.5,
            'verbosity_preference': 0.5,
            'warmth': 0.5,
            'formality': 0.5,
            'urgency_sensitivity': 0.5,
        }

    # Detect axes from message
    if user_message:
        detected = infer_axes_from_message(user_message)
        # Update session state (can be overridden by sliders)
        if 'last_message' not in st.session_state or st.session_state.last_message != user_message:
            st.session_state.axes = detected
            st.session_state.last_message = user_message

    # Generate button
    col_btn, col_space = st.columns([1, 3])
    with col_btn:
        generate_clicked = st.button("Generate Responses", type="primary", use_container_width=True)

    st.markdown("---")

    # Split screen responses
    col_vanilla, col_attuned = st.columns(2)

    with col_vanilla:
        st.markdown("""
        <div style='background: #ffebee; padding: 0.5rem 1rem; border-radius: 5px; margin-bottom: 0.5rem;'>
            <strong style='color: #c62828;'>WITHOUT ATTUNED</strong>
            <span style='color: #666; font-size: 0.9em;'> - No state awareness</span>
        </div>
        """, unsafe_allow_html=True)
        vanilla_container = st.container()

    with col_attuned:
        st.markdown("""
        <div style='background: #e8f5e9; padding: 0.5rem 1rem; border-radius: 5px; margin-bottom: 0.5rem;'>
            <strong style='color: #2e7d32;'>WITH ATTUNED</strong>
            <span style='color: #666; font-size: 0.9em;'> - Adapts to your state</span>
        </div>
        """, unsafe_allow_html=True)
        attuned_container = st.container()

    # Generate responses if button clicked
    if generate_clicked and api_key and user_message:
        with st.spinner("Sending to LLM twice: once vanilla, once with Attuned context..."):
            (vanilla_text, vanilla_tokens), (attuned_text, attuned_tokens) = generate_responses(
                api_key, model, base_prompt, user_message, st.session_state.axes
            )
            st.session_state.vanilla_response = (vanilla_text, vanilla_tokens)
            st.session_state.attuned_response = (attuned_text, attuned_tokens)

    # Display responses
    with vanilla_container:
        if 'vanilla_response' in st.session_state:
            text, tokens = st.session_state.vanilla_response
            st.markdown(text)
            st.caption(f"~{tokens} tokens")
        else:
            st.markdown("*Response will appear here...*")

    with attuned_container:
        if 'attuned_response' in st.session_state:
            text, tokens = st.session_state.attuned_response
            st.markdown(text)
            st.caption(f"~{tokens} tokens")
        else:
            st.markdown("*Response will appear here...*")

    st.markdown("---")

    # Axis controls
    st.markdown("### How Attuned reads your message")
    st.markdown("""
    These axes were **detected from your message**. Attuned uses these to tell the LLM
    how to respond. Drag the sliders to experiment - then click "Generate Responses" again.
    """)

    # Axis sliders in columns
    axis_col1, axis_col2, axis_col3 = st.columns(3)

    with axis_col1:
        st.session_state.axes['anxiety_level'] = st.slider(
            "Anxiety Level",
            0.0, 1.0, st.session_state.axes['anxiety_level'],
            help="How anxious the user seems (0=calm, 1=very anxious)"
        )
        st.session_state.axes['cognitive_load'] = st.slider(
            "Cognitive Load",
            0.0, 1.0, st.session_state.axes['cognitive_load'],
            help="How overwhelmed the user is (0=relaxed, 1=overloaded)"
        )

    with axis_col2:
        st.session_state.axes['verbosity_preference'] = st.slider(
            "Verbosity Preference",
            0.0, 1.0, st.session_state.axes['verbosity_preference'],
            help="How much detail to provide (0=brief, 1=detailed)"
        )
        st.session_state.axes['warmth'] = st.slider(
            "Warmth",
            0.0, 1.0, st.session_state.axes['warmth'],
            help="Emotional warmth (0=neutral, 1=very warm)"
        )

    with axis_col3:
        st.session_state.axes['formality'] = st.slider(
            "Formality",
            0.0, 1.0, st.session_state.axes['formality'],
            help="Communication style (0=casual, 1=formal)"
        )
        st.session_state.axes['urgency_sensitivity'] = st.slider(
            "Urgency Sensitivity",
            0.0, 1.0, st.session_state.axes['urgency_sensitivity'],
            help="How urgent the situation is (0=relaxed, 1=urgent)"
        )

    # Reset button
    if st.button("Reset to Detected Values"):
        if user_message:
            st.session_state.axes = infer_axes_from_message(user_message)
            st.rerun()

    st.markdown("---")

    # Code snippet
    st.markdown("### Add this to your project")
    st.markdown("""
    This is all you need. Attuned works with **any LLM** - just inject `state.prompt()` into your system prompt.
    """)

    code = generate_code_snippet(st.session_state.axes)
    st.code(code, language="python")

    st.markdown("""
    ```python
    # That's it. Your LLM now adapts to user state.
    response = openai.chat.completions.create(
        model="gpt-4o-mini",
        messages=[
            {"role": "system", "content": system},  # Includes Attuned context
            {"role": "user", "content": user_message}
        ]
    )
    ```
    """)

    # How it works
    st.markdown("---")
    st.markdown("### How it works")

    st.markdown("""
    <div style='display: flex; align-items: center; gap: 1rem; margin: 1rem 0; flex-wrap: wrap;'>
        <div style='background: #f5f5f5; padding: 1rem; border-radius: 8px; flex: 1; min-width: 150px; text-align: center;'>
            <div style='font-size: 1.5em;'>💬</div>
            <strong>User Message</strong>
            <p style='margin: 0.5rem 0 0 0; font-size: 0.85em; color: #666;'>"I'm losing my mind"</p>
        </div>
        <div style='font-size: 1.5em; color: #999;'>→</div>
        <div style='background: #e3f2fd; padding: 1rem; border-radius: 8px; flex: 1; min-width: 150px; text-align: center;'>
            <div style='font-size: 1.5em;'>📊</div>
            <strong>Detect State</strong>
            <p style='margin: 0.5rem 0 0 0; font-size: 0.85em; color: #666;'>anxiety: 0.8<br>cognitive_load: 0.7</p>
        </div>
        <div style='font-size: 1.5em; color: #999;'>→</div>
        <div style='background: #f3e5f5; padding: 1rem; border-radius: 8px; flex: 1; min-width: 150px; text-align: center;'>
            <div style='font-size: 1.5em;'>📝</div>
            <strong>Generate Guidelines</strong>
            <p style='margin: 0.5rem 0 0 0; font-size: 0.85em; color: #666;'>"Be brief, warm,<br>acknowledge feelings"</p>
        </div>
        <div style='font-size: 1.5em; color: #999;'>→</div>
        <div style='background: #e8f5e9; padding: 1rem; border-radius: 8px; flex: 1; min-width: 150px; text-align: center;'>
            <div style='font-size: 1.5em;'>🤖</div>
            <strong>LLM Responds</strong>
            <p style='margin: 0.5rem 0 0 0; font-size: 0.85em; color: #666;'>Adapted to<br>user's state</p>
        </div>
    </div>
    """, unsafe_allow_html=True)

    st.markdown("""
    **That's the whole trick.** Attuned doesn't change the LLM or require fine-tuning.
    It just tells the LLM what it can't see - the human's emotional and cognitive state -
    so it can respond appropriately.

    - **23 axes** capture human state (anxiety, cognitive load, verbosity preference, etc.)
    - **Works with any LLM** - OpenAI, Anthropic, Ollama, or your own
    - **No data stored** - state is computed per-request, nothing is logged
    - **You control it** - set axes manually or let Attuned detect them
    """)

    # Footer
    st.markdown("---")
    st.markdown("""
    <div style='text-align: center; color: #888;'>
        <p><strong>Attuned</strong> - Context, not control.</p>
        <p><code>pip install attuned</code> &nbsp;|&nbsp; <a href="https://github.com/JtPerez-Acle/Attuned">GitHub</a></p>
    </div>
    """, unsafe_allow_html=True)


if __name__ == "__main__":
    main()
