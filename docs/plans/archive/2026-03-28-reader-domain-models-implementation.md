# Reader Domain Models (Minimal Reading Core) Implementation Plan

**Status:** Done
**Date:** 2026-03-28

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the minimal reader domain model in `crates/core` (Paper/Block/ReadingProgress + IDs + DomainError) with integration tests under `tests/`.

**Architecture:** Add a `domain/` module to `paper-immersive-reader-core` that defines persistence-agnostic types and minimal invariants. Keep constructors small and explicit. Integration tests validate the public API surface and basic invariants without introducing storage/import dependencies.

**Tech Stack:** Rust, Cargo workspace, integration tests in `/tests`, clippy/rustfmt.

---

### Task 1: Add `domain` module skeleton and exports

**Files:**
- Create: `crates/core/src/domain/mod.rs`
- Create: `crates/core/src/domain/error.rs`
- Modify: `crates/core/src/lib.rs`

**Step 1: Write the failing test (domain module is exported)**

Create `tests/domain_exports_smoke.rs`:

```rust
#[test]
fn core_domain_module_is_exported() {
    // This should compile once domain is exported.
    let _ = paper_immersive_reader_core::domain::DomainError::EmptyBlockText;
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test domain_exports_smoke -q`

Expected: FAIL with unresolved import/module `domain`.

**Step 3: Implement minimal skeleton**

- Create `crates/core/src/domain/mod.rs` that declares submodules.
- Create `crates/core/src/domain/error.rs` containing:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyBlockText,
}
```

- Update `crates/core/src/lib.rs`:
  - `pub mod domain;`

**Step 4: Run test to verify it passes**

Run: `cargo test --test domain_exports_smoke -q`

Expected: PASS.

---

### Task 2: Implement IDs (`PaperId`, `BlockId`) and paper source/status enums

**Files:**
- Create: `crates/core/src/domain/paper.rs`
- Modify: `crates/core/src/domain/mod.rs`
- Test: `tests/domain_paper_smoke.rs`

**Step 1: Write the failing test**

Create `tests/domain_paper_smoke.rs`:

```rust
use paper_immersive_reader_core::domain::{Paper, PaperId, PaperSourceType, PaperStatus};

#[test]
fn paper_can_be_constructed() {
    let paper = Paper {
        id: PaperId("p1".to_string()),
        title: None,
        source_type: PaperSourceType::Pdf,
        source_ref: None,
        status: PaperStatus::Created,
        parse_version: 1,
    };

    assert_eq!(paper.parse_version, 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test domain_paper_smoke -q`

Expected: FAIL because `Paper`, `PaperId`, and enums don’t exist yet.

**Step 3: Write minimal implementation**

In `crates/core/src/domain/paper.rs`, implement:
- `pub struct PaperId(pub String);`
- `pub enum PaperStatus { Created, Ingesting, ReadyText, ReadyOcr, Failed }`
- `pub enum PaperSourceType { Pdf, Paste, Url, Doi, Zotero }`
- `pub struct Paper { ... }` matching the design doc

In `crates/core/src/domain/mod.rs`:
- `pub mod paper;`
- `pub use paper::{Paper, PaperId, PaperSourceType, PaperStatus};`

**Step 4: Run test to verify it passes**

Run: `cargo test --test domain_paper_smoke -q`

Expected: PASS.

---

### Task 3: Implement `BlockId` and `Block` with validation constructor

**Files:**
- Create: `crates/core/src/domain/block.rs`
- Modify: `crates/core/src/domain/mod.rs`
- Test: `tests/domain_block_validation.rs`

**Step 1: Write the failing test**

Create `tests/domain_block_validation.rs`:

```rust
use paper_immersive_reader_core::domain::{Block, BlockId, PaperId};

#[test]
fn block_rejects_empty_text() {
    let result = Block::try_new(
        BlockId("b1".to_string()),
        PaperId("p1".to_string()),
        0,
        "   ".to_string(),
        None,
    );

    assert!(result.is_err());
}

#[test]
fn block_accepts_non_empty_text() {
    let block = Block::try_new(
        BlockId("b1".to_string()),
        PaperId("p1".to_string()),
        0,
        "Hello".to_string(),
        None,
    )
    .expect("block should be valid");

    assert_eq!(block.order_index, 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test domain_block_validation -q`

Expected: FAIL (Block/BlockId missing).

**Step 3: Write minimal implementation**

In `crates/core/src/domain/block.rs`:
- `pub struct BlockId(pub String);`
- `pub struct Block { ... }`
- `impl Block { pub fn try_new(...) -> Result<Self, DomainError> { ... } }`
  - return `Err(DomainError::EmptyBlockText)` if `text.trim().is_empty()`

In `crates/core/src/domain/mod.rs`:
- `pub mod block;`
- `pub use block::{Block, BlockId};`

**Step 4: Run test to verify it passes**

Run: `cargo test --test domain_block_validation -q`

Expected: PASS.

---

### Task 4: Implement `ReadingProgress`

**Files:**
- Create: `crates/core/src/domain/reading_progress.rs`
- Modify: `crates/core/src/domain/mod.rs`
- Test: `tests/domain_progress_smoke.rs`

**Step 1: Write the failing test**

Create `tests/domain_progress_smoke.rs`:

```rust
use paper_immersive_reader_core::domain::{PaperId, ReadingProgress};

#[test]
fn reading_progress_can_be_constructed() {
    let progress = ReadingProgress {
        paper_id: PaperId("p1".to_string()),
        current_block_index: 3,
        current_block_id: None,
        updated_at_ms: 123,
    };

    assert_eq!(progress.current_block_index, 3);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test domain_progress_smoke -q`

Expected: FAIL because `ReadingProgress` doesn’t exist yet.

**Step 3: Write minimal implementation**

In `crates/core/src/domain/reading_progress.rs` implement `pub struct ReadingProgress { ... }` per design.

In `crates/core/src/domain/mod.rs`:
- `pub mod reading_progress;`
- `pub use reading_progress::ReadingProgress;`

**Step 4: Run test to verify it passes**

Run: `cargo test --test domain_progress_smoke -q`

Expected: PASS.

---

### Task 5: Repository verification (format/lint/test)

**Step 1: rustfmt**
Run: `cargo fmt --check`
Expected: PASS.

**Step 2: clippy**
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: PASS.

**Step 3: tests**
Run: `cargo test --workspace`
Expected: PASS.

---

### Task 6: Update `progress.txt` and `lesson.md`

**Files:**
- Modify: `progress.txt`
- Modify: `lesson.md` (ONLY if there was debugging/iteration beyond normal TDD)

**Step 1: Append milestone progress**
Append to `progress.txt`:
- date/time
- bullets: added core domain models, added tests
- what’s next (import pipeline skeleton / parsing)

**Step 2: Debugging notes (conditional)**
If any non-trivial debugging occurred, append:
- symptom, root cause, attempts, fix, prevention

---

### Task 7: Commit (ONLY if requested)

If you explicitly ask for a commit and git is available:
- `git status`
- `git diff --stat`
- `git add ...`
- `git commit -m "feat(core): add minimal reader domain models"`
