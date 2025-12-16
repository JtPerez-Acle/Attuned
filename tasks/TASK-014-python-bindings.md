# TASK-014: Python Bindings

## Status
- [x] Not Started
- [ ] In Progress
- [ ] Completed
- [ ] Blocked

**Priority**: Medium
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: 5 - Ecosystem
**Depends On**: TASK-011 (Release v1.0)
**Blocks**: None

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

- [ ] `pip install attuned` works on Linux/macOS/Windows
- [ ] StateSnapshot, PromptContext, AxisDefinition wrapped
- [ ] RuleTranslator.to_prompt_context() works
- [ ] Async HTTP client with context manager
- [ ] Type hints work in VS Code / PyCharm
- [ ] Examples for LangChain and FastAPI
- [ ] CI builds wheels for all platforms
- [ ] Published to PyPI

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
