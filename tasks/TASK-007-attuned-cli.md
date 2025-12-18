# TASK-007: attuned-cli - Developer CLI Tool

## Status
- [x] Partial (basic `serve` command works)
- [ ] In Progress
- [ ] Completed
- [ ] Blocked

**Priority**: Low (Post-v1.0 Enhancement)
**Created**: 2025-12-16
**Last Updated**: 2025-12-18
**Phase**: 7 - Post-v1.0 Enhancements
**Depends On**: TASK-002, TASK-003 (completed)
**Blocks**: None (optional tooling)

> **Note**: Basic structure exists with `attuned serve` working. Full CLI polish is deferred to post-v1.0. Python bindings are the primary user interface for v1.0.

## Task Description
Build a CLI tool for developers to interact with Attuned during development and testing. Useful for debugging, manual state management, and integration testing.

## Requirements
1. CRUD operations for state
2. Translation testing
3. Multiple output formats (JSON, human-readable)
4. Works with both local memory store and remote HTTP server
5. Built with `clap` for modern CLI experience

## Commands

```bash
# State Management
attuned state get <user_id> [--format json|pretty]
attuned state set <user_id> --axis warmth=0.7 --axis formality=0.3 [--source self_report]
attuned state delete <user_id>
attuned state history <user_id> [--limit 10]

# Translation
attuned translate <user_id> [--format json|pretty]
attuned translate --inline '{"warmth": 0.8, "cognitive_load": 0.9}'

# Server Interaction (when using HTTP backend)
attuned --server http://localhost:8080 state get <user_id>

# Development
attuned serve [--port 8080]            # Start local server
attuned health [--server URL]          # Check server health

# Schema
attuned axes list                       # List all known axes
attuned axes describe <axis_name>       # Describe an axis
```

## Implementation Notes

### Configuration
```rust
#[derive(Parser)]
struct Cli {
    #[clap(long, env = "ATTUNED_SERVER")]
    server: Option<String>,

    #[clap(long, default_value = "pretty")]
    format: OutputFormat,

    #[clap(subcommand)]
    command: Commands,
}
```

### Backend Selection
- If `--server` provided: use HTTP client
- Otherwise: use in-memory store (ephemeral, for testing)
- Future: local SQLite for persistent local storage

### Output Formats
```rust
pub enum OutputFormat {
    Json,           // Raw JSON
    Pretty,         // Colorized, human-readable
    Quiet,          // Minimal output (for scripts)
}
```

## Acceptance Criteria
- [ ] All commands implemented
- [ ] Works with local and remote backends
- [ ] JSON and pretty output formats
- [ ] Shell completions generated (bash, zsh, fish)
- [ ] Man page generated
- [ ] Integration tests for CLI commands
- [ ] Useful error messages with suggestions

## Progress Log
- 2025-12-16: Task created
