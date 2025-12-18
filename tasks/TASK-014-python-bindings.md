# TASK-014: Python Bindings

## Status
- [ ] Not Started
- [x] In Progress
- [ ] Completed
- [ ] Blocked

**Priority**: **Critical** (Required for v1.0.0)
**Created**: 2025-12-16
**Last Updated**: 2025-12-18
**Phase**: 5 - Ecosystem (Core for v1.0)
**Depends On**: TASK-013 (Governance - completed)
**Blocks**: TASK-011 (Release v1.0), TASK-018 (Demo/Showcase)

## Task Description

Create Python bindings for Attuned to enable native usage by Python/ML developers. Python is the dominant language in the AI/ML ecosystem (LangChain, LlamaIndex, etc.), so native bindings will significantly boost adoption.

## Requirements

1. PyO3-based bindings for core Attuned types
2. Async HTTP client for distributed deployments
3. Type stubs (.pyi) for IDE autocomplete
4. Published to PyPI as `attuned`
5. Pre-built wheels for Linux/macOS/Windows

## Architecture

### Option C: Both Native Bindings + HTTP Client

```
crates/
└── attuned-python/
    ├── Cargo.toml         # PyO3 + pyo3-asyncio
    ├── pyproject.toml     # maturin build config
    ├── src/
    │   ├── lib.rs         # PyO3 module definition
    │   ├── snapshot.rs    # StateSnapshot wrapper
    │   ├── axes.rs        # AxisDefinition exposure
    │   ├── translator.rs  # RuleTranslator wrapper
    │   └── client.rs      # Async HTTP client
    └── attuned/
        ├── __init__.py    # Python package
        ├── py.typed       # PEP 561 marker
        └── *.pyi          # Type stubs
```

## Python API Design

```python
from attuned import StateSnapshot, RuleTranslator, Source

# Create a state snapshot
snapshot = StateSnapshot.builder() \
    .user_id("user_123") \
    .source(Source.SELF_REPORT) \
    .axis("warmth", 0.7) \
    .axis("cognitive_load", 0.9) \
    .build()

# Translate to prompt context
translator = RuleTranslator()
context = translator.to_prompt_context(snapshot)

print(context.guidelines)  # ['Offer suggestions, not actions', ...]
print(context.tone)        # 'warm-casual'
print(context.verbosity)   # Verbosity.MEDIUM

# Access axis metadata (governance!)
from attuned import get_axis, CANONICAL_AXES

axis = get_axis("cognitive_load")
print(axis.intent)          # What it's for
print(axis.forbidden_uses)  # What it must NOT be used for

# HTTP client for distributed deployments
from attuned import AttunedClient

async with AttunedClient("http://localhost:8080", api_key="...") as client:
    await client.upsert_state(snapshot)
    context = await client.get_context("user_123")
```

## Integration Examples

### LangChain

```python
from langchain.prompts import SystemMessagePromptTemplate
from attuned import AttunedClient

async def get_conditioned_prompt(user_id: str, base_prompt: str) -> str:
    async with AttunedClient(...) as client:
        context = await client.get_context(user_id)
        return f"""
{base_prompt}

## Interaction Guidelines
{chr(10).join(f'- {g}' for g in context.guidelines)}

Tone: {context.tone}
Verbosity: {context.verbosity.value}
"""
```

### FastAPI Middleware

```python
from fastapi import FastAPI, Request
from attuned import AttunedClient

app = FastAPI()
attuned = AttunedClient("http://localhost:8080")

@app.middleware("http")
async def add_attuned_context(request: Request, call_next):
    user_id = request.headers.get("X-User-ID")
    if user_id:
        request.state.attuned_context = await attuned.get_context(user_id)
    return await call_next(request)
```

## Technical Stack

| Component | Technology |
|-----------|------------|
| Rust → Python | PyO3 |
| Build tool | maturin |
| Async runtime | pyo3-asyncio + tokio |
| HTTP client | reqwest (wrapped) |
| Type stubs | Manual + stubgen |

## Deliverables

| Deliverable | Description |
|-------------|-------------|
| `attuned-python` crate | PyO3 bindings |
| `attuned` PyPI package | Published wheels |
| Type stubs | .pyi files for IDE support |
| Documentation | Python-specific README |
| Examples | LangChain, FastAPI integrations |

## Acceptance Criteria

