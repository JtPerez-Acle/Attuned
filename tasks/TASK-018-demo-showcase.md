# TASK-018: Demo & Showcase Readiness

## Status
- [ ] Not Started
- [ ] In Progress
- [ ] Completed
- [x] Superseded by TASK-019

**Priority**: Low (Superseded)
**Created**: 2025-12-18
**Last Updated**: 2025-12-18
**Phase**: 6 - DevOps & Release
**Depends On**: TASK-014 (Python bindings)
**Superseded By**: **TASK-019** (Split-Screen Demo - better approach)

> **NOTE**: This task has been superseded by TASK-019 which defines a more compelling
> "split-screen comparison" demo that better demonstrates Attuned's value proposition.
> See [TASK-019-split-screen-demo.md](TASK-019-split-screen-demo.md) for the new approach.

## Task Description

Prepare a polished, video-ready demonstration of the complete Attuned system. The demo should showcase the real system working end-to-end, be recordable in a single session, and serve as both validation and marketing material for v1.0.0.

## Requirements

1. **Working end-to-end demo** that can be recorded
2. **Python-first showcase** (using the new bindings)
3. **Clear narrative** showing the problem → solution flow
4. **Professional documentation** with screenshots/examples
5. **Zero-friction setup** for viewers to replicate

## Demo Scenario

### The Story
Show how Attuned prevents an AI agent from being tone-deaf to user state:

1. **Without Attuned**: Agent sends complex, enthusiastic response to stressed user
2. **With Attuned**: Agent adapts tone, verbosity, and suggestions based on user state

### Demo Script (~3-5 minutes)

```
[Scene 1: The Problem]
- Show a typical LLM interaction
- User is stressed/overwhelmed (imagine: "I'm swamped, just need quick help")
- Agent responds with walls of text, exclamation marks, unsolicited advice
- "This is the uncanny valley of AI assistance"

[Scene 2: Enter Attuned]
- pip install attuned
- Quick Python code showing state capture
- Show the 23 axes briefly (the human model)

[Scene 3: State → Context Translation]
- Demonstrate RuleTranslator transforming state to PromptContext
- Show guidelines, tone, verbosity output
- "This is what your LLM system prompt receives"

[Scene 4: The Difference]
- Same user, same query
- Agent now responds appropriately: concise, calm, actionable
- "Attuned doesn't control what the AI says—it provides context for HOW to say it"

[Scene 5: The Governance]
- Brief mention of forbidden_uses
- "Built-in guardrails prevent misuse"
- Show MANIFESTO philosophy

[Closing]
- GitHub link
- PyPI install command
- "Context, not control"
```

## Technical Deliverables

### 1. Demo Application (`examples/demo/`)

```
examples/demo/
├── README.md              # Setup instructions
├── requirements.txt       # Python deps
├── demo_without_attuned.py
├── demo_with_attuned.py
├── sample_states.py       # Pre-configured user states
└── run_demo.sh           # One-command demo runner
```

### 2. Documentation Polish

| Document | Status | Required Updates |
|----------|--------|------------------|
| README.md | Exists | Add video link, quickstart for Python |
| TECHNICAL.md | Exists | Ensure current |
| Python README | New | In attuned-python crate |
| Examples index | New | Link to demo |

### 3. Visual Assets

- [ ] Architecture diagram (mermaid or SVG)
- [ ] Axis categories visualization
- [ ] Before/after comparison image
- [ ] Terminal recording (asciinema or similar)

### 4. Demo Environment

```bash
# One-liner setup for viewers
git clone https://github.com/user/attuned && cd attuned
pip install attuned  # or: pip install -e crates/attuned-python
python examples/demo/demo_with_attuned.py
```

## Validation Checklist

Before recording:
- [ ] Demo runs without errors
- [ ] All Python bindings work as shown
- [ ] HTTP server starts cleanly
- [ ] Inference produces sensible results
- [ ] No console warnings/errors visible
- [ ] Works on fresh virtualenv

## Recording Setup

### Recommended Tools
- **Terminal**: Clean theme, large font (20pt+)
- **Recording**: OBS, asciinema, or native screen recording
- **Resolution**: 1920x1080 minimum
- **Audio**: Optional narration or silent with captions

### Terminal Prep
```bash
# Clean terminal
export PS1="$ "
clear

# No warnings
export PYTHONWARNINGS="ignore"
```

## Acceptance Criteria

- [ ] Demo script runs end-to-end without intervention
- [ ] Python `pip install attuned` works (local or PyPI test)
- [ ] HTTP server showcases REST API
- [ ] Inference engine demonstrates text → state
- [ ] Documentation has video embed or link
- [ ] README quickstart works for new users
- [ ] Demo can be recorded in single take (<10 minutes)

## Progress Log

- 2025-12-18: Task created as v1.0.0 blocker
- 2025-12-18: **Python bindings ready for demo use**

  ### Available for Demo (from TASK-014)

  The Python API is now complete and can be used for all demo scenarios:

  ```python
  from attuned import Attuned

  # Simple API - the demo star
  state = Attuned(
      verbosity_preference=0.2,  # Brief
      warmth=0.9,                # Warm
      cognitive_load=0.8,        # Overwhelmed
  )

  # Universal - works with ANY LLM
  system_prompt = f"You are an assistant.\n\n{state.prompt()}"

  # Presets for quick scenarios
  state = Attuned.presets.anxious_user()
  state = Attuned.presets.busy_executive()
  ```

  ### Demo Implementation Options

  **Option A: Streamlit** (recommended for visual appeal)
  - Interactive sliders for all 23 axes
  - Real-time prompt context preview
  - Side-by-side LLM comparison
  - Easy to record, good for non-technical audiences

  **Option B: Gradio** (alternative)
  - Similar capabilities
  - Hugging Face ecosystem integration
  - Potentially easier sharing via Spaces

  **Option C: Pure Terminal** (fastest to build)
  - Python script with Rich/Typer for formatting
  - Good for technical audiences
  - Asciinema recording

  ### Validation Test Available

  `examples/demo/rigorous_test_v2.py` provides statistical validation:
  - Tests verbosity, warmth, formality, cognitive load effects
  - Compares baseline vs Attuned conditions
  - Shows actual prompt context injected
  - Reports effect sizes and p-values

  **Validation Results (2025-12-18):**
  - Before translator fixes: 8/13 tests passed (62%)
  - After translator fixes: **11/13 tests passed (85%)**

  | Test | Result | Effect Size |
  |------|--------|-------------|
  | Verbosity control | 4/4 ✓ | d=7.40 |
  | Tone (warm/formal) | 2/2 ✓ | d=4.11 |
  | Cognitive load | 2/2 ✓ | d=1.90 |
  | Base guidelines | 1/1 ✓ | d=1.23 |
  | Combined conditions | 2/3 ◐ | d=2.37 |

  Note: Anxiety reassurance metric (H3) shows model follows guideline in text
  but test metric doesn't capture the phrases used.

  ### Next Steps for Demo

  1. Choose demo framework (Streamlit vs Gradio vs Terminal)
  2. Build interactive playground
  3. Re-run validation tests to verify improved guidelines
  4. Record demo video
