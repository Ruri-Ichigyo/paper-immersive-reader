# OCR Enqueue and Local Executor Design

**Status:** Approved
**Date:** 2026-04-03

## Goal
Connect the existing `OcrRequired` import boundary to explicit OCR job creation, and define a minimal local asynchronous executor model that consumes queued PaddleOCR jobs without coupling OCR execution into the import pipeline.

## Architecture
Keep the system split into four layers:

- `crates/importing`
  - continues to perform import only
  - returns normal import success or `OcrRequired`
- `crates/core`
  - owns `OcrJob`, `OcrJobStatus`, `OcrJobResult`, `OcrJobRepository`, and lifecycle rules
- thin application service layer
  - calls import
  - interprets `OcrRequired`
  - creates OCR jobs
  - returns higher-level orchestration outcomes
- local OCR executor layer
  - claims queued jobs
  - runs PaddleOCR
  - writes results and final job state

Important boundary rule: import does not create jobs and does not execute OCR. OCR execution remains a follow-up step outside the import pipeline.

## Scope
In scope:
- orchestration result model for import vs. OCR-queued outcomes
- thin service that creates `OcrJob` when import returns `OcrRequired`
- minimal local executor API and lifecycle behavior
- PaddleOCR runner abstraction for engine-specific execution
- tests covering orchestration and executor state flow

Out of scope:
- background threads or daemon management
- retry scheduling
- cancellation
- distributed workers
- OCR page geometry / word boxes
- replacing the core OCR job model already added

## Core Components

### ImportOrchestrator
A thin application service that coordinates import and OCR queue creation.

Responsibilities:
- call `ImportPipeline`
- return imported text result on normal success
- if import returns `OcrRequired`, create an `OcrJob`
- return a higher-level result such as queued OCR metadata

Recommended high-level output shape:
- `ImportedText { ... }`
- `QueuedOcr { paper_id, job_id }`

### OcrJobService
A small service over `OcrJobRepository`.

Responsibilities:
- create OCR jobs for a paper
- fetch jobs by id
- list jobs for a paper

This service should not execute OCR itself.

### LocalOcrExecutor
A local asynchronous execution surface for OCR jobs.

Minimal responsibilities:
- select or receive a queued job
- mark `Queued -> Processing`
- call the OCR runner
- persist result and final status

Recommended minimal API for this milestone:
- `run_job(job_id)` or
- `run_next_job()`

Do not design a full scheduler yet.

### PaddleOcrRunner
An engine-specific execution boundary.

Responsibilities:
- accept OCR execution input for a job
- invoke PaddleOCR
- return recognized text or failure

This keeps PaddleOCR details out of the orchestrator and job service layers.

## Data Flow
1. Caller submits a PDF import request to `ImportOrchestrator`.
2. `ImportOrchestrator` calls `ImportPipeline`.
3. If import succeeds with text:
   - return `ImportedText`
   - do not create an OCR job
4. If import returns `OcrRequired`:
   - call `OcrJobService`
   - create `OcrJob { engine = PaddleOcr, status = Queued, ... }`
   - return `QueuedOcr { paper_id, job_id }`

Execution side:
5. `LocalOcrExecutor` receives a queued job id or selects a queued job.
6. It marks the job as `Processing`.
7. It calls `PaddleOcrRunner`.
8. If OCR succeeds:
   - save `OcrJobResult`
   - mark job `Completed`
9. If OCR fails:
   - mark job `Failed`
   - persist `error_message`

## Result Model
A small orchestration result type should prevent callers from having to manually stitch import and OCR queue logic together.

Recommended shape:
- `ImportedText { import_result }`
- `QueuedOcr { paper_id, job_id }`

This is intentionally higher-level than raw `ImportPipeline` output.

## Error Handling
Keep orchestration and execution error handling intentionally small.

### ImportOrchestrator
- import success -> return imported result
- `OcrRequired` -> create job and return queued result
- job creation failure -> return orchestration/service error

### LocalOcrExecutor
- job not found -> return executor error
- invalid job state -> return executor error
- PaddleOCR failure -> mark job `Failed` and persist `error_message`
- OCR success -> persist result and mark `Completed`

Do not add a deep error taxonomy in this milestone. Keep enough structure to make failures visible and state transitions correct.

## Testing Strategy
Required future implementation coverage:

1. Orchestrator success path
- if import succeeds, orchestrator returns imported text result
- no OCR job is created

2. Orchestrator OCR path
- if import returns `OcrRequired`, orchestrator creates a `Queued` PaddleOCR job
- returns `QueuedOcr { paper_id, job_id }`

3. Executor success path
- `Queued -> Processing -> Completed`
- `OcrJobResult` is saved

4. Executor failure path
- `Queued -> Processing -> Failed`
- `error_message` is saved

5. Invalid execution cases
- executing a non-existent job fails
- executing a non-queued job fails

## Implementation Notes
Recommended placement:
- keep domain types in `crates/core`
- add thin orchestration/service code in the current application layer rather than inventing a heavy new subsystem
- keep PaddleOCR invocation behind a runner trait or small adapter boundary

Suggested first milestone shape:
- in-memory repository implementation for tests
- synchronous `run_job` executor method that models the async lifecycle without requiring a real background runtime

This keeps the design local-first and easy to test while preserving the eventual queue/worker direction.

## Deliverables
This milestone is complete when the repo contains:
- an import orchestration layer that converts `OcrRequired` into queued OCR jobs
- a minimal local OCR executor API
- a PaddleOCR runner boundary
- tests covering orchestration success, OCR queue creation, executor success, executor failure, and invalid execution cases
- passing fmt, clippy, and full workspace tests
