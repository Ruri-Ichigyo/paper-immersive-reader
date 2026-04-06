# OCR JSON Persistence and Local Queue Implementation Plan

**Status:** Done
**Date:** 2026-04-05

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate OCR orchestration into a new application crate and add a JSON-backed local persistence layer that turns `OcrRequired` imports into persisted papers, queued jobs, and completed readable OCR results.

**Architecture:** Add `crates/application` as the business-layer crate for orchestration and persistence, leaving the root crate as thin glue only. Persist all local reader state in `database/app-state.json`, reuse importing text segmentation for OCR completion, and keep writes snapshot-based so OCR queue creation and OCR completion stay internally consistent.

**Tech Stack:** Rust, Cargo workspace, `serde`, `serde_json`, existing core/importing crates, integration tests in `tests/`

---

### Task 1: Create the application crate and migrate orchestration out of root `src/`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/application/Cargo.toml`
- Create: `crates/application/src/lib.rs`
- Create: `crates/application/src/orchestration/mod.rs`
- Create: `crates/application/src/orchestration/import_orchestrator.rs`
- Create: `crates/application/src/orchestration/ocr_job_service.rs`
- Create: `crates/application/src/orchestration/local_ocr_executor.rs`
- Create: `crates/application/src/orchestration/paddle_ocr_runner.rs`
- Modify: `src/lib.rs`
- Test: `tests/application_crate_linkable.rs`

**Step 1: Write the failing test**

Create `tests/application_crate_linkable.rs`:

```rust
#[test]
fn application_crate_exports_orchestration_types() {
    let _ = paper_immersive_reader_application::orchestration::ImportOrchestrationResult::ImportedText;
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test application_crate_linkable -q`

Expected: FAIL because `paper_immersive_reader_application` does not exist yet.

**Step 3: Write minimal implementation**

1. Add `crates/application` to the workspace in `Cargo.toml`.
2. Create `crates/application/Cargo.toml` with dependencies on:
- `paper-immersive-reader-core`
- `paper-immersive-reader-importing`
3. Move the current orchestration implementation from root `src/orchestration/` into `crates/application/src/orchestration/`.
4. Create `crates/application/src/lib.rs` exporting `pub mod orchestration;`.
5. Update root `src/lib.rs` so it stays thin and re-exports the application crate surface needed by existing tests.

**Step 4: Run test to verify it passes**

Run: `cargo test --test application_crate_linkable -q`

Expected: PASS.

### Task 2: Add reusable text segmentation for an existing paper

**Files:**
- Modify: `crates/importing/src/lib.rs`
- Test: `tests/import_existing_paper_text.rs`

**Step 1: Write the failing test**

Create `tests/import_existing_paper_text.rs`:

```rust
use paper_immersive_reader_core::domain::{Paper, PaperId, PaperSourceType, PaperStatus};
use paper_immersive_reader_importing::segment_text_for_existing_paper;

#[test]
fn existing_paper_text_segmentation_reuses_given_paper_id() {
    let paper = Paper {
        id: PaperId("paper-ocr-1".to_string()),
        title: Some("OCR paper".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    };

    let artifacts = segment_text_for_existing_paper(&paper, "first\n\nsecond".to_string(), 123)
        .expect("segmentation should succeed");

    assert_eq!(artifacts.blocks.len(), 2);
    assert_eq!(artifacts.blocks[0].paper_id, paper.id);
    assert_eq!(artifacts.reading_progress.paper_id, PaperId("paper-ocr-1".to_string()));
    assert_eq!(artifacts.reading_progress.updated_at_ms, 123);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test import_existing_paper_text -q`

Expected: FAIL because `segment_text_for_existing_paper` does not exist yet.

**Step 3: Write minimal implementation**

In `crates/importing/src/lib.rs`:
- add a small public result type for generated readable artifacts
- add `segment_text_for_existing_paper(&Paper, String, u64) -> Result<..., ImportPipelineError>`
- reuse the current segmentation and block/progress creation rules
- keep `DefaultImportPipeline` using the same shared helper so paste/PDF import behavior does not drift

**Step 4: Run test to verify it passes**

Run: `cargo test --test import_existing_paper_text -q`

Expected: PASS.

### Task 3: Add JSON app-state persistence primitives

