# Development Standards & Guidelines

This document outlines the architectural standards, code quality conventions,
and contribution workflows applied to this project and fork.

---

## 1. Core Architecture Principles

1. **Separation of State and Runtime**:
   - `AppState` and `Workspace` represent pure data structures testable without
     PTYs or asynchronous event loops (`AppState::test_new()`).
   - `PaneState` remains decoupled from `PaneRuntime`.
2. **Pure Rendering Pipeline**:
   - `compute_view()` handles layout geometry and state projections.
   - `render()` accepts `&AppState` as read-only input. Render functions must
     never mutate state.
3. **Decoupled Detection & Lifecycle**:
   - Agent detection operates over immutable screen snapshots and process
     inspection (`src/detect/`).
   - Detection manifests define state rules declaratively via AND/OR/NOT logic
     gates without touching the active terminal emulator viewport.
4. **Boundary Isolation & Modularity**:
   - No god objects. Modules are scoped strictly to their responsibilities
     (`state`, `actions`, `input`).
   - Platform-specific code resides in `src/platform/<os>.rs`, with
     cross-platform abstractions in `src/platform/mod.rs`.
5. **Stable Wire Protocol (Generation 1 Contract)**:
   - Client-server wire codec definitions remain backward compatible.
   - Wire tag digests and method schemas in
     `tests/fixtures/endpoint-method-shapes-v1.json` are frozen contracts.

---

## 2. Tooling and Development Commands

### Building

```bash
# Debug build
cargo build

# Optimized release binary
cargo build --release
```

### Testing

Use `just` recipes or `cargo nextest` to execute tests:

```bash
# Run unit and integration tests
cargo nextest run --locked

# For environments with global git hooks (e.g. pre-commit linters):
GIT_CONFIG_GLOBAL=/dev/null cargo nextest run --locked

# Run specific module tests
cargo test --bin herdr <filter>
```

### Linting & Formatting

```bash
# Format check
cargo fmt --check

# Strict Clippy check
cargo clippy --all-targets --locked -- -D warnings
```

---

## 3. Coding Conventions & Quality Rules

- **No Panics in Production Paths**: Never use `unwrap()` or `expect()` in
  production runtime paths. Return explicit `io::Result<T>` or custom errors
  and propagate them cleanly.
- **Thread Spawning Resilience**: Spawn worker threads via
  `src/thread_spawn.rs` to handle operating system thread limits (`EAGAIN`)
  gracefully without panicking or terminating the server process.
- **Code Comments**: Every module, struct, public method, and non-trivial
  algorithm must have clear documentation comments describing purpose,
  parameters, error cases, and return values.
- **Conventional Commits**: Commit messages follow conventional format:

  ```text
  <type>(<scope>): <subject> (#issue)
  ```

  Example: `feat(detect): support Muse Code 1.4+ prompts (#1)`

---

## 4. Fork Maintenance & Upstream Policy

1. **Remote Repository Alignment**:
   - Upstream canonical: `git@github.com:herdrdev/herdr.git` (`upstream`).
   - Fork repository: `git@github.com:alfonsodg/herdr.git` (`origin`).
2. **Branching Model**:
   - `master`: Synchronized fast-forward mirror of `upstream/master`.
   - `develop`: Integration branch containing verified features, bug fixes,
     and agent extensions.
3. **Upstream Contribution Rules**:
   - Herdr enforces an approved-contributor policy
     (`.github/APPROVED_CONTRIBUTORS`). Unsolicited pull requests are
     automatically closed by `kangal-bot`.
   - Bug reports are filed via factual, reproducible GitHub issue
     submissions matching the official issue template.
   - New agent proposals and architectural discussions belong in GitHub
     Discussions before pull requests are opened.
