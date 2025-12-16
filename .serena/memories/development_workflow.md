# Attuned Development Workflow

## Code Style & Conventions

### Rust Style
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- All public items must have doc comments
- Use `#[must_use]` for functions with important return values
- Prefer `thiserror` for error types, `anyhow` for application code

### Test Organization
- Tests go in same file as code: `#[cfg(test)] mod tests { ... }`
- Use descriptive test names: `test_high_cognitive_load_produces_concise_guideline`
- Integration tests in `/tests/` directory

### Naming Conventions
- Types: PascalCase (`StateSnapshot`, `PromptContext`)
- Functions/methods: snake_case (`upsert_latest`, `get_history`)
- Constants: SCREAMING_SNAKE_CASE (`CANONICAL_AXES`)
- Modules: snake_case (`attuned_core`, `telemetry`)

### Error Handling
- Use `Result<T, E>` with typed errors for library code
- Return `anyhow::Result` in CLI and server code
- Use `?` operator for propagation
- Provide context with `.context()` from anyhow

## Task Completion Checklist

When completing a task:

1. **Run tests**: `cargo test`
2. **Run linter**: `cargo clippy --all-targets`
3. **Format code**: `cargo fmt`
4. **Update task file**: Mark status as completed, add progress log entry
5. **Update STATUS.md**: If task affects project status
6. **Update CHANGELOG.md**: For user-facing changes

## Adding New Features

### Adding a New Axis
1. Add to `CANONICAL_AXES` in `crates/attuned-core/src/axes.rs`
2. Update test count in `test_canonical_axes_count`
3. Add translation rules in `RuleTranslator::to_prompt_context` if behavior depends on it
4. Update README.md documentation

### Adding a New API Endpoint
1. Add handler in `crates/attuned-http/src/handlers.rs`
2. Register route in `crates/attuned-http/src/server.rs`
3. Add tests for the endpoint
4. Update README.md API table

### Adding a New Storage Backend
1. Create new crate: `crates/attuned-{backend}/`
2. Implement `StateStore` trait from `attuned-store`
3. Add to workspace in root `Cargo.toml`
4. Add integration tests

## Task Management

### Task Files Location
`tasks/TASK-*.md` - Active tasks
`tasks/archive/` - Completed tasks

### Task File Format
```markdown
# TASK-XXX: Title

## Status
- [ ] Not Started / [x] In Progress / [x] Completed

**Priority**: High | Medium | Low
**Created**: YYYY-MM-DD

## Task Description
...

## Progress Log
- YYYY-MM-DD: Entry
```

### Picking a Task
1. Check STATUS.md for recommended priority order
2. Look at `tasks/` directory for task files
3. Pick task matching your skill level and priority
4. Update task status to "In Progress"

## Remaining Tasks (Priority Order)

1. **TASK-010** (CI/CD) - Essential for quality gates
2. **TASK-008** (Testing) - Property tests catch edge cases
3. **TASK-012** (Security) - Required before v1.0
4. **TASK-004** (Qdrant) - Enables persistent storage
5. **TASK-011** (Release) - Final polish
6. **TASK-007** (CLI) - Nice to have

## Documentation Standards

- User-facing docs: README.md
- Developer docs: DEVELOPMENT.md
- Status tracking: STATUS.md
- Change log: CHANGELOG.md
- Constraints: NON_GOALS.md
- Project spec: AGENTS.md