**Files:**
- Modify: `crates/core/Cargo.toml`
- Modify: `crates/core/src/domain/block.rs`
- Modify: `crates/core/src/domain/error.rs`
- Modify: `crates/core/src/domain/ocr_job.rs`
- Modify: `crates/core/src/domain/paper.rs`
- Modify: `crates/core/src/domain/reading_progress.rs`
- Modify: `crates/application/Cargo.toml`
- Create: `crates/application/src/persistence/mod.rs`
- Create: `crates/application/src/persistence/json_app_state_store.rs`
- Modify: `crates/application/src/lib.rs`
- Test: `tests/json_app_state_store_smoke.rs`

**Step 1: Write the failing test**

Create `tests/json_app_state_store_smoke.rs`:

```rust
use paper_immersive_reader_application::persistence::{JsonAppStateStore, DEFAULT_APP_STATE_PATH};
use paper_immersive_reader_core::domain::{Paper, PaperId, PaperSourceType, PaperStatus};

#[test]
fn json_app_state_store_loads_missing_file_as_empty_and_round_trips_papers() {
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-json-app-state-smoke.json");
    let _ = std::fs::remove_file(&temp_path);

    let mut store = JsonAppStateStore::load_or_default(&temp_path).expect("store should load");
    assert!(store.snapshot().papers.is_empty());
    assert_eq!(DEFAULT_APP_STATE_PATH, "database/app-state.json");

    store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("persisted".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    });
    store.save().expect("store should save");

    let reloaded = JsonAppStateStore::load_or_default(&temp_path).expect("store should reload");
    assert_eq!(reloaded.snapshot().papers.len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test json_app_state_store_smoke -q`

Expected: FAIL because persistence types do not exist and domain types are not JSON-serializable yet.

**Step 3: Write minimal implementation**

1. Add `serde = { version = "...", features = ["derive"] }` to `crates/core/Cargo.toml`.
2. Derive `Serialize` and `Deserialize` for persisted domain types and enums used in app state.
3. Add `serde` and `serde_json` to `crates/application/Cargo.toml`.
4. Create `crates/application/src/persistence/json_app_state_store.rs` with:
- `pub const DEFAULT_APP_STATE_PATH: &str = "database/app-state.json";`
- serializable `AppState`
- `JsonAppStateStore::load_or_default(path)`
- `snapshot()` / `snapshot_mut()`
- `save()` using temp file + rename
5. Export `pub mod persistence;` from `crates/application/src/lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test --test json_app_state_store_smoke -q`

Expected: PASS.

### Task 4: Persist successful imports as real reader state

**Files:**
- Modify: `crates/application/src/orchestration/import_orchestrator.rs`
- Modify: `crates/application/src/persistence/json_app_state_store.rs`
- Test: `tests/import_orchestrator_persists_ready_text.rs`

**Step 1: Write the failing test**

Create `tests/import_orchestrator_persists_ready_text.rs`:

```rust
use paper_immersive_reader_application::orchestration::{ImportOrchestrationResult, ImportOrchestrator};
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportRequest};

#[test]
fn import_orchestrator_persists_embedded_text_import_result() {
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-import-success.json");
    let _ = std::fs::remove_file(&temp_path);

    let store = JsonAppStateStore::load_or_default(&temp_path).expect("store should load");
    let mut orchestrator = ImportOrchestrator::new_with_store(
        DefaultImportPipeline::new_for_tests_with_pdf_text("hello\n\nworld"),
        store,
    );

    let result = orchestrator
        .import(ImportRequest::PdfDocument {
            bytes: b"fake-pdf".to_vec(),
            title: Some("paper".to_string()),
            source_ref: Some("paper.pdf".to_string()),
        })
        .expect("import should succeed");

    assert_eq!(result, ImportOrchestrationResult::ImportedText);
    let snapshot = orchestrator.store().snapshot();
    assert_eq!(snapshot.papers.len(), 1);
    assert_eq!(snapshot.blocks.len(), 2);
    assert_eq!(snapshot.reading_progress.len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test import_orchestrator_persists_ready_text -q`

Expected: FAIL because the orchestrator does not persist successful imports yet.

**Step 3: Write minimal implementation**

1. Add a store-aware orchestrator constructor.
2. Add store methods to persist a successful `ImportResult`.
3. Update the success path so it writes `Paper`, `Block[]`, and `ReadingProgress` to the JSON snapshot before returning `ImportedText`.

