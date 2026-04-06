# OCR JSON Persistence and Local Queue Design

**Status:** Approved
**Date:** 2026-04-05

## Goal
Close the current OCR gap by persisting real paper records and queued OCR jobs to a local JSON file under `database/`, then letting a local executor consume queued jobs and turn OCR output into persisted readable content.

## Architecture
Move the current orchestration layer out of the root `src/` tree and into a new business crate: `crates/application`. The root crate remains a thin workspace harness and export surface only.

Use a single JSON snapshot file at `database/app-state.json` as the local persistence boundary for this milestone. The application crate owns the JSON persistence implementation and coordinates import success, OCR queue creation, and OCR completion updates, while `crates/core` stays domain-focused and `crates/importing` stays responsible for text extraction and text-to-block segmentation.

## Scope
- Create a new `crates/application` crate and migrate existing orchestration code into it.
- Keep the root `src/` crate as thin glue only.
- Persist successful embedded-text imports as real `Paper`, `Block`, and `ReadingProgress` records.
- Persist `OcrRequired` imports as a real `Paper(status = ReadyOcr)` plus a queued `OcrJob`.
- Persist OCR execution results to `database/app-state.json`.
- Add `run_next_job()` so the local executor can consume the next queued job from persisted state.
- Reuse importing logic to convert OCR text into `Block` and `ReadingProgress` for an existing `Paper`.

## Non-Goals
- No SQLite or other database engine.
- No background polling or daemon worker.
- No file locking or multi-process concurrency control.
- No UI or CLI workflow for OCR execution.
- No remote OCR service integration.
- No sync or Zotero work.

## Storage Model
The JSON database is a single application snapshot stored at:

- `database/app-state.json`

The snapshot contains:

- `papers: Vec<Paper>`
- `blocks: Vec<Block>`
- `reading_progress: Vec<ReadingProgress>`
- `ocr_jobs: Vec<OcrJob>`
- `ocr_job_results: Vec<OcrJobResult>`

The store writes by building a full next-state snapshot, writing it to a temporary file, and then atomically replacing `database/app-state.json`.

## Data Flow
### Embedded-text import success
1. `ImportOrchestrator` runs the import pipeline.
2. If import succeeds with usable text, it persists:
- `Paper(status = ReadyText)`
- generated `Block[]`
- generated `ReadingProgress`
3. The orchestrator returns `ImportedText`.

### OCR-required import
1. `ImportOrchestrator` runs the import pipeline.
2. If import returns `OcrRequired`, it first creates a real `Paper`.
3. The paper is persisted with `status = ReadyOcr`.
4. A queued `OcrJob(engine = PaddleOcr, status = Queued)` is created for that real `paper_id`.
5. Both records are persisted together.
6. The orchestrator returns `QueuedOcr { paper_id, job_id }`.

### OCR execution success
1. `LocalOcrExecutor::run_job(job_id)` loads the queued job from persisted state.
2. The job transitions to `Processing`.
3. `PaddleOcrRunner` returns OCR text.
4. The application reuses importing text segmentation for the existing paper.
5. The store atomically persists:
- `OcrJob(status = Completed)`
- `OcrJobResult`
- generated `Block[]`
- generated `ReadingProgress`
- `Paper(status = ReadyText)`
6. The executor returns the completed job.

### OCR execution failure
1. The job transitions from `Queued` to `Processing`.
2. If OCR runner execution fails, or if OCR text cannot be segmented into valid readable artifacts, the job is persisted as `Failed`.
3. `Paper.status` remains `ReadyOcr`.
4. No partial `Block` or `ReadingProgress` records are written.

### Next-job execution
`LocalOcrExecutor::run_next_job()` loads the first queued job from persisted state and reuses `run_job(job_id)`.

## Cross-Crate Responsibilities
### `crates/core`
- Owns domain types only.
- Adds serialization derives needed by JSON persistence.
- Keeps OCR lifecycle validation inside domain methods.

### `crates/importing`
- Continues to own PDF extraction and text segmentation.
- Exposes a reusable text-to-readable-artifacts function for an existing `Paper`.

### `crates/application`
- Owns orchestration and persistence implementations.
- Owns the JSON app-state store.
- Coordinates real paper creation, OCR queue creation, and OCR completion writes.

## Error Handling and Consistency
- Missing `database/app-state.json` is treated as an empty store, not a startup error.
- `ImportOrchestrator` must persist `Paper + OcrJob` together for `OcrRequired`.
- OCR completion must persist all success artifacts together in one snapshot write.
- OCR failure persists only the failed job update and leaves the paper in `ReadyOcr`.
- This milestone assumes single-process serial writes only.

## Testing Strategy
All tests remain under `tests/` and cover:

- crate migration and thin-root re-export behavior
- reusable text segmentation for an existing paper
- JSON snapshot load/save round-trips
- persisted embedded-text import success
- persisted `OcrRequired` queue creation with a real `Paper`
- `run_job(job_id)` success and failure behavior against persisted state
- `run_next_job()` consuming the first queued job
- regression coverage for existing embedded-text PDF behavior

## Acceptance Criteria
- Business orchestration code no longer lives in the root `src/` tree.
- `database/app-state.json` is the default persistence file for this milestone.
- OCR queue creation always references a real persisted `Paper`.
- Successful OCR execution produces persisted readable artifacts for that paper.
- Failed OCR execution leaves the paper OCR-pending and marks the job failed.
- `run_next_job()` can execute the next queued OCR job from persisted state.
