# TASK-009: Documentation

## Status
- [ ] Not Started
- [ ] In Progress
- [ ] Completed
- [ ] Blocked

**Priority**: Medium
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: 5 - Documentation
**Depends On**: TASK-002, TASK-003, TASK-005
**Blocks**: TASK-011 (Release)

## Task Description
Create comprehensive documentation for users, integrators, and contributors. Documentation is critical for adoption and long-term maintenance.

## Requirements
1. README.md with quick start
2. API documentation (rustdoc)
3. NON_GOALS.md (hard constraints)
4. Architecture overview
5. Integration guides
6. Contributing guide

## Documentation Structure

```
attuned/
├── README.md                    # Overview, quick start, badges
├── NON_GOALS.md                 # Hard constraints (from AGENTS.md)
├── CONTRIBUTING.md              # How to contribute
├── CHANGELOG.md                 # Release history
├── docs/
│   ├── architecture.md          # System design overview
│   ├── axes.md                  # Axis schema documentation
│   ├── integration/
│   │   ├── langchain.md         # LangChain integration guide
│   │   ├── openai.md            # OpenAI API integration
│   │   └── custom.md            # Custom integration patterns
│   ├── deployment/
│   │   ├── docker.md            # Docker deployment
│   │   ├── kubernetes.md        # K8s deployment
│   │   └── observability.md     # Setting up monitoring
│   └── api/
│       └── http.md              # HTTP API reference
└── examples/
    ├── basic/                   # Minimal usage
    ├── with-langchain/          # LangChain integration
    └── full-stack/              # Complete deployment
```

## Key Documents

### README.md
- Project description (from AGENTS.md)
- Quick start (5 lines to working code)
- Feature highlights
- Installation instructions
- Links to detailed docs
- Badges (crates.io, docs.rs, CI status)

### NON_GOALS.md
Expand from AGENTS.md:
- No action execution
- No persuasion optimization
- No covert inference drift
- No UI
- No content memory
- Each with detailed rationale

### Architecture Overview
- Diagram of crate relationships
- Data flow from input to PromptContext
- Storage backend options
- Extension points

### Axes Documentation
For each of the 24 axes:
- Name and category
- Description of what it measures
- Range interpretation (what 0.0 vs 1.0 means)
- How it affects translation
- Example use cases

### Integration Guides
Step-by-step guides for common scenarios:
- Adding to an existing LLM app
- Using with LangChain/LlamaIndex
- Custom translator implementation
- Storing state in different backends

## Rustdoc Standards
- All public items documented
- Examples in doc comments
- `#[doc(hidden)]` for internal items
- Module-level documentation
- Crate-level documentation with examples

```rust
/// A snapshot of user state at a point in time.
///
/// # Examples
///
/// ```
/// use attuned_core::{StateSnapshot, Source};
///
/// let snapshot = StateSnapshot::builder()
///     .user_id("user_123")
///     .source(Source::SelfReport)
///     .axis("warmth", 0.7)
///     .build();
/// ```
pub struct StateSnapshot { ... }
```

## Acceptance Criteria
- [ ] README.md with working quick start
- [ ] NON_GOALS.md with full rationale
- [ ] Architecture doc with diagrams
- [ ] All 24 axes documented
- [ ] At least 2 integration guides
- [ ] Deployment guide for Docker
- [ ] Observability setup guide
- [ ] rustdoc builds without warnings
- [ ] Examples compile and run
- [ ] CONTRIBUTING.md with development setup

## Progress Log
- 2025-12-16: Task created
- 2025-12-16: Created README.md with quick start examples
- 2025-12-16: Created NON_GOALS.md with hard constraints
- 2025-12-16: Added rustdoc comments to all public items
- 2025-12-16: **PARTIALLY COMPLETED** - Core docs done, integration guides pending