**Step 4: Run test to verify it passes**

Run: `cargo test --test import_orchestrator_persists_ready_text -q`

Expected: PASS.

### Task 5: Persist `OcrRequired` as a real paper plus queued OCR job

**Files:**
- Modify: `crates/application/src/orchestration/import_orchestrator.rs`
- Modify: `crates/application/src/orchestration/ocr_job_service.rs`
- Modify: `crates/application/src/persistence/json_app_state_store.rs`
- Test: `tests/import_orchestrator_queues_ocr_persisted.rs`

**Step 1: Write the failing test**

Create `tests/import_orchestrator_queues_ocr_persisted.rs`:

```rust
use paper_immersive_reader_application::orchestration::{ImportOrchestrationResult, ImportOrchestrator, OcrJobService};
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_core::domain::{OcrJobId, OcrJobStatus, PaperId, PaperStatus};
use paper_immersive_reader_importing::{ImportPipeline, ImportPipelineError, ImportRequest};

struct OcrRequiredPipeline;

impl ImportPipeline for OcrRequiredPipeline {
    fn import(
        &self,
        _request: ImportRequest,
    ) -> Result<paper_immersive_reader_importing::ImportResult, ImportPipelineError> {
        Err(ImportPipelineError::OcrRequired)
    }
}

#[test]
fn import_orchestrator_persists_real_paper_and_queued_job_for_ocr() {
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-ocr-queued.json");
    let _ = std::fs::remove_file(&temp_path);

    let store = JsonAppStateStore::load_or_default(&temp_path).expect("store should load");
    let service = OcrJobService::new(
        || OcrJobId("job-1".to_string()),
        || PaperId("paper-1".to_string()),
        || 100,
    );
    let mut orchestrator = ImportOrchestrator::new_with_store_and_ocr_job_service(
        OcrRequiredPipeline,
        store,
        service,
    );

    let result = orchestrator
        .import(ImportRequest::PdfDocument {
            bytes: b"fake".to_vec(),
            title: Some("scan".to_string()),
            source_ref: Some("scan.pdf".to_string()),
        })
        .expect("should queue OCR");

    assert_eq!(
        result,
        ImportOrchestrationResult::QueuedOcr {
            paper_id: PaperId("paper-1".to_string()),
            job_id: OcrJobId("job-1".to_string()),
        }
    );

    let snapshot = orchestrator.store().snapshot();
    assert_eq!(snapshot.papers.len(), 1);
    assert_eq!(snapshot.papers[0].status, PaperStatus::ReadyOcr);
    assert_eq!(snapshot.ocr_jobs.len(), 1);
    assert_eq!(snapshot.ocr_jobs[0].status, OcrJobStatus::Queued);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test import_orchestrator_queues_ocr_persisted -q`

Expected: FAIL because the queue path does not currently persist a real paper plus queued job together.

**Step 3: Write minimal implementation**

1. Refactor `OcrJobService` so it creates job metadata without owning the repository.
2. Add store methods that atomically persist:
- `Paper(status = ReadyOcr)`
- `OcrJob(status = Queued)`
3. Update the store-aware orchestrator queue path to use a real paper id from the configured factory and persist both records together.

**Step 4: Run test to verify it passes**

Run: `cargo test --test import_orchestrator_queues_ocr_persisted -q`

Expected: PASS.

### Task 6: Persist OCR completion, failed runs, and next-job execution

**Files:**
- Modify: `crates/application/src/orchestration/local_ocr_executor.rs`
- Modify: `crates/application/src/persistence/json_app_state_store.rs`
- Modify: `crates/application/src/orchestration/mod.rs`
- Test: `tests/local_ocr_executor_persisted_success.rs`
- Test: `tests/local_ocr_executor_persisted_failure.rs`
- Test: `tests/local_ocr_executor_run_next_job.rs`

**Step 1: Write the failing success test**

Create `tests/local_ocr_executor_persisted_success.rs`:

