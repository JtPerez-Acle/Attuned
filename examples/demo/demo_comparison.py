#!/usr/bin/env python3
"""
Attuned Demo: Before & After Comparison

This demo shows how Attuned transforms LLM interactions by providing
human state context. Same user, same question - different responses.

Requirements:
    - Attuned server running: cargo run -p attuned-cli -- serve
    - OpenAI API key in .env file or OPENAI_API_KEY env var
    - pip install attuned openai python-dotenv
"""

import os
from pathlib import Path
from dotenv import load_dotenv
from openai import OpenAI
from attuned import (
    AttunedClient,
    StateSnapshot,
    Source,
    get_axis,
)

# Load .env from project root
env_path = Path(__file__).parent.parent.parent / ".env"
load_dotenv(env_path)

# Initialize clients
openai_client = OpenAI()
attuned_client = AttunedClient("http://localhost:8080")

# The user's question (same in both scenarios)
USER_QUESTION = "Can you help me organize my project? I have tasks scattered everywhere."

def print_header(title: str):
    """Print a formatted header."""
    print()
    print("=" * 60)
    print(f"  {title}")
    print("=" * 60)
    print()

def call_llm(system_prompt: str, user_message: str) -> str:
    """Call OpenAI with given prompts."""
    response = openai_client.chat.completions.create(
        model="gpt-4o-mini",
        messages=[
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_message},
        ],
        max_tokens=300,
        temperature=0.7,
    )
    return response.choices[0].message.content

def demo_without_attuned():
    """Show response without Attuned context."""
    print_header("WITHOUT ATTUNED")

    system_prompt = """You are a helpful AI assistant. You help users with
productivity and organization tasks. Be thorough and enthusiastic!"""

    print("System prompt:")
    print(f"  \"{system_prompt[:80]}...\"")
    print()
    print(f"User: {USER_QUESTION}")
    print()
    print("Assistant:")
    response = call_llm(system_prompt, USER_QUESTION)
    print(response)
    return response

def demo_with_attuned():
    """Show response with Attuned context."""
    print_header("WITH ATTUNED")

    # Set user state: stressed, overwhelmed, needs brevity
    print("Step 1: Capture user state")
    snapshot = StateSnapshot.builder() \
        .user_id("demo_user") \
        .source(Source.SelfReport) \
        .axis("cognitive_load", 0.95) \
        .axis("anxiety_level", 0.8) \
        .axis("decision_fatigue", 0.85) \
        .axis("warmth", 0.6) \
        .axis("verbosity_preference", 0.15) \
        .axis("urgency_sensitivity", 0.9) \
        .build()

    print(f"  cognitive_load: 0.95 (overwhelmed)")
    print(f"  anxiety_level: 0.8 (stressed)")
    print(f"  decision_fatigue: 0.85 (can't handle choices)")
    print(f"  verbosity_preference: 0.15 (wants brevity)")
    print()

    # Store state and get context
    print("Step 2: Translate state to context")
    attuned_client.upsert_state(snapshot)
    context = attuned_client.get_context("demo_user")

    print(f"  Tone: {context.tone}")
    print(f"  Verbosity: {context.verbosity}")
    print(f"  Flags: {context.flags}")
    print()

    # Build system prompt with Attuned context
    base_prompt = "You are a helpful AI assistant for productivity tasks."
    attuned_section = context.format_for_prompt()

    system_prompt = f"""{base_prompt}

{attuned_section}"""

    print("Step 3: Inject context into system prompt")
    print("  (Attuned guidelines added to base prompt)")
    print()
    print(f"User: {USER_QUESTION}")
    print()
    print("Assistant:")
    response = call_llm(system_prompt, USER_QUESTION)
    print(response)
    return response

def show_governance():
    """Show axis governance information."""
    print_header("GOVERNANCE: What Attuned Refuses To Do")

    axis = get_axis("cognitive_load")
    print(f"Axis: {axis.name}")
    print(f"Description: {axis.description}")
    print()
    print("Intended uses:")
    for use in axis.intent[:2]:
        print(f"  ✓ {use}")
    print()
    print("FORBIDDEN uses:")
    for forbidden in axis.forbidden_uses[:2]:
        print(f"  ✗ {forbidden}")
    print()
    print("See MANIFESTO.md for full philosophy.")

def main():
    print()
    print("╔══════════════════════════════════════════════════════════╗")
    print("║            ATTUNED DEMO: Context, Not Control            ║")
    print("╚══════════════════════════════════════════════════════════╝")

    # Check server
    if not attuned_client.health():
        print("\n❌ Attuned server not running!")
        print("   Start it with: cargo run -p attuned-cli -- serve")
        return

    print("\n✓ Attuned server connected")

    # Run demos
    response_without = demo_without_attuned()
    response_with = demo_with_attuned()

    # Summary
    print_header("THE DIFFERENCE")
    print("Without Attuned:")
    print(f"  - Response length: {len(response_without)} chars")
    print("  - Likely verbose, enthusiastic, many options")
    print()
    print("With Attuned:")
    print(f"  - Response length: {len(response_with)} chars")
    print("  - Concise, calm, single recommendation")
    print()
    print("Same user. Same question. Different context.")
    print("Attuned doesn't control WHAT the AI says - it provides")
    print("context for HOW to say it.")

    # Show governance
    show_governance()

    print()
    print("─" * 60)
    print("  pip install attuned")
    print("  https://github.com/attuned-dev/attuned")
    print("─" * 60)
    print()

if __name__ == "__main__":
    main()
