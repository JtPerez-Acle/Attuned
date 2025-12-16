# Suggested Commands for Attuned Development

## System: Linux

## Build Commands

```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Build specific crate
cargo build -p attuned-core
cargo build -p attuned-store
cargo build -p attuned-http
cargo build -p attuned-qdrant
cargo build -p attuned-cli
```

## Test Commands

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p attuned-core
cargo test -p attuned-store
cargo test -p attuned-http

# Run specific test
cargo test -p attuned-core translator::tests::test_high_cognitive_load

# Run with output visible
cargo test -- --nocapture

# Run doc tests only
cargo test --doc
```

## Linting & Formatting

```bash
# Format all code
cargo fmt

# Check formatting without changing
cargo fmt -- --check

# Run clippy linter
cargo clippy --all-targets

# Clippy with all warnings as errors
cargo clippy --all-targets -- -D warnings
```

## Running the Server

```bash
# Run CLI (basic structure only)
cargo run -p attuned-cli -- serve

# Run with specific bind address
cargo run -p attuned-cli -- serve --bind 0.0.0.0:8080
```

## API Testing (when server is running)

```bash
# Health check
curl http://localhost:8080/health

# Ready check
curl http://localhost:8080/ready

# Set state
curl -X POST http://localhost:8080/v1/state \
  -H "Content-Type: application/json" \
  -d '{"user_id": "u_123", "source": "self_report", "confidence": 1.0, "axes": {"warmth": 0.6, "formality": 0.3}}'

# Get state
curl http://localhost:8080/v1/state/u_123

# Get context (translated)
curl http://localhost:8080/v1/context/u_123

# Delete state
curl -X DELETE http://localhost:8080/v1/state/u_123

# Translate inline
curl -X POST http://localhost:8080/v1/translate \
  -H "Content-Type: application/json" \
  -d '{"user_id": "temp", "source": "self_report", "axes": {"cognitive_load": 0.9}}'
```

## Task Management

```bash
# List all active tasks
ls tasks/TASK-*.md

# Find high priority tasks
grep -l "Priority: High" tasks/TASK-*.md

# Find tasks in progress
grep -l "\- \[x\] In Progress" tasks/TASK-*.md

# Search for specific topics
grep -r "authentication" tasks/
```

## Git Commands

```bash
# Standard git workflow
git status
git add .
git commit -m "message"
git push

# View recent commits
git log --oneline -10

# Check diff
git diff
```

## Utility Commands

```bash
# List project structure
ls -la

# Find files
find . -name "*.rs" -type f

# Search in code
grep -r "pattern" crates/

# Watch for changes and rebuild (requires cargo-watch)
cargo watch -x build

# Check dependencies for security issues (requires cargo-audit)
cargo audit
```

## Documentation

```bash
# Generate and open documentation
cargo doc --open

# Generate docs for all crates
cargo doc --workspace
```

## Dependency Management

```bash
# Update dependencies
cargo update

# Check outdated dependencies (requires cargo-outdated)
cargo outdated

# Tree view of dependencies
cargo tree
```

## Benchmarks (when available)

```bash
# Run benchmarks (criterion)
cargo bench
```