```rust
use paper_immersive_reader_application::orchestration::{LocalOcrExecutor, PaddleOcrRunOutput, PaddleOcrRunner};
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_core::domain::{OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus};

struct SuccessRunner;

impl PaddleOcrRunner for SuccessRunner {
    fn run(&self, _job: &OcrJob) -> Result<PaddleOcrRunOutput, String> {
        Ok(PaddleOcrRunOutput { text: "first\n\nsecond".to_string() })
    }
}

#[test]
fn run_job_persists_completed_job_result_blocks_progress_and_ready_text_paper() {
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-ocr-success.json");
    let _ = std::fs::remove_file(&temp_path);
    let mut store = JsonAppStateStore::load_or_default(&temp_path).expect("store should load");
    store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("scan".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    });
    store.snapshot_mut().ocr_jobs.push(OcrJob {
        id: OcrJobId("job-1".to_string()),
        paper_id: PaperId("paper-1".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 1,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    });
    store.save().expect("seed state should save");

    let mut executor = LocalOcrExecutor::new(store, SuccessRunner, || 200);
    let completed = executor
        .run_job(&OcrJobId("job-1".to_string()))
        .expect("job should complete");

    assert_eq!(completed.status, OcrJobStatus::Completed);
    let snapshot = executor.store().snapshot();
    assert_eq!(snapshot.ocr_job_results.len(), 1);
    assert_eq!(snapshot.blocks.len(), 2);
    assert_eq!(snapshot.reading_progress.len(), 1);
    assert_eq!(snapshot.papers[0].status, PaperStatus::ReadyText);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test local_ocr_executor_persisted_success -q`

Expected: FAIL because executor success does not currently persist full readable state.

**Step 3: Write the failing failure/next-job tests**

Create `tests/local_ocr_executor_persisted_failure.rs` to verify runner or segmentation failure marks the job `Failed` and keeps the paper `ReadyOcr`.

Create `tests/local_ocr_executor_run_next_job.rs` to verify `run_next_job()` picks the first queued job from the JSON snapshot and completes it.

**Step 4: Run tests to verify they fail**

Run:
- `cargo test --test local_ocr_executor_persisted_failure -q`
- `cargo test --test local_ocr_executor_run_next_job -q`

Expected: FAIL because failure persistence and next-job selection are not implemented yet.

**Step 5: Write minimal implementation**

1. Add store methods to:
- fetch a job by id
- fetch the first queued job
- persist a failed job update
- atomically persist OCR completion artifacts for an existing paper
2. Update `LocalOcrExecutor::run_job(job_id)` to:
- load the queued job
- mark it `Processing`
- run OCR
- reuse `segment_text_for_existing_paper(...)`
- atomically persist completed job, OCR result, blocks, reading progress, and `Paper.status = ReadyText`
- mark the job `Failed` and keep the paper `ReadyOcr` if OCR or segmentation fails
3. Implement `run_next_job()` by selecting the first queued job from the store and delegating to `run_job(job_id)`.

**Step 6: Run tests to verify they pass**

Run:
- `cargo test --test local_ocr_executor_persisted_success -q`
- `cargo test --test local_ocr_executor_persisted_failure -q`
- `cargo test --test local_ocr_executor_run_next_job -q`

Expected: PASS.

### Task 7: Re-run regression coverage and full verification

**Files:**
- Modify: none unless verification exposes a narrow fix
- Test: existing workspace tests

**Step 1: Run focused regressions**

Run:
- `cargo test --test import_orchestrator_success -q`
- `cargo test --test import_pdf_real_text -q`
- `cargo test --test import_pdf_real_ocr_boundary -q`

Expected: PASS.

**Step 2: Run formatting check**

Run: `cargo fmt --check`

Expected: PASS.

**Step 3: Run clippy**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`

Expected: PASS.

**Step 4: Run the full test suite**

Run: `cargo test --workspace`

Expected: PASS.

### Task 8: Update repository tracking files

**Files:**
- Modify: `docs/plans/2026-04-05-ocr-json-persistence-local-queue-implementation.md`
- Modify: `docs/plans/INDEX.md`
- Modify: `progress.txt`
- Modify: `lesson.md` (if debugging or iteration occurred)

**Step 1: Mark the implementation document complete**

Update:

```md
**Status:** Done
```

after the milestone is fully implemented and verified.

**Step 2: Update the plans index**

Change the topic row status from `Planned` to `Done` after verification succeeds.

**Step 3: Append milestone progress**

Append a dated entry to `progress.txt` summarizing:
- application crate migration
- JSON persistence
- OCR queue + executor closed loop
- verification results

**Step 4: Append lessons if needed**

If any debugging or investigative iteration occurred, record it in `lesson.md`.
