# OCR Job Model (PaddleOCR) Implementation Plan

**Status:** Done
**Date:** 2026-04-02

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a minimal OCR job domain model in `crates/core` that explicitly targets PaddleOCR and represents queued OCR work, status transitions, and result linkage without implementing the runner itself.

**Architecture:** Keep OCR execution outside the import pipeline. Implement the OCR job model in `crates/core` so import can continue returning `OcrRequired`, while future local async queue runners or workers consume `OcrJob` records through a small repository boundary. Validate lifecycle transitions in domain code rather than relying only on storage behavior.

**Tech Stack:** Rust, Cargo workspace, `paper-immersive-reader-core`, integration tests in `/tests`, clippy/rustfmt

---

### Task 1: Add OCR job module skeleton and exports

**Files:**
- Create: `crates/core/src/domain/ocr_job.rs`
- Modify: `crates/core/src/domain/mod.rs`
- Test: `tests/domain_ocr_job_exports_smoke.rs`

**Step 1: Write the failing test**

Create `tests/domain_ocr_job_exports_smoke.rs`:

```rust
#[test]
fn core_ocr_job_types_are_exported() {
    let _ = paper_immersive_reader_core::domain::OcrEngine::PaddleOcr;
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test domain_ocr_job_exports_smoke -q
```
Expected: FAIL because the OCR job module and exports do not exist yet.

**Step 3: Write minimal implementation**

Create `crates/core/src/domain/ocr_job.rs` with the minimal enum:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrEngine {
    PaddleOcr,
}
```

Update `crates/core/src/domain/mod.rs`:
- `pub mod ocr_job;`
- `pub use ocr_job::OcrEngine;`

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test domain_ocr_job_exports_smoke -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/domain_ocr_job_exports_smoke.rs crates/core/src/domain/ocr_job.rs crates/core/src/domain/mod.rs
git commit -m "Add OCR engine domain module"
```

### Task 2: Add OCR job identifiers, status enum, and job struct

**Files:**
- Modify: `crates/core/src/domain/ocr_job.rs`
- Modify: `crates/core/src/domain/mod.rs`
- Test: `tests/domain_ocr_job_smoke.rs`

**Step 1: Write the failing test**

Create `tests/domain_ocr_job_smoke.rs`:

```rust
use paper_immersive_reader_core::domain::{OcrEngine, OcrJob, OcrJobId, OcrJobStatus, PaperId};

#[test]
fn ocr_job_can_be_constructed() {
    let job = OcrJob {
        id: OcrJobId("ocr-1".to_string()),
        paper_id: PaperId("paper-1".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 100,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    };

    assert_eq!(job.status, OcrJobStatus::Queued);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test domain_ocr_job_smoke -q
```
Expected: FAIL because the OCR job types do not exist yet.

**Step 3: Write minimal implementation**

In `crates/core/src/domain/ocr_job.rs`, add:

```rust
use crate::domain::PaperId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrJobId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrJobStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrJob {
    pub id: OcrJobId,
    pub paper_id: PaperId,
    pub engine: OcrEngine,
    pub status: OcrJobStatus,
    pub requested_at_ms: u64,
    pub started_at_ms: Option<u64>,
    pub finished_at_ms: Option<u64>,
    pub error_message: Option<String>,
}
```

Update `crates/core/src/domain/mod.rs` exports:
- `OcrJob`
- `OcrJobId`
- `OcrJobStatus`

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test domain_ocr_job_smoke -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/domain_ocr_job_smoke.rs crates/core/src/domain/ocr_job.rs crates/core/src/domain/mod.rs
git commit -m "Add OCR job domain types"
```

### Task 3: Add OCR job result type

**Files:**
- Modify: `crates/core/src/domain/ocr_job.rs`
- Modify: `crates/core/src/domain/mod.rs`
- Test: `tests/domain_ocr_job_result_smoke.rs`

**Step 1: Write the failing test**

Create `tests/domain_ocr_job_result_smoke.rs`:

```rust
use paper_immersive_reader_core::domain::{OcrJobResult, PaperId};

