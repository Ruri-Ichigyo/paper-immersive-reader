# Real PDF Text Extraction Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the fake PDF text extractor with a real embedded-text extraction path in `crates/importing` while keeping the current PDF import orchestration and OCR boundary behavior simple.

**Architecture:** Keep `DefaultImportPipeline` and the `PdfTextExtractor` seam in `crates/importing`. Add the real extractor implementation in a dedicated PDF module inside the importing crate, then route extracted text back through the existing shared text-import path so PDF and paste imports continue to share segmentation, block construction, and progress creation.

**Tech Stack:** Rust, Cargo workspace, `paper-immersive-reader-importing`, existing core domain types, integration tests in `tests/`, a real PDF text extraction library selected during implementation.

---

### Task 1: Add a real-PDF fixture happy-path test

**Files:**
- Create: `tests/import_pdf_real_text.rs`
- Create: `tests/fixtures/pdf/embedded-text-minimal.pdf`
- Test: `tests/import_pdf_real_text.rs`

**Step 1: Write the failing test**

Create `tests/import_pdf_real_text.rs`:

```rust
use paper_immersive_reader_core::domain::{PaperSourceType, PaperStatus};
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportPipeline, ImportRequest};

#[test]
fn pdf_import_with_real_embedded_text_produces_ready_text_result() {
    let pdf_bytes = std::fs::read("tests/fixtures/pdf/embedded-text-minimal.pdf")
        .expect("fixture pdf should exist");

    let pipeline = DefaultImportPipeline::new();

    let result = pipeline
        .import(ImportRequest::PdfDocument {
            bytes: pdf_bytes,
            title: Some("fixture".to_string()),
            source_ref: None,
        })
        .expect("pdf import should succeed");

    assert_eq!(result.paper.source_type, PaperSourceType::Pdf);
    assert_eq!(result.paper.status, PaperStatus::ReadyText);
    assert!(!result.blocks.is_empty());
    assert_eq!(result.reading_progress.current_block_index, 0);
}
```

Use a small stable fixture PDF that contains embedded text split into at least one usable block.

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test import_pdf_real_text -q
```
Expected: FAIL because `DefaultImportPipeline::new()` or a real PDF extractor path does not exist yet.

**Step 3: Add the fixture file**

Create `tests/fixtures/pdf/embedded-text-minimal.pdf` as a repository fixture with embedded selectable text.

**Step 4: Commit**

```bash
git add tests/import_pdf_real_text.rs tests/fixtures/pdf/embedded-text-minimal.pdf
git commit -m "Add real PDF fixture happy-path test"
```

### Task 2: Add a real extractor module and wire a default pipeline constructor

**Files:**
- Modify: `crates/importing/Cargo.toml`
- Modify: `crates/importing/src/lib.rs`
- Create: `crates/importing/src/pdf.rs`
- Test: `tests/import_pdf_real_text.rs`

**Step 1: Add the real extraction dependency**

Choose one Rust PDF text extraction library and add it to `crates/importing/Cargo.toml`.

Keep the dependency narrowly scoped to embedded-text extraction only.

**Step 2: Write the dedicated PDF module**

Create `crates/importing/src/pdf.rs` with a real extractor implementation that satisfies the existing seam:

```rust
use crate::{ImportPipelineError, PdfTextExtractor};

pub struct RealPdfTextExtractor;

impl PdfTextExtractor for RealPdfTextExtractor {
    fn extract_text(&self, bytes: &[u8]) -> Result<String, ImportPipelineError> {
        // call real PDF extraction library here
        // return extracted embedded text as String
    }
}
```

Do not add OCR logic. Only extract embedded text.

**Step 3: Wire the default constructor**

Modify `crates/importing/src/lib.rs` to:
- `mod pdf;`
- expose `DefaultImportPipeline::new() -> Self`
- use `pdf::RealPdfTextExtractor` inside `new()`
- keep existing test constructors (`new_for_tests`, `new_for_tests_with_pdf_text`, `new_for_tests_with_pdf_failure`) intact

**Step 4: Run the happy-path test to verify it passes**

Run:
```bash
cargo test --test import_pdf_real_text -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/importing/Cargo.toml crates/importing/src/lib.rs crates/importing/src/pdf.rs tests/import_pdf_real_text.rs
git commit -m "Add real embedded text PDF extractor"
```

### Task 3: Add the real-path OCR boundary test

**Files:**
- Create: `tests/import_pdf_real_ocr_boundary.rs`
- Create: `tests/fixtures/pdf/no-usable-text.pdf`
- Modify: `crates/importing/src/lib.rs` (only if needed)
- Test: `tests/import_pdf_real_ocr_boundary.rs`

**Step 1: Write the failing test**

Create `tests/import_pdf_real_ocr_boundary.rs`:

```rust
use paper_immersive_reader_importing::{
    DefaultImportPipeline, ImportPipeline, ImportPipelineError, ImportRequest,
};

