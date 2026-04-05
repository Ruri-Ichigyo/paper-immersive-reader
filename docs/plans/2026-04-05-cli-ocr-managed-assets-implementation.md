# CLI OCR Managed Assets Implementation Plan

**Status:** Done
**Date:** 2026-04-05

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add managed PDF asset persistence plus a minimal CLI entrypoint that can execute queued OCR jobs through a local script-backed runner.

**Architecture:** Extend the application layer so PDF imports store managed assets under `database/assets/`, add a real `PaddleOcrRunner` implementation that calls a script in `scripts/`, and expose only `ocr run-next` and `ocr run <job-id>` from the CLI. Keep the default state file fixed at `database/app-state.json` and keep CLI output human-readable.

**Tech Stack:** Rust, Cargo workspace, standard library CLI parsing, `std::process::Command`, JSON persistence, shell script integration, integration tests in `tests/`

---

### Task 1: Add managed PDF asset persistence for OCR-capable imports

**Files:**
- Modify: `crates/application/src/persistence/json_app_state_store.rs`
- Modify: `crates/application/src/orchestration/import_orchestrator.rs`
- Test: `tests/import_pdf_asset_persisted.rs`

**Step 1: Write the failing test**

Create `tests/import_pdf_asset_persisted.rs`:

```rust
use paper_immersive_reader_application::orchestration::ImportOrchestrator;
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportRequest};

#[test]
fn pdf_import_persists_managed_pdf_asset_under_database_assets() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-cli-assets");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).expect("temp root should exist");
    let state_path = temp_root.join("app-state.json");

    let store = JsonAppStateStore::load_or_default(&state_path).expect("store should load");
    let mut orchestrator = ImportOrchestrator::new_with_store(
        DefaultImportPipeline::new_for_tests_with_pdf_text("hello"),
        store,
    );

    orchestrator
        .import(ImportRequest::PdfDocument {
            bytes: b"%PDF fake asset".to_vec(),
            title: Some("paper".to_string()),
            source_ref: Some("original.pdf".to_string()),
        })
        .expect("import should succeed");

    let asset_path = temp_root.join("assets").join("paper-1.pdf");
    assert!(asset_path.exists());
    assert_eq!(std::fs::read(&asset_path).expect("asset bytes"), b"%PDF fake asset".to_vec());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test import_pdf_asset_persisted -q`

Expected: FAIL because PDF assets are not currently copied into a managed location.

**Step 3: Write minimal implementation**

1. Add an asset-path helper in the application persistence layer.
2. On PDF import, copy bytes to `database/assets/<paper-id>.pdf` relative to the state file location.
3. Keep non-PDF imports unchanged.

**Step 4: Run test to verify it passes**

Run: `cargo test --test import_pdf_asset_persisted -q`

Expected: PASS.

### Task 2: Add a real script-backed PaddleOCR runner

**Files:**
- Modify: `crates/application/src/orchestration/paddle_ocr_runner.rs`
- Modify: `crates/application/src/persistence/json_app_state_store.rs`
- Create: `scripts/run_paddle_ocr.sh`
- Test: `tests/paddle_ocr_script_runner_success.rs`
- Test: `tests/paddle_ocr_script_runner_failure.rs`

**Step 1: Write the failing success test**

Create `tests/paddle_ocr_script_runner_success.rs` with a temporary stub shell script that prints OCR text to stdout and verify the real runner captures it.

**Step 2: Run test to verify it fails**

Run: `cargo test --test paddle_ocr_script_runner_success -q`

Expected: FAIL because there is no real script-backed runner yet.

**Step 3: Write the failing failure test**

Create `tests/paddle_ocr_script_runner_failure.rs` with a temporary stub shell script that exits non-zero and verify the runner returns an error.

**Step 4: Run test to verify it fails**

Run: `cargo test --test paddle_ocr_script_runner_failure -q`

Expected: FAIL because the runner does not yet map script failures.

**Step 5: Write minimal implementation**

1. Extend the runner layer with a concrete script-backed runner.
2. Resolve the managed asset path for the paper id.
3. Execute `scripts/run_paddle_ocr.sh <pdf-path>` with `std::process::Command`.
4. Read stdout as OCR text.
5. Return failure when:
- the asset file is missing
- the script exits non-zero
- stdout is empty after trimming
6. Add `scripts/run_paddle_ocr.sh` as a minimal placeholder adapter script with documented expected behavior.

**Step 6: Run tests to verify they pass**

Run:
- `cargo test --test paddle_ocr_script_runner_success -q`
- `cargo test --test paddle_ocr_script_runner_failure -q`

Expected: PASS.

### Task 3: Wire the local executor to the real runner and managed assets

**Files:**
- Modify: `crates/application/src/orchestration/local_ocr_executor.rs`
- Modify: `crates/application/src/persistence/json_app_state_store.rs`
- Test: `tests/local_ocr_executor_managed_asset_missing.rs`

**Step 1: Write the failing test**

Create `tests/local_ocr_executor_managed_asset_missing.rs`:

```rust
use paper_immersive_reader_application::orchestration::{LocalOcrExecutor, ScriptPaddleOcrRunner};
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_core::domain::{OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus};

#[test]
fn run_job_fails_when_managed_pdf_asset_is_missing() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-cli-missing-asset");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).expect("temp root should exist");
    let state_path = temp_root.join("app-state.json");

    let mut store = JsonAppStateStore::load_or_default(&state_path).expect("store should load");
    store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("scan".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("original.pdf".to_string()),
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

    let runner = ScriptPaddleOcrRunner::new(std::path::PathBuf::from("scripts/run_paddle_ocr.sh"));
    let mut executor = LocalOcrExecutor::new(store, runner, || 200);

    let result = executor.run_job(&OcrJobId("job-1".to_string()));

    assert!(result.is_none());
    assert_eq!(executor.store().snapshot().ocr_jobs[0].status, OcrJobStatus::Failed);
    assert_eq!(executor.store().snapshot().papers[0].status, PaperStatus::ReadyOcr);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test local_ocr_executor_managed_asset_missing -q`

Expected: FAIL because the real runner/asset-resolution path does not yet exist.

**Step 3: Write minimal implementation**

1. Add managed asset path lookup helpers to the store.
2. Ensure the executor can work with the real script runner.
3. Keep the existing state-transition behavior unchanged except for real asset lookup.

**Step 4: Run test to verify it passes**

Run: `cargo test --test local_ocr_executor_managed_asset_missing -q`

Expected: PASS.

### Task 4: Add CLI `ocr run-next`

**Files:**
- Modify: `crates/cli/Cargo.toml`
- Modify: `crates/cli/src/main.rs`
- Test: `tests/cli_ocr_run_next_success.rs`
- Test: `tests/cli_ocr_run_next_no_jobs.rs`

**Step 1: Write the failing success test**

Create `tests/cli_ocr_run_next_success.rs` that seeds `database/app-state.json` in a temp workspace, seeds one queued job plus managed asset, runs the CLI binary with `ocr run-next`, and asserts:
- exit code `0`
- human-readable success text is printed
- job becomes `Completed`

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_ocr_run_next_success -q`

Expected: FAIL because the CLI does not yet parse or execute OCR commands.

**Step 3: Write the failing no-jobs test**

Create `tests/cli_ocr_run_next_no_jobs.rs` to verify:
- exit code is non-zero
- output explains that no queued job exists

**Step 4: Run test to verify it fails**

Run: `cargo test --test cli_ocr_run_next_no_jobs -q`

Expected: FAIL because the CLI does not yet support the command.

**Step 5: Write minimal implementation**

In `crates/cli/src/main.rs`:
- parse `std::env::args()`
- support `ocr run-next`
- load the default state file from `database/app-state.json`
- create the script-backed runner
- execute the next queued job
- print plain-text status lines
- exit with non-zero code for failure/no queued job

**Step 6: Run tests to verify they pass**

Run:
- `cargo test --test cli_ocr_run_next_success -q`
- `cargo test --test cli_ocr_run_next_no_jobs -q`

Expected: PASS.

### Task 5: Add CLI `ocr run <job-id>`

**Files:**
- Modify: `crates/cli/src/main.rs`
- Test: `tests/cli_ocr_run_job_success.rs`
- Test: `tests/cli_ocr_run_job_missing.rs`

**Step 1: Write the failing success test**

Create `tests/cli_ocr_run_job_success.rs` to seed a specific queued job and verify `ocr run job-1` executes only that job and returns exit code `0`.

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_ocr_run_job_success -q`

Expected: FAIL because the command is not implemented yet.

**Step 3: Write the failing missing-job test**

Create `tests/cli_ocr_run_job_missing.rs` to verify missing job ids return non-zero and print a short error.

**Step 4: Run test to verify it fails**

Run: `cargo test --test cli_ocr_run_job_missing -q`

Expected: FAIL because the command is not implemented yet.

**Step 5: Write minimal implementation**

Extend CLI parsing to support:
- `ocr run <job-id>`

Keep help/error behavior simple and human-readable.

**Step 6: Run tests to verify they pass**

Run:
- `cargo test --test cli_ocr_run_job_success -q`
- `cargo test --test cli_ocr_run_job_missing -q`

Expected: PASS.

### Task 6: Run regressions and full verification

**Files:**
- Modify: none unless verification exposes a narrow fix
- Test: existing workspace tests

**Step 1: Run focused regressions**

Run:
- `cargo test --test import_orchestrator_queues_ocr_persisted -q`
- `cargo test --test local_ocr_executor_persisted_success -q`
- `cargo test --test import_pdf_real_text -q`

Expected: PASS.

**Step 2: Run formatting**

Run: `cargo fmt --check`

Expected: PASS.

**Step 3: Run clippy**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`

Expected: PASS.

**Step 4: Run full tests**

Run: `cargo test --workspace`

Expected: PASS.

### Task 7: Update tracking files

**Files:**
- Modify: `docs/plans/2026-04-05-cli-ocr-managed-assets-implementation.md`
- Modify: `docs/plans/INDEX.md`
- Modify: `progress.txt`
- Modify: `lesson.md` (if debugging or investigation occurred)

**Step 1: Mark the plan complete**

Change:

```md
**Status:** Done
```

after the milestone is fully implemented and verified.

**Step 2: Update index row**

Change the topic status from `Planned` to `Done`.

**Step 3: Append milestone progress**

Record:
- managed asset persistence
- CLI OCR commands
- script-backed OCR runner
- verification results

**Step 4: Append lessons if needed**

If any debugging/iteration happened, document it in `lesson.md`.
