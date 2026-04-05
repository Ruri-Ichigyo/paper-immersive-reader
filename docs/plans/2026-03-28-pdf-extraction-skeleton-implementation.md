# PDF Extraction Skeleton Implementation Plan

**Status:** Done
**Date:** 2026-03-28

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a minimal PDF import path in `crates/importing` that supports embedded-text extraction through a seam and returns a structured OCR-required boundary when usable text is unavailable.

**Architecture:** `crates/core` stays domain-only. `crates/importing` gains PDF request handling, a `PdfTextExtractor` seam, and PDF-specific error handling while continuing to produce `Paper`, `Block`, and `ReadingProgress`. Real OCR and real PDF parser integration are deferred.

**Tech Stack:** Rust, Cargo workspace, integration tests under `tests/`, existing `paper-immersive-reader-core` domain models, existing `paper-immersive-reader-importing` crate.

---

### Task 1: Add PDF request/API smoke coverage

**Files:**
- Modify: `tests/import_types_smoke.rs`
- Test: `tests/import_types_smoke.rs`

**Step 1: Write the failing test**

Extend the existing smoke test so it references the new PDF request variant and PDF-specific error variants:

```rust
use paper_immersive_reader_importing::{ImportPipelineError, ImportRequest};

#[test]
fn importing_types_exist() {
    let _pdf = ImportRequest::PdfDocument {
        bytes: vec![1, 2, 3],
        title: Some("paper".to_string()),
        source_ref: None,
    };

    let _ = ImportPipelineError::PdfExtractionFailed;
    let _ = ImportPipelineError::OcrRequired;
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test import_types_smoke -q
```
Expected: FAIL because `PdfDocument`, `PdfExtractionFailed`, and `OcrRequired` do not exist yet.

**Step 3: Write minimal implementation**

Modify `crates/importing/src/lib.rs` to add:
- `ImportRequest::PdfDocument { bytes, title, source_ref }`
- `ImportPipelineError::PdfExtractionFailed`
- `ImportPipelineError::OcrRequired`

Do not implement real PDF behavior yet.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test import_types_smoke -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/import_types_smoke.rs crates/importing/src/lib.rs
git commit -m "Add PDF import request types"
```

### Task 2: Add PDF extractor seam with a deterministic test extractor

**Files:**
- Modify: `crates/importing/src/lib.rs`
- Test: `tests/import_pdf_smoke.rs`

**Step 1: Write the failing test**

Create `tests/import_pdf_smoke.rs` covering the happy path through a deterministic extractor seam:

```rust
use paper_immersive_reader_core::domain::{PaperSourceType, PaperStatus};
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportPipeline, ImportRequest};

