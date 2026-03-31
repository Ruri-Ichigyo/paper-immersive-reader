# Rust Workspace Scaffold Implementation Plan

**Status:** Planned
**Date:** 2026-03-27

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a Rust Cargo workspace scaffold for `paper-immersive-reader` with `core`, `gateway`, and `cli` crates, governance-compliant directories, initial logging bootstrap, and initial tests.

**Architecture:** The scaffold uses a root Cargo workspace with three member crates under `crates/`. `core` contains reusable domain and infrastructure-agnostic logic, while `gateway` and `cli` act as future delivery surfaces. The initial implementation keeps all behavior minimal and structural, focusing on buildability, testability, and directory compliance rather than product features.

**Tech Stack:** Rust, Cargo workspace, `tracing`, `tracing-subscriber`, Rust integration tests

---

### Task 1: Create root workspace manifest and required governance files

**Files:**
- Create: `Cargo.toml`
- Create: `progress.txt`
- Create: `lesson.md`
- Create: `src/main.rs`

**Step 1: Write the failing test expectation for workspace visibility**
Create `tests/workspace_smoke.rs` with a minimal smoke test plan comment block describing expected crates and root layout.

**Step 2: Run test to verify the workspace does not exist yet**
Run: `cargo test -q`
Expected: FAIL because `Cargo.toml` does not exist yet.

**Step 3: Write minimal root workspace manifest**
Create `Cargo.toml` defining a workspace with members:
- `crates/core`
- `crates/gateway`
- `crates/cli`

Also define a minimal root package only if needed for `src/main.rs`; otherwise keep the root as workspace-only and document that the root `src/` is reserved.

**Step 4: Create governance support files**
Create:
- `progress.txt` with an initial milestone entry stub
- `lesson.md` with a placeholder retrospective section header

**Step 5: Add minimal root entrypoint**
Create `src/main.rs` only if the chosen manifest structure includes a root package; otherwise do not create executable code and document the reserved role of root `src/` in the plan execution notes.

**Step 6: Run test to verify the workspace manifest is valid**
Run: `cargo metadata --no-deps`
Expected: PASS and list workspace members.

### Task 2: Create the `core` crate

**Files:**
- Create: `crates/core/Cargo.toml`
- Create: `crates/core/src/lib.rs`
- Test: `tests/core_smoke.rs`

**Step 1: Write the failing integration test**
Create `tests/core_smoke.rs`:
```rust
#[test]
fn core_crate_is_linkable() {
    let version = paper_immersive_reader_core::workspace_marker();
    assert_eq!(version, "core");
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test tests::core_smoke --quiet`
Expected: FAIL because the crate does not exist yet.

**Step 3: Write minimal implementation**
Create `crates/core/src/lib.rs` exporting:
```rust
pub fn workspace_marker() -> &'static str {
    "core"
}
```
Configure `crates/core/Cargo.toml` with package name `paper-immersive-reader-core` and library target accessible as `paper_immersive_reader_core`.

**Step 4: Run test to verify it passes**
Run: `cargo test --test core_smoke -q`
Expected: PASS.

### Task 3: Create the `gateway` crate

**Files:**
- Create: `crates/gateway/Cargo.toml`
- Create: `crates/gateway/src/main.rs`
- Test: `tests/gateway_smoke.rs`

**Step 1: Write the failing gateway test**
Create `tests/gateway_smoke.rs` with a smoke assertion that the gateway crate exposes a constant or function through a tiny library module, or alternatively update the plan to validate gateway by `cargo check -p paper-immersive-reader-gateway`.

**Step 2: Run check to verify it fails**
Run: `cargo check -p paper-immersive-reader-gateway`
Expected: FAIL because the package does not exist yet.

**Step 3: Write minimal implementation**
Create a binary crate with a `main.rs` that prints a placeholder startup message or initializes logging and exits.
Add dependency on `paper-immersive-reader-core`.

**Step 4: Run check to verify it passes**
Run: `cargo check -p paper-immersive-reader-gateway`
Expected: PASS.

### Task 4: Create the `cli` crate

**Files:**
- Create: `crates/cli/Cargo.toml`
- Create: `crates/cli/src/main.rs`
- Test: `tests/cli_smoke.rs`

**Step 1: Write the failing CLI validation**
Create `tests/cli_smoke.rs` or use a package check strategy similar to the gateway task.

**Step 2: Run check to verify it fails**
Run: `cargo check -p paper-immersive-reader-cli`
Expected: FAIL because the package does not exist yet.

**Step 3: Write minimal implementation**
Create a binary crate with a placeholder command entrypoint.
Add dependency on `paper-immersive-reader-core`.

**Step 4: Run check to verify it passes**
Run: `cargo check -p paper-immersive-reader-cli`
Expected: PASS.

### Task 5: Add logging bootstrap utilities

**Files:**
- Create: `crates/core/src/logging.rs`
- Modify: `crates/core/src/lib.rs`
- Modify: `crates/gateway/src/main.rs`
- Modify: `crates/cli/src/main.rs`
- Test: `tests/logging_smoke.rs`

**Step 1: Write the failing logging test**
Create `tests/logging_smoke.rs` that verifies the logging bootstrap function returns a path or creates a file path string under `logs/` ending with `.log`.

**Step 2: Run test to verify it fails**
Run: `cargo test --test logging_smoke -q`
Expected: FAIL because logging bootstrap does not exist yet.

**Step 3: Write minimal implementation**
Implement in `crates/core/src/logging.rs`:
- a function that computes a timestamped log file path in `logs/`
- a bootstrap function that initializes `tracing`/`tracing-subscriber`
Keep it minimal and deterministic enough for testing.

**Step 4: Wire bootstrap into binaries**
Update `gateway` and `cli` `main.rs` files to call the logging bootstrap.

**Step 5: Run test to verify it passes**
Run: `cargo test --test logging_smoke -q`
Expected: PASS.

### Task 6: Create required top-level directories and placeholders

**Files:**
- Create: `tests/.gitkeep` (or first real test files already satisfy this)
- Create: `scripts/.gitkeep`
- Create: `database/.gitkeep`
- Create: `logs/.gitkeep`

**Step 1: Verify missing directories**
Run: `ls -alh`
Expected: some required directories may be absent.

**Step 2: Create missing directories**
Ensure `tests/`, `scripts/`, `database/`, and `logs/` exist.

**Step 3: Verify layout compliance**
Run: `ls -alh`
Expected: required directories are present.

### Task 7: Run repository verification

**Files:**
- Read: `AGENTS.md`
- Read: `progress.txt`
- Read: `lesson.md`

**Step 1: Format check**
Run: `cargo fmt --check`
Expected: PASS.

**Step 2: Lint check**
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: PASS.

**Step 3: Test check**
Run: `cargo test --workspace`
Expected: PASS.

**Step 4: Record progress**
Append milestone completion notes to `progress.txt`.

**Step 5: Record debugging notes if any iteration occurred**
Append any setup/debugging lessons to `lesson.md`.

### Task 8: Commit

**Files:**
- Modify: all created files from prior tasks

**Step 1: Review changed files**
Run: `git status` and `git diff --stat`
Expected: only scaffold and governance-support files changed.

**Step 2: Commit**
```bash
git add Cargo.toml src crates tests scripts database logs progress.txt lesson.md docs/plans/2026-03-27-rust-workspace-scaffold-design.md docs/plans/2026-03-27-rust-workspace-scaffold-implementation.md
git commit -m "chore: scaffold rust workspace foundation"
```
Expected: commit created successfully.
