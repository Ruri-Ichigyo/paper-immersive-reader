# Rust Workspace Scaffold Design

**Status:** Approved
**Date:** 2026-03-27

## Goal
Define the first implementation milestone for the `paper-immersive-reader` codebase as a Rust workspace scaffold that is consistent with the repository governance in `AGENTS.md` and ready for later reader, sync, and integration features.

## Milestone Scope (M0)
This milestone creates the project skeleton only. It does **not** implement PDF parsing, reading UI, sync, OCR, or Zotero logic.

The milestone is successful when:
- the workspace layout is defined
- `cargo test` is expected to run successfully
- `cargo fmt` and `cargo clippy` are expected to run successfully
- logging has a clear standard for writing timestamped files into `logs/`
- repository layout follows `AGENTS.md`

## Why Workspace First
The product documents already imply several separable subsystems:
- domain/core logic
- service/integration logic
- developer/local task entrypoints

Starting with a single crate would make early code placement easy, but would likely force a structural split later once import, sync, OCR, and gateway features appear. A workspace scaffold avoids that reorganization cost.

## Selected Structure

### Root
- `Cargo.toml` — workspace definition
- `src/` — reserved only for the root entrypoint or thin glue if absolutely needed
- `crates/` — all workspace member crates
- `tests/` — workspace integration tests
- `docs/` — documentation
- `scripts/` — initialization/start scripts
- `database/` — database-related files
- `logs/` — runtime logs
- `progress.txt` — milestone progress ledger
- `lesson.md` — debugging and iteration retrospective

### Workspace crates
- `crates/core`
  - purpose: domain types, shared errors, config types, storage abstractions, service traits
  - must avoid web-framework coupling
- `crates/gateway`
  - purpose: future HTTP/API surface for sync, AI proxying, OCR jobs, Zotero integration
  - depends on `core`
- `crates/cli`
  - purpose: future local utilities, maintenance tasks, import helpers, developer workflows
  - depends on `core`

## Boundary Rules

### `core`
- pure domain and service-boundary logic only
- no transport-layer assumptions
- no framework-specific types exposed publicly

### `gateway`
- owns HTTP-facing entrypoints
- translates requests into `core` operations
- must keep business rules out of handlers when possible

### `cli`
- owns command parsing and task invocation
- reuses `core` for logic instead of duplicating behavior

## Logging Design
Because `AGENTS.md` preserves a Python-oriented rule while the project stack is Rust, the implementation should use the Rust-equivalent approach:
- `tracing` as the logging facade
- `tracing-subscriber` for subscriber setup
- file appender configured to create `logs/YYYY-MM-DD_HHMMSS.log`
- log level configurable through environment variables such as `RUST_LOG`

This keeps runtime behavior aligned with the governance document without pretending the stack is Python.

## Testing Design
- unit tests live inside each crate as appropriate
- integration tests live in root `tests/`
- the default verification path is `cargo test`

## Non-Goals
This scaffold milestone does **not** decide:
- the final HTTP framework (`axum` vs others)
- the persistence engine
- the sync protocol implementation
- the OCR engine
- the reader frontend implementation

Those belong to later milestones once the workspace exists.

## Trade-offs

### Rejected option: single binary crate
Pros:
- fastest to bootstrap
- fewer files initially

Cons:
- invites early coupling
- likely requires a disruptive split later

### Chosen option: workspace with three crates
Pros:
- matches the future system shape
- keeps business logic reusable
- easier to test and evolve boundaries

Cons:
- slightly more setup work at the start
- requires early discipline on crate boundaries

## Validation Criteria
This design is acceptable if the implementation plan built from it will:
- create a valid Cargo workspace
- place code under crate-local `src/`
- keep tests in `tests/`
- define a Rust logging path that writes to `logs/`
- avoid implementing product features prematurely

## Output
This design should lead to an implementation plan that creates:
- workspace manifest
- `core`, `gateway`, and `cli` crates
- initial tests
- initial logging bootstrap
- required governance support files if still missing