#[test]
fn ocr_job_result_can_be_constructed() {
    let result = OcrJobResult {
        paper_id: PaperId("paper-1".to_string()),
        text: "recognized text".to_string(),
    };

    assert_eq!(result.text, "recognized text");
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test domain_ocr_job_result_smoke -q
```
Expected: FAIL because `OcrJobResult` does not exist yet.

**Step 3: Write minimal implementation**

In `crates/core/src/domain/ocr_job.rs`, add:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrJobResult {
    pub paper_id: PaperId,
    pub text: String,
}
```

Update exports in `crates/core/src/domain/mod.rs`.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test domain_ocr_job_result_smoke -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/domain_ocr_job_result_smoke.rs crates/core/src/domain/ocr_job.rs crates/core/src/domain/mod.rs
git commit -m "Add OCR job result type"
```

### Task 4: Add validated lifecycle transition methods

**Files:**
- Modify: `crates/core/src/domain/ocr_job.rs`
- Test: `tests/domain_ocr_job_transitions.rs`

**Step 1: Write the failing tests**

Create `tests/domain_ocr_job_transitions.rs`:

```rust
use paper_immersive_reader_core::domain::{OcrEngine, OcrJob, OcrJobId, OcrJobStatus, PaperId};

fn queued_job() -> OcrJob {
    OcrJob {
        id: OcrJobId("ocr-1".to_string()),
        paper_id: PaperId("paper-1".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 100,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    }
}

#[test]
fn queued_job_can_start_processing() {
    let job = queued_job().mark_processing(200).expect("transition should succeed");
    assert_eq!(job.status, OcrJobStatus::Processing);
    assert_eq!(job.started_at_ms, Some(200));
}

#[test]
fn processing_job_can_complete() {
    let job = queued_job().mark_processing(200).unwrap().mark_completed(300).unwrap();
    assert_eq!(job.status, OcrJobStatus::Completed);
    assert_eq!(job.finished_at_ms, Some(300));
}

#[test]
fn processing_job_can_fail() {
    let job = queued_job().mark_processing(200).unwrap().mark_failed(300, "ocr failed".to_string()).unwrap();
    assert_eq!(job.status, OcrJobStatus::Failed);
    assert_eq!(job.error_message.as_deref(), Some("ocr failed"));
}

#[test]
fn completed_job_cannot_reenter_processing() {
    let err = queued_job().mark_processing(200).unwrap().mark_completed(300).unwrap().mark_processing(400);
    assert!(err.is_err());
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test domain_ocr_job_transitions -q
```
Expected: FAIL because transition methods do not exist yet.

**Step 3: Write minimal implementation**

In `crates/core/src/domain/ocr_job.rs`, add a small domain error for transition violations:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrJobTransitionError {
    InvalidStatusTransition,
}
```

Add methods on `OcrJob`:
- `mark_processing(started_at_ms: u64) -> Result<Self, OcrJobTransitionError>`
- `mark_completed(finished_at_ms: u64) -> Result<Self, OcrJobTransitionError>`
- `mark_failed(finished_at_ms: u64, error_message: String) -> Result<Self, OcrJobTransitionError>`

Rules:
- only `Queued -> Processing`
- only `Processing -> Completed`
- only `Processing -> Failed`
- reject all others with `InvalidStatusTransition`

Keep the implementation minimal; return updated structs by value.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test domain_ocr_job_transitions -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/domain_ocr_job_transitions.rs crates/core/src/domain/ocr_job.rs
git commit -m "Add OCR job lifecycle transitions"
```

### Task 5: Add minimal repository trait shape

**Files:**
- Modify: `crates/core/src/domain/ocr_job.rs`
- Modify: `crates/core/src/domain/mod.rs`
- Test: `tests/domain_ocr_job_repository_smoke.rs`

**Step 1: Write the failing test**

Create `tests/domain_ocr_job_repository_smoke.rs`:

```rust
use paper_immersive_reader_core::domain::{OcrJob, OcrJobId, OcrJobRepository, OcrJobResult, PaperId};

fn _assert_repository<T: OcrJobRepository>(_repo: &T) {}

struct NoopRepo;

impl OcrJobRepository for NoopRepo {
    fn create_job(&mut self, job: OcrJob) {
        let _ = job;
    }

    fn get_job(&self, _id: &OcrJobId) -> Option<&OcrJob> {
        None
    }

    fn list_jobs_for_paper(&self, _paper_id: &PaperId) -> Vec<&OcrJob> {
        Vec::new()
    }

    fn save_result(&mut self, result: OcrJobResult) {
        let _ = result;
    }
}

#[test]
fn ocr_job_repository_trait_is_implementable() {
    let repo = NoopRepo;
    _assert_repository(&repo);
}
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test domain_ocr_job_repository_smoke -q
```
Expected: FAIL because the trait does not exist yet.

**Step 3: Write minimal implementation**

In `crates/core/src/domain/ocr_job.rs`, add:

```rust
pub trait OcrJobRepository {
    fn create_job(&mut self, job: OcrJob);
    fn get_job(&self, id: &OcrJobId) -> Option<&OcrJob>;
    fn list_jobs_for_paper(&self, paper_id: &PaperId) -> Vec<&OcrJob>;
    fn save_result(&mut self, result: OcrJobResult);
}
```

Keep the trait intentionally small. Do not add worker orchestration methods here.

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test domain_ocr_job_repository_smoke -q
```
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/domain_ocr_job_repository_smoke.rs crates/core/src/domain/ocr_job.rs crates/core/src/domain/mod.rs
git commit -m "Add OCR job repository interface"
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

If verification required a small corrective edit, pair the commit with that exact fix.

### Task 7: Update milestone tracking

**Files:**
- Modify: `progress.txt`
- Modify: `lesson.md` (only if non-trivial debugging occurred)

**Step 1: Update progress log**

Append a milestone note describing:
- added `OcrJob`, `OcrJobStatus`, `OcrJobResult`, and `OcrEngine::PaddleOcr`
- added lifecycle transition validation
- added minimal repository interface shape
- verification commands run

**Step 2: Update debugging retrospective only if needed**

If actual debugging occurred, append a short retrospective entry with symptom, root cause, fix, and prevention.

**Step 3: Verify tracking changes**

Run:
```bash
cargo test --workspace
```
Expected: PASS.

**Step 4: Commit**

```bash
git add progress.txt lesson.md
git commit -m "Record OCR job model milestone progress"
```

### Task 8: Stop unless the user asks to execute the plan

**Files:**
- Modify: none

**Step 1: Do not start implementation automatically**

Stop after saving this plan.

**Step 2: Offer execution choice**

After saving this plan, tell the user:

Plan complete and saved to `docs/plans/2026-04-02-ocr-job-model-paddle-implementation.md`. Two execution options:

1. Subagent-Driven (this session) - I dispatch fresh subagent per task, review between tasks, fast iteration
2. Parallel Session (separate) - Open new session with executing-plans, batch execution with checkpoints

Which approach?
