# TASK-011: Release v1.0.0

## Status
- [ ] Not Started
- [ ] In Progress
- [ ] Completed
- [x] Blocked (waiting on TASK-014, TASK-018)

**Priority**: High
**Created**: 2025-12-16
**Last Updated**: 2025-12-18
**Phase**: 6 - DevOps & Release
**Depends On**: TASK-001 through TASK-013 (completed), **TASK-014 (Python bindings)**, TASK-018 (Demo)
**Blocks**: PyPI publication, crates.io publication

## Task Description
Prepare and execute the first production release of Attuned. This includes final polish, version bumping, changelog generation, and publishing.

## Pre-Release Checklist

### Code Quality
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Code coverage >80% on core
- [ ] Benchmarks baselined
- [ ] Security audit clean

### Documentation
- [ ] README complete and accurate
- [ ] NON_GOALS.md finalized
- [ ] All rustdoc builds
- [ ] Examples work out of the box
- [ ] CHANGELOG.md written

### API Stability
- [ ] Public API reviewed for stability
- [ ] Breaking changes documented
- [ ] Deprecations marked if any
- [ ] Version numbers consistent across crates

### Infrastructure
- [ ] CI/CD pipeline working
- [ ] crates.io account ready
- [ ] Docker Hub account ready
- [ ] GitHub release permissions

## Release Process

### 1. Version Bump
```bash
# Update Cargo.toml versions
# All crates should have matching versions for v1.0.0
cargo set-version 1.0.0 --workspace
```

### 2. Update CHANGELOG
```markdown
# Changelog

## [1.0.0] - 2025-XX-XX

### Added
- Initial release of Attuned
- Core types: StateSnapshot, Source, PromptContext, Verbosity
- RuleTranslator with configurable thresholds
- StateStore trait with in-memory and Qdrant backends
- HTTP reference server
- CLI tool for development
- Full observability: tracing, metrics, OpenTelemetry
- Comprehensive documentation

### Security
- Input validation on all axis values
- No action execution by design
- Self-report always overrides inference
```

### 3. Create Release Tag
```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

### 4. Verify Release
- [ ] GitHub Release created automatically
- [ ] Crates published to crates.io
- [ ] Docker images on Docker Hub
- [ ] docs.rs builds successfully

### 5. Post-Release
- [ ] Announce on relevant channels
- [ ] Monitor for issues
- [ ] Update project boards

## Crate Publishing Order

Due to dependencies, crates must be published in this order:
1. `attuned-core` (no internal deps)
2. `attuned-store` (depends on core)
3. `attuned-infer` (depends on core)
4. `attuned-qdrant` (depends on core, store) - optional
5. `attuned-http` (depends on core, store, infer)
6. `attuned-cli` (depends on all) - optional
7. `attuned-python` → **PyPI as `attuned`** (depends on core, infer)

Wait 30-60 seconds between publishes for crates.io indexing.

## PyPI Publishing

After crates.io:
```bash
cd crates/attuned-python
maturin build --release
maturin publish  # Publishes to PyPI as 'attuned'
```

## Version Strategy

### Semantic Versioning
- MAJOR: Breaking API changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes, backward compatible

### Pre-1.0 vs Post-1.0
- v1.0.0 signals API stability commitment
- After v1.0.0, breaking changes require major version bump
- Consider starting at v0.1.0 if API may change

## Risk Mitigation

### Rollback Plan
If critical issues found:
1. Yank affected crate versions
2. Fix issue
3. Release patch version

### Known Limitations
Document any v1.0 limitations:
- Performance characteristics
- Unsupported configurations
- Future roadmap items

## Acceptance Criteria
- [ ] All dependent tasks completed
- [ ] Version 1.0.0 tagged
- [ ] All crates on crates.io
- [ ] Docker images published
- [ ] GitHub Release with notes
- [ ] Documentation live on docs.rs
- [ ] No critical issues in first 48 hours

## Progress Log
- 2025-12-16: Task created