- [x] `pip install attuned` works on Linux/macOS/Windows (local dev works)
- [x] StateSnapshot, PromptContext, AxisDefinition wrapped
- [x] RuleTranslator.to_prompt_context() works
- [x] HTTP client for distributed deployments
- [ ] Type hints work in VS Code / PyCharm (needs .pyi stubs)
- [x] Examples for OpenAI, Anthropic, LiteLLM
- [ ] CI builds wheels for all platforms
- [ ] Published to PyPI
- [x] **NEW**: Simple `Attuned` class for universal LLM conditioning

## Implementation Steps

1. **Setup** (~1 hour)
   - Create `attuned-python` crate
   - Configure PyO3 and maturin
   - Set up pyproject.toml

2. **Core Bindings** (~3 hours)
   - Wrap StateSnapshot with builder pattern
   - Wrap PromptContext and Verbosity
   - Expose AxisDefinition and CANONICAL_AXES
   - Implement RuleTranslator

3. **HTTP Client** (~2 hours)
   - Async client with pyo3-asyncio
   - Wrap all API endpoints
   - Error handling

4. **Type Stubs** (~1 hour)
   - Generate .pyi files
   - Add py.typed marker

5. **CI/CD** (~2 hours)
   - GitHub Actions for wheel building
   - PyPI publishing workflow

6. **Documentation** (~1 hour)
   - Python-specific README
   - Integration examples

## Progress Log

- 2025-12-16: Task created
- 2025-12-18: **Major Progress - Universal Python API Complete**

  ### Completed Work

  **1. Core Bindings (PyO3)** ✅
  - `StateSnapshot`, `StateSnapshotBuilder`, `Source` wrapped
  - `PromptContext`, `Verbosity`, `RuleTranslator`, `Thresholds` wrapped
  - `AxisDefinition`, `AxisCategory` exposed
  - Module functions: `get_axis`, `is_valid_axis_name`, `get_axis_names`, `get_all_axes`

  **2. HTTP Client** ✅
  - `AttunedClient` with sync API for distributed deployments
  - Supports: `upsert_state()`, `get_context()`, health checks

  **3. Simple "Attuned" API** ✅ (NEW - key differentiator)
  - File: `python/attuned/core.py`
  - `Attuned` class with all 23 axes as kwargs
  - `state.prompt()` → string for ANY LLM system prompt
  - Works with OpenAI, Anthropic, Ollama, Mistral, or any LLM

  **4. Integration Wrappers** ✅
  - `attuned.integrations.openai.AttunedOpenAI`
  - `attuned.integrations.anthropic.AttunedAnthropic`
  - `attuned.integrations.litellm.AttunedLiteLLM` (100+ providers)

  **5. Presets** ✅
  - `Attuned.presets.anxious_user()`
  - `Attuned.presets.busy_executive()`
  - `Attuned.presets.learning_student()`
  - `Attuned.presets.casual_chat()`
  - `Attuned.presets.high_stakes()`
  - `Attuned.presets.overwhelmed()`
  - `Attuned.presets.privacy_conscious()`

  **6. Documentation** ✅
  - README.md with simple examples
  - Docstrings throughout

  **7. Translator Improvements** ✅
  - Fixed vague guidelines in `translator.rs`
  - Added explicit warmth guidelines (high and low)
  - Added explicit formality guidelines (high and low)
  - Added explicit verbosity guidelines (brief and comprehensive)
  - Added specific anxiety guidelines with examples

  ### Remaining Work

  - [ ] Type stubs (.pyi files) for full IDE autocomplete
  - [ ] CI/CD wheel building for Linux/macOS/Windows
  - [ ] PyPI publishing
  - [ ] Re-run validation tests to verify improved guidelines

  ### Key Files

  ```
  crates/attuned-python/
  ├── src/lib.rs              # PyO3 module definition
  ├── python/attuned/
  │   ├── __init__.py         # Main exports
  │   ├── core.py             # Simple Attuned class
  │   ├── integrations/
  │   │   ├── openai.py       # OpenAI wrapper
  │   │   ├── anthropic.py    # Anthropic wrapper
  │   │   └── litellm.py      # LiteLLM (100+ providers)
  └── README.md               # User documentation
  ```

  ### Usage Example

  ```python
  from attuned import Attuned

  # Set any axes (others default to neutral 0.5)
  state = Attuned(
      verbosity_preference=0.2,  # Brief
      warmth=0.9,                # Warm
  )

  # Works with ANY LLM
  system_prompt = f"You are an assistant.\n\n{state.prompt()}"
  ```
