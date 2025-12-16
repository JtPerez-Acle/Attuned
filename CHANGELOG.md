# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with 5 crates
- **attuned-core**: Core types and traits
  - `StateSnapshot` with builder pattern and validation
  - `Source` enum (SelfReport, Inferred, Mixed)
  - `PromptContext` with guidelines, tone, verbosity, flags
  - `RuleTranslator` implementing `Translator` trait
  - 23 canonical axes across 6 categories
  - Telemetry module with health checks and audit events
- **attuned-store**: Storage abstraction
  - `StateStore` async trait
  - `MemoryStore` thread-safe in-memory implementation
  - History support with configurable retention
- **attuned-http**: HTTP reference server
  - REST API for state management (CRUD)
  - Context translation endpoint
  - Health and readiness endpoints
  - Tracing middleware integration
- **attuned-qdrant**: Qdrant backend (stubs only)
- **attuned-cli**: CLI tool (basic structure)
- Documentation
  - README.md with quick start guide
  - NON_GOALS.md with hard constraints
  - DEVELOPMENT.md for contributors
  - STATUS.md with current project state
- 26 passing tests across all crates
- Dual license (MIT / Apache-2.0)

### Architecture Decisions
- Chose `DashMap` over `RwLock<HashMap>` for MemoryStore (better concurrent performance)
- Used `BTreeMap` for axes (deterministic ordering)
- Made `Translator` a trait for extensibility
- Separated telemetry into its own module for optional use

### Not Yet Implemented
- Qdrant backend (methods return `todo!()`)
- Full CLI functionality
- Metrics exposition (/metrics endpoint)
- OpenTelemetry export
- API authentication
- CI/CD pipeline
- Property-based tests
- Benchmarks

## [0.1.0] - TBD

Initial release targeting:
- Stable core API
- Production-ready MemoryStore
- Working HTTP server
- Basic documentation

---

## Release Checklist

Before releasing v1.0.0:

- [ ] All tests passing
- [ ] Clippy warnings resolved
- [ ] Security audit clean (`cargo audit`)
- [ ] Documentation complete
- [ ] CHANGELOG updated
- [ ] Version numbers bumped
- [ ] Git tag created
- [ ] Crates published to crates.io
