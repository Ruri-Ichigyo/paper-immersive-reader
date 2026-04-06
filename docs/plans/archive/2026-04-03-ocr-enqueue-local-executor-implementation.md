# OCR Enqueue and Local Executor Implementation Plan

**Status:** Done
**Date:** 2026-04-03

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a thin orchestration layer that converts `OcrRequired` import outcomes into queued PaddleOCR jobs, plus a minimal local executor that processes those jobs without coupling OCR execution into the import pipeline.

**Architecture:** Keep `crates/importing` focused on import and `crates/core` focused on OCR job domain types. Add a small orchestration/service layer that bridges import to OCR queue creation, and a local executor layer that consumes queued jobs, calls a PaddleOCR runner boundary, and records completion or failure through repository operations.

**Tech Stack:** Rust, Cargo workspace, existing `paper-immersive-reader-core` and `paper-immersive-reader-importing` crates, integration tests in `/tests`, clippy/rustfmt

---

### Task 1: Add orchestration result types and a no-op orchestrator smoke test

**Files:**
- Create: `src/orchestration/mod.rs`
- Create: `src/orchestration/import_orchestrator.rs`
- Modify: `src/lib.rs`
- Test: `tests/import_orchestrator_smoke.rs`

**Step 1: Write the failing test**

Create `tests/import_orchestrator_smoke.rs`:

```rust
use paper_immersive_reader::orchestration::ImportOrchestrationResult;

#[test]
fn import_orchestration_result_type_is_exported() {
    let _ = ImportOrchestrationResult::QueuedOcr {
        paper_id: paper_immersive_reader_core::domain::PaperId("paper-1".to_string()),
        job_id: paper_immersive_reader_core::domain::OcrJobId("job-1".to_string()),
    };
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test import_orchestrator_smoke -q
```
Expected: FAIL because the orchestration module and result type do not exist yet.

**Step 3: Write minimal implementation**

Create `src/orchestration/import_orchestrator.rs` with:

```rust
use paper_immersive_reader_core::domain::{OcrJobId, PaperId};

pub enum ImportOrchestrationResult {
    ImportedText,
    QueuedOcr { paper_id: PaperId, job_id: OcrJobId },
}
```

Create `src/orchestration/mod.rs` re-exporting the type.
Update `src/lib.rs` to export `pub mod orchestration;`.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test import_orchestrator_smoke -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add src/lib.rs src/orchestration/mod.rs src/orchestration/import_orchestrator.rs tests/import_orchestrator_smoke.rs
git commit -m "Add OCR import orchestration result types"
```

### Task 2: Add import orchestrator success path

**Files:**
- Modify: `src/orchestration/import_orchestrator.rs`
- Test: `tests/import_orchestrator_success.rs`

**Step 1: Write the failing test**

Create `tests/import_orchestrator_success.rs`:

```rust
use paper_immersive_reader::orchestration::{ImportOrchestrationResult, ImportOrchestrator};
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportRequest};