#[test]
fn pdf_import_with_no_usable_real_text_requests_ocr() {
    let pdf_bytes = std::fs::read("tests/fixtures/pdf/no-usable-text.pdf")
        .expect("fixture pdf should exist");

    let pipeline = DefaultImportPipeline::new();

    let err = pipeline
        .import(ImportRequest::PdfDocument {
            bytes: pdf_bytes,
            title: None,
            source_ref: None,
        })
        .expect_err("should request OCR");

    assert_eq!(err, ImportPipelineError::OcrRequired);
}
```

Use a real fixture representing one of:
- image-only / scanned PDF with no embedded text
- PDF where the selected library yields no usable text

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test import_pdf_real_ocr_boundary -q
```
Expected: FAIL until the real extractor path and no-usable-text mapping are aligned.

**Step 3: Make the minimal implementation adjustment**

Ensure the real extractor path + shared PDF import flow maps all “no usable text” outcomes to `ImportPipelineError::OcrRequired`.

Do not reintroduce a separate public `PdfExtractionFailed` branch for the real path in this milestone.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test import_pdf_real_ocr_boundary -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/import_pdf_real_ocr_boundary.rs tests/fixtures/pdf/no-usable-text.pdf crates/importing/src/lib.rs
git commit -m "Map real PDF no-text cases to OCR boundary"
```

### Task 4: Keep seam-level regression coverage aligned

**Files:**
- Modify: `tests/import_pdf_smoke.rs`
- Modify: `tests/import_pdf_ocr_boundary.rs`
- Modify: `tests/import_pdf_failure.rs`
- Modify: `tests/import_types_smoke.rs`
- Test: existing PDF seam tests

**Step 1: Re-read the seam tests**

Confirm which tests remain useful as fast seam-level coverage after the real extractor is added.

**Step 2: Trim or update only what is necessary**

Keep seam tests that still provide value:
- injected extractor happy path
- explicit seam failure mapping, if still meaningful for test-only constructors
- placeholder/type-surface coverage

Remove or rewrite any assertion that conflicts with the new real default constructor behavior.

**Step 3: Run the seam-level PDF tests**

Run:
```bash
cargo test --test import_pdf_smoke -q
cargo test --test import_pdf_ocr_boundary -q
cargo test --test import_pdf_failure -q
cargo test --test import_types_smoke -q
```
Expected: PASS.

**Step 4: Commit**

```bash
git add tests/import_pdf_smoke.rs tests/import_pdf_ocr_boundary.rs tests/import_pdf_failure.rs tests/import_types_smoke.rs
git commit -m "Align PDF seam tests with real extractor path"
```

### Task 5: Run repository verification

**Files:**
- Modify: none unless a verification failure requires a narrow fix
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

**Step 3: Run the full test suite**

Run:
```bash
cargo test --workspace
```
Expected: PASS.

**Step 4: Commit only if a narrow follow-up fix was needed**

If verification required a small corrective edit, pair the commit with that exact fix.

### Task 6: Update milestone tracking

**Files:**
- Modify: `progress.txt`
- Modify: `lesson.md` (only if non-trivial debugging actually occurred)

**Step 1: Update progress log**

Append a new milestone entry describing:
- real embedded-text PDF extraction wired into the importing crate
- real PDF fixtures added
- OCR boundary preserved for no-usable-text cases
- verification commands run

**Step 2: Update debugging retrospective only if needed**

If implementation required real debugging beyond normal TDD, append:
- symptom
- root cause
- fix
- prevention

**Step 3: Verify tracking update does not break the repo**

Run:
```bash
cargo test --workspace
```
Expected: PASS.

**Step 4: Commit**

```bash
git add progress.txt lesson.md
git commit -m "Record real PDF extraction milestone progress"
```

### Task 7: Stop unless the user asks to execute the plan

**Files:**
- Modify: none

**Step 1: Do not start implementation automatically**

Stop after the implementation plan is saved.

**Step 2: Offer execution choice**

After saving this plan, tell the user:

Plan complete and saved to `docs/plans/2026-03-31-real-pdf-text-extraction-implementation.md`. Two execution options:

1. Subagent-Driven (this session) - I dispatch fresh subagent per task, review between tasks, fast iteration
2. Parallel Session (separate) - Open new session with executing-plans, batch execution with checkpoints

Which approach?
