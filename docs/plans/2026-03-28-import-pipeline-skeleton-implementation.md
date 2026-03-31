# Import Pipeline Skeleton (M1.2 — Minimal Runnable Paste Text Import) Implementation Plan

**Status:** Planned
**Date:** 2026-03-28

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a new `crates/importing` crate that defines an import pipeline abstraction and a runnable `paste text` import path producing `Paper`, `Block`, and `ReadingProgress` from `paper-immersive-reader-core`.

**Architecture:** Keep `crates/core` free of import logic. Implement a small importing crate that depends on core domain. Use root integration tests in `/tests` for behavior. Focus on a single runnable path: `PasteText`.

**Tech Stack:** Rust, Cargo workspace, clippy/rustfmt, root integration tests.

---

### Task 1: Add importing crate to workspace

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/importing/Cargo.toml`
- Create: `crates/importing/src/lib.rs`

**Step 1: Write the failing test (import crate linkable)**

Create `tests/import_crate_linkable.rs`:

```rust
#[test]
fn importing_crate_is_linkable() {
    let _ = paper_immersive_reader_importing::ImportPipelineError::EmptyInput;
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test import_crate_linkable -q`

Expected: FAIL because the crate doesn’t exist yet.

**Step 3: Create crate skeleton**

1) Update root `Cargo.toml` workspace members to include:
- `crates/importing`

2) Create `crates/importing/Cargo.toml`:

```toml
[package]
name = "paper-immersive-reader-importing"
version = "0.1.0"
edition = "2021"

[dependencies]
paper-immersive-reader-core = { path = "../core" }
```

3) Create `crates/importing/src/lib.rs` exporting at least:
- `pub enum ImportPipelineError { EmptyInput }`

**Step 4: Run test to verify it passes**

Run: `cargo test --test import_crate_linkable -q`

Expected: PASS.

---

### Task 2: Define import request/result/error types and pipeline trait

**Files:**
- Modify: `crates/importing/src/lib.rs`
- Test: `tests/import_types_smoke.rs`

**Step 1: Write the failing test**

Create `tests/import_types_smoke.rs`:

```rust
use paper_immersive_reader_importing::{ImportRequest, ImportResult, ImportPipeline};

#[test]
fn importing_types_exist() {
    let _req = ImportRequest::PasteText {
        text: "hello".to_string(),
        title: None,
        source_ref: None,
    };

    // Type-level check only
    fn _assert_pipeline<T: ImportPipeline>(_t: &T) {}

    // ImportResult must be name-resolvable
    let _maybe: Option<ImportResult> = None;
    assert!(_maybe.is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test import_types_smoke -q`

Expected: FAIL because the types/trait don’t exist.

**Step 3: Implement minimal types/trait**

In `crates/importing/src/lib.rs`, implement:

```rust
use paper_immersive_reader_core::domain::{Block, DomainError, Paper, ReadingProgress};

pub enum ImportRequest {
    PasteText {
        text: String,
        title: Option<String>,
        source_ref: Option<String>,
    },
}

pub struct ImportResult {
    pub paper: Paper,
    pub blocks: Vec<Block>,
    pub reading_progress: ReadingProgress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportPipelineError {
    EmptyInput,
    NoBlocksProduced,
    Domain(DomainError),
}

pub trait ImportPipeline {
    fn import(&self, request: ImportRequest) -> Result<ImportResult, ImportPipelineError>;
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test import_types_smoke -q`

Expected: PASS.

---

### Task 3: Implement runnable paste text importer (happy path)

**Files:**
- Modify: `crates/importing/src/lib.rs`
- Test: `tests/import_paste_smoke.rs`

**Step 1: Write the failing test**

Create `tests/import_paste_smoke.rs`:

```rust
use paper_immersive_reader_core::domain::{PaperSourceType, PaperStatus};
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportPipeline, ImportRequest};

#[test]
fn paste_text_import_produces_paper_blocks_and_progress() {
    let pipeline = DefaultImportPipeline::new_for_tests();

    let result = pipeline
        .import(ImportRequest::PasteText {
            text: "p1\n\n p2".to_string(),
            title: Some("t".to_string()),
            source_ref: None,
        })
        .expect("import should succeed");

    assert_eq!(result.paper.source_type, PaperSourceType::Paste);
    assert_eq!(result.paper.status, PaperStatus::ReadyText);
    assert!(result.blocks.len() >= 2);
    assert_eq!(result.blocks[0].order_index, 0);
    assert_eq!(result.reading_progress.current_block_index, 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test import_paste_smoke -q`

Expected: FAIL because `DefaultImportPipeline` is missing.

**Step 3: Implement minimal pipeline**

In `crates/importing/src/lib.rs`:
- Add a `pub struct DefaultImportPipeline { ... }`
- Provide constructors:
  - `pub fn new_for_tests() -> Self` (keep deterministic ID strategy)
- Implement `ImportPipeline` for it.

Minimal behavior for `PasteText`:
- Reject if `text.trim().is_empty()` -> `EmptyInput`
- Split into segments by `"\n\n"`
- Trim segments, drop empties
- If no segments -> `NoBlocksProduced`
- Create `Paper`:
  - `source_type = Paste`
  - `status = ReadyText`
  - `title` and `source_ref` set from request
  - `parse_version = 1`
  - generate `PaperId` deterministically for tests (e.g. constant or counter)
- Create blocks using `Block::try_new` and map `DomainError` to `ImportPipelineError::Domain`
- Create `ReadingProgress`:
  - `current_block_index = 0`
  - `current_block_id = Some(blocks[0].id.clone())`
  - `updated_at_ms = 0` (deterministic for tests)

**Step 4: Run test to verify it passes**

Run: `cargo test --test import_paste_smoke -q`

Expected: PASS.

---

### Task 4: Segmentation and empty-input tests

**Files:**
- Test: `tests/import_paste_segmentation.rs`
- Test: `tests/import_paste_empty.rs`

**Step 1: Write failing segmentation test**

Create `tests/import_paste_segmentation.rs`:

```rust
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportPipeline, ImportRequest};

#[test]
fn paste_text_splits_on_double_newline_and_drops_empty_segments() {
    let pipeline = DefaultImportPipeline::new_for_tests();

    let result = pipeline
        .import(ImportRequest::PasteText {
            text: "a\n\n\n\n b\n\n  \n\n c".to_string(),
            title: None,
            source_ref: None,
        })
        .expect("import should succeed");

    assert_eq!(result.blocks.len(), 3);
}
```

Run: `cargo test --test import_paste_segmentation -q`
Expected: FAIL until segmentation handles empties.

**Step 2: Write failing empty-input test**

Create `tests/import_paste_empty.rs`:

```rust
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportPipeline, ImportPipelineError, ImportRequest};

#[test]
fn paste_text_empty_input_is_rejected() {
    let pipeline = DefaultImportPipeline::new_for_tests();

    let err = pipeline
        .import(ImportRequest::PasteText {
            text: "   ".to_string(),
            title: None,
            source_ref: None,
        })
        .expect_err("should fail");

    assert_eq!(err, ImportPipelineError::EmptyInput);
}
```

Run: `cargo test --test import_paste_empty -q`
Expected: FAIL until error mapping is correct.

**Step 3: Minimal implementation tweaks**
- Adjust segmentation and validation until both tests pass.

**Step 4: Verify both pass**
- `cargo test --test import_paste_segmentation -q`
- `cargo test --test import_paste_empty -q`

---

### Task 5: Repository verification

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

### Task 6: Progress tracking

**Files:**
- Modify: `progress.txt`
- Modify: `lesson.md` (ONLY if debugging/iteration beyond normal TDD occurred)

**Step 1: Append milestone entry**
Add a new entry noting:
- Added importing crate skeleton
- Paste text import runnable
- New integration tests
- Next: PDF importer skeleton, parsing heuristics, persistence wiring

---

### Task 7: Commit (ONLY if requested and git is available)

If you explicitly ask for a commit and `git` works in this environment:
- `git status`
- `git diff --stat`
- `git add Cargo.toml crates/importing tests progress.txt lesson.md docs/plans/2026-03-28-import-pipeline-skeleton-*.md`
- `git commit -m "feat(import): add minimal runnable import pipeline skeleton"`

Otherwise, skip.
