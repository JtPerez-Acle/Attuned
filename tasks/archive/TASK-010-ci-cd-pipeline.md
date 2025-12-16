# TASK-010: CI/CD Pipeline

## Status
- [ ] Not Started
- [ ] In Progress
- [x] Completed
- [ ] Blocked

**Priority**: High
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: 6 - DevOps & Release
**Depends On**: TASK-001, TASK-008
**Blocks**: TASK-011 (Release)

## Task Description
Set up comprehensive CI/CD pipeline for automated testing, building, and releasing. The pipeline ensures quality gates are enforced and releases are reproducible.

## Requirements
1. GitHub Actions for CI
2. Automated testing on every PR
3. Security scanning
4. Release automation
5. Docker image publishing
6. Crate publishing to crates.io

## CI Pipeline (`.github/workflows/ci.yml`)

### Triggers
- Push to main
- Pull requests to main
- Release tags

### Jobs

#### 1. Check
```yaml
check:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
    - run: cargo fmt --all --check
    - run: cargo clippy --all-targets --all-features -- -D warnings
```

#### 2. Test
```yaml
test:
  runs-on: ubuntu-latest
  services:
    qdrant:
      image: qdrant/qdrant:latest
      ports:
        - 6333:6333
        - 6334:6334
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo test --all-features
    - run: cargo test --no-default-features
```

#### 3. Coverage
```yaml
coverage:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: llvm-tools-preview
    - uses: taiki-e/install-action@cargo-llvm-cov
    - run: cargo llvm-cov --all-features --lcov --output-path lcov.info
    - uses: codecov/codecov-action@v3
      with:
        files: lcov.info
```

#### 4. Security Audit
```yaml
audit:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: rustsec/audit-check@v1
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
```

#### 5. Build Matrix
```yaml
build:
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
      rust: [stable, beta]
  runs-on: ${{ matrix.os }}
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}
    - run: cargo build --all-features
```

## Release Pipeline (`.github/workflows/release.yml`)

### Trigger
```yaml
on:
  push:
    tags:
      - 'v*'
```

### Jobs

#### 1. Create Release
```yaml
release:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Create GitHub Release
      uses: softprops/action-gh-release@v1
      with:
        generate_release_notes: true
```

#### 2. Publish Crates
```yaml
publish:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Publish to crates.io
      run: |
        cargo publish -p attuned-core
        sleep 30  # Wait for crates.io to index
        cargo publish -p attuned-store
        sleep 30
        cargo publish -p attuned-qdrant
        cargo publish -p attuned-http
        cargo publish -p attuned-cli
      env:
        CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

#### 3. Docker Images
```yaml
docker:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: docker/setup-buildx-action@v3
    - uses: docker/login-action@v3
      with:
        username: ${{ secrets.DOCKERHUB_USERNAME }}
        password: ${{ secrets.DOCKERHUB_TOKEN }}
    - uses: docker/build-push-action@v5
      with:
        push: true
        tags: |
          attuned/server:${{ github.ref_name }}
          attuned/server:latest
```

## Additional Workflows

### Benchmarks (on main only)
```yaml
benchmarks:
  if: github.ref == 'refs/heads/main'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - run: cargo bench --all-features
    - uses: benchmark-action/github-action-benchmark@v1
```

### Dependency Updates
```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: weekly
```

## Branch Protection Rules
- Require PR reviews
- Require status checks to pass
- Require branches to be up to date
- No force push to main

## Acceptance Criteria
- [x] CI runs on every PR
- [x] All tests pass before merge
- [x] Code coverage tracked
- [x] Security audit runs automatically
- [x] Multi-platform builds work
- [x] Release workflow publishes to crates.io
- [ ] Docker images published on release (skipped - no Dockerfile yet)
- [ ] Branch protection configured (requires GitHub repo admin)
- [x] Dependabot configured

## Progress Log
- 2025-12-16: Task created
- 2025-12-16: Created CI workflow with fmt, clippy, test, doc, build, MSRV, audit, coverage jobs
- 2025-12-16: Created release workflow with validation, GitHub release, crates.io publishing, binary builds
- 2025-12-16: Added dependabot configuration for Cargo and GitHub Actions
- 2025-12-16: Fixed clippy warnings (derivable_impls for Source and Verbosity enums)
- 2025-12-16: **COMPLETED** - CI/CD pipeline ready for use
