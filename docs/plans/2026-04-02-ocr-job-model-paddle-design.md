# OCR Job Model (PaddleOCR) Design

**Status:** Approved
**Date:** 2026-04-02

## Goal
Add a minimal local asynchronous OCR job model that explicitly targets PaddleOCR, so the system can represent OCR work as queued domain state instead of treating OCR only as a generic boundary result.

This milestone defines the job model, status flow, and storage-facing interface shape. It does not implement the OCR runner itself.

## Architecture
Keep OCR execution outside the import pipeline.

Recommended boundary split:
- `crates/core`
  - owns OCR job domain types and lifecycle rules
- `crates/importing`
  - continues to detect `OcrRequired` during PDF import
  - does not execute OCR
- future OCR executor / worker layer
  - consumes OCR jobs and runs PaddleOCR

This preserves the current import pipeline shape while making OCR a first-class follow-up task in the domain model.

## Scope
In scope:
- OCR job domain model
- PaddleOCR engine binding in the job model
- job status lifecycle
- OCR result model
- minimal repository interface for storing/querying/updating jobs
- validation rules for legal state transitions

Out of scope:
- invoking PaddleOCR
- queue runner / worker implementation
- retry scheduling
- cancellation
- OCR page geometry / bounding box modeling
- image rasterization from PDF pages
- replacing the current `OcrRequired` import outcome

## Core Components

### OcrEngine
Use an explicit engine enum in the domain model:

```rust
pub enum OcrEngine {
    PaddleOcr,
}
```

This makes OCR jobs auditable and intentionally tied to the selected engine.

### OcrJob
Minimal OCR job record:
- `id`
- `paper_id`
- `engine`
- `status`
- `requested_at_ms`
- `started_at_ms`
- `finished_at_ms`
- `error_message`

Recommended shape:
- `id: OcrJobId`
- `paper_id: PaperId`
- `engine: OcrEngine`
- `status: OcrJobStatus`
- `requested_at_ms: u64`
- `started_at_ms: Option<u64>`
- `finished_at_ms: Option<u64>`
- `error_message: Option<String>`

### OcrJobStatus
Keep the state machine intentionally small:
- `Queued`
- `Processing`
- `Completed`
- `Failed`

Do not add `Retrying`, `Cancelled`, or `Expired` in this milestone.

### OcrJobResult
Minimal successful OCR output model:
- `paper_id`
- `text`
- optional engine note if useful later

Recommended shape:
- `paper_id: PaperId`
- `text: String`

Keep it text-only for now. Do not model word boxes, page regions, or layout metadata yet.

### OcrJobRepository
Define a small repository boundary that future local or persistent queue implementations can satisfy.

Minimum operations:
- create job
- get by job id
- list by paper id
- mark processing
- mark completed
- mark failed

## Data Flow
1. The import pipeline attempts PDF text extraction.
2. If embedded text is available, import completes normally with `ReadyText`.
3. If no usable text is available, the import pipeline returns `OcrRequired`.
4. The caller may then create an `OcrJob` with:
   - `paper_id`
   - `engine = PaddleOcr`
   - `status = Queued`
5. A future local asynchronous executor or worker picks up the job:
   - `Queued -> Processing`
6. If PaddleOCR succeeds:
   - `Processing -> Completed`
   - persist `OcrJobResult`
7. If PaddleOCR fails:
   - `Processing -> Failed`
   - record `error_message`

Important constraint: this milestone does not change import to automatically enqueue or execute OCR. It only defines the model that makes OCR queueing possible.

## State Transition Rules
Allowed transitions:
- create job -> `Queued`
- `Queued -> Processing`
- `Processing -> Completed`
- `Processing -> Failed`

Disallowed transitions in this milestone:
- `Completed -> Processing`
- `Completed -> Failed`
- `Failed -> Completed`
- `Queued -> Completed`
- `Queued -> Failed` without processing, unless the repository layer later needs a system-level shortcut

Recommended modeling approach:
- expose small transition methods on `OcrJob`
- validate transitions in domain code, not only in storage code

## Error Handling
Keep OCR job failures simple in this milestone.

At the domain level:
- use `status = Failed`
- store a freeform `error_message`

Do not introduce a detailed PaddleOCR-specific error taxonomy yet.

Why:
- this milestone is about queue and lifecycle shape, not execution diagnostics
- a freeform message is enough for basic visibility
- richer categories can be added later without changing the core job lifecycle

## Testing Strategy
Required future implementation coverage:

1. Job creation
- creating a job sets:
  - `engine = PaddleOcr`
  - `status = Queued`
  - `requested_at_ms`

2. Valid transitions
- `Queued -> Processing`
- `Processing -> Completed`
- `Processing -> Failed`

3. Invalid transitions
- `Completed -> Processing` rejected
- `Failed -> Completed` rejected
- `Queued -> Completed` rejected

4. Result linkage
- completed jobs can be associated with `OcrJobResult`
- failed jobs may carry `error_message`

5. Paper linkage
- jobs are always tied to a `PaperId`
- repository listing by `paper_id` behaves as expected

## Implementation Notes
Recommended placement in `crates/core`:
- `crates/core/src/domain/ocr_job.rs`
- export via `crates/core/src/domain/mod.rs`

Suggested types:
- `OcrJobId(String)`
- `OcrEngine`
- `OcrJobStatus`
- `OcrJob`
- `OcrJobResult`

If a repository trait is included now, keep it small and domain-oriented. Do not add infrastructure concerns like thread pools, channels, persistence engines, or process management to this milestone.

## Deliverables
This milestone is complete when the repo contains:
- OCR job domain types in `crates/core`
- explicit `PaddleOcr` engine binding in the model
- validated job status transitions
- minimal OCR result type
- repository interface shape for future queue backends
- integration or focused tests covering creation and lifecycle rules
