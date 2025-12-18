# Remaining Tasks for Attuned (Updated 2025-12-18)

## v1.0.0 Critical Path

### 1. TASK-014: Python Bindings (CRITICAL) - 90% COMPLETE
**File**: `tasks/TASK-014-python-bindings.md`
**Status**: In Progress (near completion)
**Effort**: ~2 hours remaining
**Why**: Python is the primary user interface for v1.0.0 release

**Completed (2025-12-18)**:
- ✅ PyO3 bindings for core types (StateSnapshot, PromptContext, axes)
- ✅ RuleTranslator wrapper
- ✅ HTTP client (AttunedClient) for distributed deployments
- ✅ Simple "Attuned" class with all 23 axes (universal API)
- ✅ Integration wrappers: OpenAI, Anthropic, LiteLLM (100+ providers)
- ✅ 7 presets for common patterns
- ✅ README documentation
- ✅ Fixed translator guidelines (warmth, formality, verbosity, anxiety)

**Remaining**:
- [ ] Type stubs (.pyi) for full IDE autocomplete
- [ ] CI/CD wheel building
- [ ] PyPI publishing

**Validation Results (2025-12-18):** 11/13 tests passed (85%)
- Verbosity: d=7.40 (massive effect)
- Tone: d=4.11 (warm), d=0.83 (formal)  
- Cognitive load: d=1.90 (large)
- Combined conditions work correctly

### 2. TASK-018: Demo & Showcase (CRITICAL)
**File**: `tasks/TASK-018-demo-showcase.md`
**Status**: In Progress
**Effort**: ~4 hours
**Why**: Video-ready demo validates the complete system; required for v1.0.0

**Scope**:
- End-to-end demo script
- Python example application
- Documentation polish
- Recording-ready environment

### 3. TASK-011: Release v1.0 (Blocked)
**File**: `tasks/TASK-011-release-v1.md`
**Status**: Blocked on TASK-014, TASK-018
**Effort**: ~2 hours
**Why**: Final release after Python bindings and demo complete

**Scope**:
- Version bump to 1.0.0
- CHANGELOG update
- crates.io + PyPI publishing
- GitHub release

---

## Post-v1.0 Enhancements (Deferred)

### TASK-004: Qdrant Backend
**File**: `tasks/TASK-004-attuned-qdrant-backend.md`
**Priority**: Low
**Why**: MemoryStore sufficient for v1.0; Qdrant enables distributed production

### TASK-007: CLI Tool Completion
**File**: `tasks/TASK-007-attuned-cli.md`
**Priority**: Low
**Why**: Basic `serve` works; Python is the primary interface

---

## Completed Tasks

| Task | Description |
|------|-------------|
| TASK-017 | Inference integration |
| TASK-016 | Inference research refinements |
| TASK-015 | Inference validation (Dreaddit) |
| TASK-013 | Governance & schema formalization (MANIFESTO.md, AxisDefinition) |
| TASK-012 | Security hardening (headers, rate limiting, auth) |
| TASK-010 | CI/CD pipeline (GitHub Actions) |
| TASK-009 | Documentation |
| TASK-008 | Testing infrastructure (property tests, benchmarks) |
| TASK-006 | HTTP server |
| TASK-005 | Observability infrastructure |
| TASK-003 | StateStore + MemoryStore |
| TASK-002 | Core types & traits |
| TASK-001 | Workspace scaffolding |

---

## Known Issues

1. **Axis count mismatch**: AGENTS.md says 24 axes, implementation has 23. The count is correct; spec was off by one.

2. **Doc-tests ignored**: Some doc-tests marked `ignore` because they require runtime setup. Intentional.

3. **No Cargo.lock tracked**: `.gitignore` excludes it since this is a library workspace.