#[test]
fn pdf_import_with_extracted_text_produces_ready_text_result() {
    let pipeline = DefaultImportPipeline::new_for_tests_with_pdf_text("first block\n\nsecond block");

    let result = pipeline
        .import(ImportRequest::PdfDocument {
            bytes: vec![37, 80, 68, 70],
            title: Some("paper".to_string()),
            source_ref: None,
        })
        .expect("pdf import should succeed");

    assert_eq!(result.paper.source_type, PaperSourceType::Pdf);
    assert_eq!(result.paper.status, PaperStatus::ReadyText);
    assert_eq!(result.blocks.len(), 2);
    assert_eq!(result.reading_progress.current_block_index, 0);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test import_pdf_smoke -q
```
Expected: FAIL because the seam / constructor does not exist yet.

**Step 3: Write minimal implementation**

In `crates/importing/src/lib.rs`:
- add a `PdfTextExtractor` trait
- add a deterministic extractor mode usable from tests
- add `DefaultImportPipeline::new_for_tests_with_pdf_text(&str)`
- route `ImportRequest::PdfDocument` through the seam
- reuse the same segmentation rule used for paste text
- return `PaperSourceType::Pdf` and `PaperStatus::ReadyText`

Keep the implementation minimal and deterministic.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test import_pdf_smoke -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/import_pdf_smoke.rs crates/importing/src/lib.rs
git commit -m "Add PDF import happy path through extractor seam"
```

### Task 3: Add OCR-required boundary coverage

**Files:**
- Create: `tests/import_pdf_ocr_boundary.rs`
- Modify: `crates/importing/src/lib.rs`
- Test: `tests/import_pdf_ocr_boundary.rs`

**Step 1: Write the failing test**

Create a test for empty extracted text:

```rust
use paper_immersive_reader_importing::{
    DefaultImportPipeline, ImportPipeline, ImportPipelineError, ImportRequest,
};

#[test]
fn pdf_import_with_no_usable_text_requests_ocr() {
    let pipeline = DefaultImportPipeline::new_for_tests_with_pdf_text("   ");

    let err = pipeline
        .import(ImportRequest::PdfDocument {
            bytes: vec![37, 80, 68, 70],
            title: None,
            source_ref: None,
        })
        .expect_err("should request OCR");

    assert_eq!(err, ImportPipelineError::OcrRequired);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test import_pdf_ocr_boundary -q
```
Expected: FAIL because the PDF path does not yet return `OcrRequired` for unusable text.

**Step 3: Write minimal implementation**

Update `crates/importing/src/lib.rs` so the PDF path:
- trims extracted text
- segments it
- returns `ImportPipelineError::OcrRequired` when no usable text is available

Do not add OCR execution.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test import_pdf_ocr_boundary -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/import_pdf_ocr_boundary.rs crates/importing/src/lib.rs
git commit -m "Add OCR boundary for PDF import"
```

### Task 4: Add extractor failure mapping coverage

**Files:**
- Create: `tests/import_pdf_failure.rs`
- Modify: `crates/importing/src/lib.rs`
- Test: `tests/import_pdf_failure.rs`

**Step 1: Write the failing test**

Create a test for extractor failure:

```rust
use paper_immersive_reader_importing::{
    DefaultImportPipeline, ImportPipeline, ImportPipelineError, ImportRequest,
};

#[test]
fn pdf_extractor_failure_maps_to_pdf_extraction_failed() {
    let pipeline = DefaultImportPipeline::new_for_tests_with_pdf_failure();

    let err = pipeline
        .import(ImportRequest::PdfDocument {
            bytes: vec![37, 80, 68, 70],
            title: None,
            source_ref: None,
        })
        .expect_err("should fail cleanly");

    assert_eq!(err, ImportPipelineError::PdfExtractionFailed);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test import_pdf_failure -q
```
Expected: FAIL because the failure path does not yet exist.

**Step 3: Write minimal implementation**

Add deterministic failure support for tests and map seam failures to `ImportPipelineError::PdfExtractionFailed`.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test import_pdf_failure -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/import_pdf_failure.rs crates/importing/src/lib.rs
git commit -m "Add PDF extraction failure mapping"
```

### Task 5: Run repo verification

**Files:**
- Modify: none
- Test: full workspace

**Step 1: Run formatting check**

Run:
```bash
cargo fmt --check
```
Expected: PASS.

**Step 2: Run clippy**

Run:
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
Expected: PASS.

**Step 3: Run full test suite**

Run:
```bash
cargo test --workspace
```
Expected: PASS.

**Step 4: Commit**

No commit here unless verification requires a small follow-up fix. If fixes are needed, pair them with the failing test or affected implementation commit.

### Task 6: Update tracking files

**Files:**
- Modify: `progress.txt`
- Modify: `lesson.md` (only if actual debugging happened)

**Step 1: Update milestone progress**

Append a new milestone note describing:
- PDF import request support
- extractor seam
- OCR-required boundary
- verification commands run

**Step 2: Update debugging retrospective if needed**

Only append to `lesson.md` if a real debugging/iteration cycle occurred.

**Step 3: Verify tracking files are correct**

Run:
```bash
cargo test --workspace
```
Expected: still PASS.

**Step 4: Commit**

```bash
git add progress.txt lesson.md
git commit -m "Record PDF extraction milestone progress"
```

### Task 7: Optional commit hygiene / push

**Files:**
- Modify: none unless requested

**Step 1: Stop unless explicitly requested**

Do not create extra git history cleanup or push changes unless explicitly requested.

**Step 2: If requested later**

Use the repository's established commit style and verify before pushing.
