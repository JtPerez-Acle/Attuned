# Attuned Code Style & Conventions

## Rust Edition
- Rust 2021 edition
- Minimum supported Rust version: 1.75

## Formatting
- Use `cargo fmt` (rustfmt) with default settings
- Run before every commit
- CI should fail on formatting issues

## Linting
- Use `cargo clippy --all-targets`
- Fix all warnings before committing
- No `#[allow(...)]` without good reason and comment

## Documentation
- All public items MUST have doc comments (`///`)
- Use examples in doc comments where helpful
- Doc tests should compile (use `ignore` if they need runtime setup)

## Error Handling
- Library crates: Use `thiserror` for typed errors
- Application code: Use `anyhow::Result` for convenience
- Always propagate context: `.context("what failed")?`

## Async Patterns
- Use `tokio` as the async runtime
- Use `async-trait` for async trait methods
- Prefer `tokio::spawn` for background tasks
- Always handle task join errors

## Testing
- Place tests in same file: `#[cfg(test)] mod tests { ... }`
- Use descriptive test names: `test_<scenario>_<expected_behavior>`
- Test both success and error cases
- Use `#[tokio::test]` for async tests

## Module Organization
- One concept per file
- `lib.rs` only for re-exports and module declarations
- Keep files under 500 lines when possible

## Naming
```rust
// Types: PascalCase
struct StateSnapshot { ... }
enum Source { ... }

// Functions/methods: snake_case
fn get_latest(user_id: &str) -> ... { ... }

// Constants: SCREAMING_SNAKE_CASE
const CANONICAL_AXES: &[AxisDefinition] = ...;

// Crates/modules: snake_case with hyphens for crate names
// attuned-core → attuned_core
```

## Type Patterns
- Use builder pattern for complex construction
- Implement `Default` where sensible defaults exist
- Use `#[derive(Clone, Debug)]` on most types
- Add `#[must_use]` for functions with important return values

## Imports
- Group imports: std, external crates, internal modules
- Use explicit imports over glob imports
- Re-export public API in `lib.rs`

## Example: Typical Module Structure
```rust
//! Module documentation

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::Source;

/// Type documentation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyType {
    field: String,
}

impl MyType {
    /// Constructor documentation
    pub fn new(field: impl Into<String>) -> Self {
        Self { field: field.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_type_creation() {
        let t = MyType::new("test");
        assert_eq!(t.field, "test");
    }
}
```