#[test]
fn import_orchestrator_returns_imported_text_when_pipeline_succeeds() {
    let orchestrator = ImportOrchestrator::new(DefaultImportPipeline::new_for_tests_with_pdf_text("hello"));

    let result = orchestrator
        .import(ImportRequest::PdfDocument {
            bytes: b"%PDF fake".to_vec(),
            title: Some("paper".to_string()),
            source_ref: None,
        })
        .expect("orchestrator should succeed");

    assert!(matches!(result, ImportOrchestrationResult::ImportedText));
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test import_orchestrator_success -q
```
Expected: FAIL because `ImportOrchestrator` does not exist yet.

**Step 3: Write minimal implementation**

In `src/orchestration/import_orchestrator.rs`, add:
- `pub struct ImportOrchestrator<P> { pipeline: P }`
- `pub fn new(pipeline: P) -> Self`
- `pub fn import(&self, request: ImportRequest) -> Result<ImportOrchestrationResult, ImportPipelineError>`

Minimal behavior:
- if `pipeline.import(request)` succeeds -> return `ImportedText`
- if it fails with anything else for now -> return the same error unchanged

Do not create OCR jobs yet in this task.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test import_orchestrator_success -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add src/orchestration/import_orchestrator.rs tests/import_orchestrator_success.rs
git commit -m "Add import orchestrator success path"
```

### Task 3: Add OCR enqueue path through job service

**Files:**
- Modify: `src/orchestration/import_orchestrator.rs`
- Create: `src/orchestration/ocr_job_service.rs`
- Modify: `src/orchestration/mod.rs`
- Test: `tests/import_orchestrator_queues_ocr.rs`

**Step 1: Write the failing test**

Create `tests/import_orchestrator_queues_ocr.rs` with a simple in-memory repo and fake pipeline that returns `OcrRequired`:

```rust
use paper_immersive_reader::orchestration::{ImportOrchestrationResult, ImportOrchestrator, OcrJobService};
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobRepository, OcrJobResult, OcrJobStatus, PaperId,
};
use paper_immersive_reader_importing::{ImportPipeline, ImportPipelineError, ImportRequest, ImportResult};

struct OcrRequiredPipeline;

impl ImportPipeline for OcrRequiredPipeline {
    fn import(&self, _request: ImportRequest) -> Result<ImportResult, ImportPipelineError> {
        Err(ImportPipelineError::OcrRequired)
    }
}

struct MemoryRepo {
    jobs: Vec<OcrJob>,
}

impl OcrJobRepository for MemoryRepo {
    fn create_job(&mut self, job: OcrJob) { self.jobs.push(job); }
    fn get_job(&self, id: &OcrJobId) -> Option<&OcrJob> { self.jobs.iter().find(|job| &job.id == id) }
    fn list_jobs_for_paper(&self, paper_id: &PaperId) -> Vec<&OcrJob> { self.jobs.iter().filter(|job| &job.paper_id == paper_id).collect() }
    fn save_result(&mut self, _result: OcrJobResult) {}
}

#[test]
fn import_orchestrator_creates_queued_paddleocr_job_when_import_requires_ocr() {
    let repo = MemoryRepo { jobs: Vec::new() };
    let service = OcrJobService::new(repo);
    let orchestrator = ImportOrchestrator::new_with_ocr_job_service(OcrRequiredPipeline, service);

    let result = orchestrator
        .import(ImportRequest::PdfDocument {
            bytes: b"%PDF fake".to_vec(),
            title: Some("paper".to_string()),
            source_ref: None,
        })
        .expect("orchestrator should queue OCR");

    match result {
        ImportOrchestrationResult::QueuedOcr { paper_id: _, job_id: _ } => {}
        _ => panic!("expected queued OCR result"),
    }
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test import_orchestrator_queues_ocr -q
```
Expected: FAIL because `OcrJobService` and queue path do not exist yet.

**Step 3: Write minimal implementation**

Create `src/orchestration/ocr_job_service.rs` with a thin generic service over `OcrJobRepository`.

Minimum service behavior:
- create a new `OcrJob` with:
  - `engine = OcrEngine::PaddleOcr`
  - `status = OcrJobStatus::Queued`
  - stable deterministic IDs/timestamps for tests

Extend `ImportOrchestrator` with a constructor that includes the OCR job service, and update `import(...)` so:
- if pipeline succeeds -> `ImportedText`
- if pipeline returns `ImportPipelineError::OcrRequired` -> create job and return `QueuedOcr { paper_id, job_id }`
- other errors remain errors

Keep the implementation minimal and do not infer extra metadata.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test import_orchestrator_queues_ocr -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add src/orchestration/import_orchestrator.rs src/orchestration/ocr_job_service.rs src/orchestration/mod.rs tests/import_orchestrator_queues_ocr.rs
git commit -m "Queue PaddleOCR jobs from OcrRequired imports"
```

### Task 4: Add local executor and runner boundary

**Files:**
- Create: `src/orchestration/local_ocr_executor.rs`
- Create: `src/orchestration/paddle_ocr_runner.rs`
- Modify: `src/orchestration/mod.rs`
- Test: `tests/local_ocr_executor_success.rs`

**Step 1: Write the failing test**

Create `tests/local_ocr_executor_success.rs` with a fake runner and in-memory repo seeded with one queued job.

The test should verify:
- executor runs one queued job
- job becomes `Completed`
- OCR result is saved

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test local_ocr_executor_success -q
```
Expected: FAIL because executor/runner types do not exist yet.

**Step 3: Write minimal implementation**

Create a small runner trait or boundary in `src/orchestration/paddle_ocr_runner.rs`:

```rust
pub trait PaddleOcrRunner {
    fn run(&self, paper_id: &paper_immersive_reader_core::domain::PaperId) -> Result<String, String>;
}
```

Create `LocalOcrExecutor` in `src/orchestration/local_ocr_executor.rs` with minimal synchronous behavior:
- `run_job(job_id)` or `run_next_job()`
- mark queued job processing
- call runner
- on success:
  - save `OcrJobResult`
  - mark job completed
- on failure:
  - mark job failed with `error_message`

Keep the implementation small and local. No background threads.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test local_ocr_executor_success -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add src/orchestration/local_ocr_executor.rs src/orchestration/paddle_ocr_runner.rs src/orchestration/mod.rs tests/local_ocr_executor_success.rs
git commit -m "Add local OCR executor success path"
```

### Task 5: Add executor failure and invalid-state coverage

**Files:**
- Test: `tests/local_ocr_executor_failure.rs`
- Test: `tests/local_ocr_executor_invalid_state.rs`
- Modify: `src/orchestration/local_ocr_executor.rs`

**Step 1: Write the failing failure-path test**

Create `tests/local_ocr_executor_failure.rs` asserting:
- queued job enters processing
- runner failure ends as `Failed`
- `error_message` is saved

Run:
```bash
cargo test --test local_ocr_executor_failure -q
```
Expected: FAIL until failure handling exists.

**Step 2: Write the failing invalid-state test**

Create `tests/local_ocr_executor_invalid_state.rs` asserting:
- executing a non-existent job fails
- executing a completed/failed job fails

Run:
```bash
cargo test --test local_ocr_executor_invalid_state -q
```
Expected: FAIL until invalid execution handling exists.

**Step 3: Minimal implementation adjustments**

Update `LocalOcrExecutor` to:
- return a small executor error for job-not-found and invalid-state cases
- mark failed jobs with `error_message` on runner error

Do not add retries or cancellation.

**Step 4: Run both tests to verify they pass**

Run:
```bash
cargo test --test local_ocr_executor_failure -q
cargo test --test local_ocr_executor_invalid_state -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add src/orchestration/local_ocr_executor.rs tests/local_ocr_executor_failure.rs tests/local_ocr_executor_invalid_state.rs
git commit -m "Add OCR executor failure handling"
```

### Task 6: Run repository verification

**Files:**
- Modify: none unless a narrow follow-up fix is needed
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

**Step 4: Commit only if a narrow follow-up fix was needed**

If verification requires a small targeted fix, pair it with that exact change.

### Task 7: Update milestone tracking

**Files:**
- Modify: `progress.txt`
- Modify: `lesson.md` (only if non-trivial debugging occurred)

**Step 1: Update progress log**

Append a milestone entry describing:
- import orchestration now queues PaddleOCR jobs when import returns `OcrRequired`
- local executor and runner boundary added
- success/failure execution paths covered by tests
- verification commands run

**Step 2: Update debugging retrospective only if needed**

If implementation required real debugging beyond normal TDD, append a short retrospective.

**Step 3: Verify tracking changes**

Run:
```bash
cargo test --workspace
```
Expected: PASS.

**Step 4: Commit**

```bash
git add progress.txt lesson.md
git commit -m "Record OCR enqueue and executor milestone progress"
```

### Task 8: Stop unless the user asks to execute the plan

**Files:**
- Modify: none

**Step 1: Do not start implementation automatically**

Stop after saving this plan.

**Step 2: Offer execution choice**

After saving this plan, tell the user:

Plan complete and saved to `docs/plans/2026-04-03-ocr-enqueue-local-executor-implementation.md`. Two execution options:

1. Subagent-Driven (this session) - I dispatch fresh subagent per task, review between tasks, fast iteration
2. Parallel Session (separate) - Open new session with executing-plans, batch execution with checkpoints

Which approach?
