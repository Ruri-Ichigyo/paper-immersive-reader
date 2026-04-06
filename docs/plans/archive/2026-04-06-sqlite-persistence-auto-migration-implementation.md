# SQLite Persistence Auto-Migration Implementation Plan

**Status:** Done
**Date:** 2026-04-06

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace JSON snapshot persistence with a SQLite runtime store, automatically migrate legacy `database/app-state.json` into `database/app-state.db` on first startup, and keep the current CLI behavior unchanged.

**Architecture:** Add a SQLite-backed persistence layer in `crates/application`, keep JSON loading only as a migration input, and route CLI/orchestration through a single open-or-migrate entrypoint. Preserve `database/assets/` for managed PDF files and keep all OCR/import consistency updates transactional.

**Tech Stack:** Rust, Cargo workspace, SQLite via a Rust SQLite crate, existing `paper-immersive-reader-core` and `paper-immersive-reader-importing` crates, integration tests in `tests/`

---

### Task 1: Add SQLite dependencies and a minimal persistence skeleton

**Files:**
- Modify: `crates/application/Cargo.toml`
- Modify: `crates/application/src/persistence/mod.rs`
- Create: `crates/application/src/persistence/sqlite_app_state_store.rs`
- Test: `tests/sqlite_app_state_store_smoke.rs`

**Step 1: Write the failing test**

Create `tests/sqlite_app_state_store_smoke.rs`:

```rust
use paper_immersive_reader_application::persistence::{SqliteAppStateStore, DEFAULT_APP_DB_PATH};

#[test]
fn sqlite_store_creates_database_and_default_schema() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-sqlite-smoke");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).expect("temp root should exist");
    let db_path = temp_root.join("app-state.db");

    let store = SqliteAppStateStore::open(&db_path).expect("sqlite store should open");

    assert!(db_path.exists());
    assert_eq!(DEFAULT_APP_DB_PATH, "database/app-state.db");
    assert!(store.first_queued_job().is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test sqlite_app_state_store_smoke -q`

Expected: FAIL because there is no SQLite store or default database path constant yet.

**Step 3: Write minimal implementation**

1. Add a SQLite dependency to `crates/application/Cargo.toml`.
2. Create `crates/application/src/persistence/sqlite_app_state_store.rs`.
3. Define:
- `pub const DEFAULT_APP_DB_PATH: &str = "database/app-state.db";`
- `SqliteAppStateStore::open(path)`
- schema initialization for `papers`, `blocks`, `reading_progress`, `ocr_jobs`, and `ocr_job_results`
4. Export the SQLite store from `crates/application/src/persistence/mod.rs`.
5. Keep the JSON store exported for now because later tasks still need it as a migration input.

**Step 4: Run test to verify it passes**

Run: `cargo test --test sqlite_app_state_store_smoke -q`

Expected: PASS.

### Task 2: Add SQLite round-trip operations for the current persisted entities

**Files:**
- Modify: `crates/application/src/persistence/sqlite_app_state_store.rs`
- Test: `tests/sqlite_app_state_store_round_trip.rs`

**Step 1: Write the failing test**

Create `tests/sqlite_app_state_store_round_trip.rs` that seeds one `Paper`, one `ReadingProgress`, one queued `OcrJob`, and one `OcrJobResult`, saves them through the SQLite store, reloads the store, and asserts the same records can be loaded.

**Step 2: Run test to verify it fails**

Run: `cargo test --test sqlite_app_state_store_round_trip -q`

Expected: FAIL because the SQLite store only creates schema and does not yet implement read/write operations.

**Step 3: Write minimal implementation**

In `crates/application/src/persistence/sqlite_app_state_store.rs`:
- implement upsert helpers for `Paper`, `Block`, `ReadingProgress`, `OcrJob`, and `OcrJobResult`
- implement read helpers matching the current runtime usage:
  - `get_paper`
  - `get_job`
  - `first_queued_job`
  - store save methods for import success, paper-plus-job enqueue, job update, and completed OCR writes
- keep block replacement and progress replacement behavior aligned with the current JSON store semantics

**Step 4: Run test to verify it passes**

Run: `cargo test --test sqlite_app_state_store_round_trip -q`

Expected: PASS.

### Task 3: Add JSON-to-SQLite automatic migration with backup rename

**Files:**
- Modify: `crates/application/src/persistence/mod.rs`
- Modify: `crates/application/src/persistence/json_app_state_store.rs`
- Modify: `crates/application/src/persistence/sqlite_app_state_store.rs`
- Test: `tests/sqlite_app_state_store_migrates_json.rs`
- Test: `tests/sqlite_app_state_store_invalid_json_preserved.rs`

**Step 1: Write the failing migration success test**

Create `tests/sqlite_app_state_store_migrates_json.rs` that:
- creates a temp `database/app-state.json` with one paper and one queued job
- opens the persistence layer through an `open_or_migrate` entrypoint
- asserts `database/app-state.db` exists
- asserts `database/app-state.migrated.json` exists
- asserts the migrated paper and job are queryable from SQLite

**Step 2: Run test to verify it fails**

Run: `cargo test --test sqlite_app_state_store_migrates_json -q`

Expected: FAIL because no open-or-migrate logic exists yet.

**Step 3: Write the failing invalid JSON test**

Create `tests/sqlite_app_state_store_invalid_json_preserved.rs` that:
- writes malformed JSON to `database/app-state.json`
- calls the open-or-migrate entrypoint
- asserts the call fails
- asserts `database/app-state.json` still exists
- asserts `database/app-state.migrated.json` was not created

**Step 4: Run test to verify it fails**

Run: `cargo test --test sqlite_app_state_store_invalid_json_preserved -q`

Expected: FAIL because migration failure handling is not implemented.

**Step 5: Write minimal implementation**

1. Add a persistence entrypoint such as `SqliteAppStateStore::open_or_migrate(path)` or a module-level helper.
2. If the `.db` file exists, open it and ignore legacy JSON.
3. If the `.db` file does not exist and a sibling `.json` file exists:
- load the JSON snapshot
- create the SQLite database
- import all entity sets inside a single transaction
- rename the JSON file to `.migrated.json` only after commit succeeds
4. Keep `JsonAppStateStore` as a migration parser/input only; do not route runtime writes through it.

**Step 6: Run tests to verify they pass**

Run:
- `cargo test --test sqlite_app_state_store_migrates_json -q`
- `cargo test --test sqlite_app_state_store_invalid_json_preserved -q`

Expected: PASS.

### Task 4: Move import orchestration to the SQLite runtime store

**Files:**
- Modify: `crates/application/src/orchestration/import_orchestrator.rs`
- Modify: `crates/application/src/persistence/sqlite_app_state_store.rs`
- Test: `tests/import_orchestrator_persists_ready_text_sqlite.rs`
- Test: `tests/import_orchestrator_queues_ocr_sqlite.rs`

**Step 1: Write the failing ready-text test**

Create `tests/import_orchestrator_persists_ready_text_sqlite.rs` that runs a PDF import through the orchestrator backed by a SQLite store and verifies:
- a `Paper(status = ReadyText)` is persisted
- generated `Block[]` rows are persisted
- generated `ReadingProgress` is persisted

**Step 2: Run test to verify it fails**

Run: `cargo test --test import_orchestrator_persists_ready_text_sqlite -q`

Expected: FAIL because the orchestrator still depends on the JSON store.

**Step 3: Write the failing OCR-queue test**

Create `tests/import_orchestrator_queues_ocr_sqlite.rs` that runs an OCR-required import and verifies:
- a real paper is persisted with `status = ReadyOcr`
- one queued OCR job is persisted for that paper
- the managed PDF asset is still stored under `database/assets/<paper-id>.pdf`

**Step 4: Run test to verify it fails**

Run: `cargo test --test import_orchestrator_queues_ocr_sqlite -q`

Expected: FAIL because enqueue persistence is not yet wired to SQLite.

**Step 5: Write minimal implementation**

In `crates/application/src/orchestration/import_orchestrator.rs`:
- replace the runtime dependency on `JsonAppStateStore` with the SQLite-backed store
- preserve the current import result behavior and managed-asset persistence
- keep `Paper + OcrJob` creation transactional for OCR-required imports

**Step 6: Run tests to verify they pass**

Run:
- `cargo test --test import_orchestrator_persists_ready_text_sqlite -q`
- `cargo test --test import_orchestrator_queues_ocr_sqlite -q`

Expected: PASS.

### Task 5: Move the local OCR executor to the SQLite runtime store

**Files:**
- Modify: `crates/application/src/orchestration/local_ocr_executor.rs`
- Modify: `crates/application/src/persistence/sqlite_app_state_store.rs`
- Test: `tests/local_ocr_executor_persisted_success_sqlite.rs`
- Test: `tests/local_ocr_executor_persisted_failure_sqlite.rs`
- Test: `tests/local_ocr_executor_run_next_job_sqlite.rs`

**Step 1: Write the failing success test**

Create `tests/local_ocr_executor_persisted_success_sqlite.rs` that seeds a queued OCR job plus managed asset in SQLite, runs the executor, and verifies:
- the job becomes `Completed`
- the paper becomes `ReadyText`
- blocks, progress, and OCR result are persisted

**Step 2: Run test to verify it fails**

Run: `cargo test --test local_ocr_executor_persisted_success_sqlite -q`

Expected: FAIL because the executor still depends on the JSON runtime store.

**Step 3: Write the failing failure test**

Create `tests/local_ocr_executor_persisted_failure_sqlite.rs` that seeds a queued OCR job with a runner failure and verifies:
- the job becomes `Failed`
- the paper remains `ReadyOcr`
- no blocks or progress are written

**Step 4: Run test to verify it fails**

Run: `cargo test --test local_ocr_executor_persisted_failure_sqlite -q`

Expected: FAIL because failure persistence is not yet wired to SQLite.

**Step 5: Write the failing run-next test**

Create `tests/local_ocr_executor_run_next_job_sqlite.rs` that seeds multiple queued jobs, runs `run_next_job()`, and verifies the oldest queued job is claimed and completed first.

**Step 6: Run test to verify it fails**

Run: `cargo test --test local_ocr_executor_run_next_job_sqlite -q`

Expected: FAIL because `run_next_job()` is not yet implemented transactionally on SQLite.

**Step 7: Write minimal implementation**

In `crates/application/src/orchestration/local_ocr_executor.rs` and the SQLite store:
- route runtime reads/writes through SQLite
- claim queued jobs transactionally
- preserve existing failure messages and state transitions
- keep managed-asset path resolution unchanged

**Step 8: Run tests to verify they pass**

Run:
- `cargo test --test local_ocr_executor_persisted_success_sqlite -q`
- `cargo test --test local_ocr_executor_persisted_failure_sqlite -q`
- `cargo test --test local_ocr_executor_run_next_job_sqlite -q`

Expected: PASS.

### Task 6: Keep CLI behavior unchanged while switching the default store to SQLite

**Files:**
- Modify: `crates/cli/src/main.rs`
- Test: `tests/cli_ocr_run_next_sqlite_success.rs`
- Test: `tests/cli_ocr_run_job_sqlite_success.rs`
- Test: `tests/cli_auto_migrates_legacy_json.rs`

**Step 1: Write the failing run-next CLI test**

Create `tests/cli_ocr_run_next_sqlite_success.rs` that seeds the default SQLite database path in a temp workspace, runs `paper-immersive-reader-cli ocr run-next`, and verifies the command still exits `0` with the existing success message shape.

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_ocr_run_next_sqlite_success -q`

Expected: FAIL because the CLI still loads `database/app-state.json`.

**Step 3: Write the failing run-job CLI test**

Create `tests/cli_ocr_run_job_sqlite_success.rs` that seeds a specific queued SQLite job and verifies `ocr run <job-id>` still succeeds with the same user-visible behavior.

**Step 4: Run test to verify it fails**

Run: `cargo test --test cli_ocr_run_job_sqlite_success -q`

Expected: FAIL because the CLI has not switched its default path or persistence entrypoint.

**Step 5: Write the failing auto-migration CLI test**

Create `tests/cli_auto_migrates_legacy_json.rs` that seeds only `database/app-state.json` in a temp workspace, runs `ocr run-next`, and verifies the CLI:
- creates `database/app-state.db`
- renames the source JSON to `database/app-state.migrated.json`
- completes the queued job successfully

**Step 6: Run test to verify it fails**

Run: `cargo test --test cli_auto_migrates_legacy_json -q`

Expected: FAIL because CLI-triggered auto-migration is not yet wired.

**Step 7: Write minimal implementation**

In `crates/cli/src/main.rs`:
- switch the default path constant to `database/app-state.db`
- load the runtime store via the SQLite open-or-migrate entrypoint
- keep the command surface, success strings, and failure semantics unchanged

**Step 8: Run tests to verify they pass**

Run:
- `cargo test --test cli_ocr_run_next_sqlite_success -q`
- `cargo test --test cli_ocr_run_job_sqlite_success -q`
- `cargo test --test cli_auto_migrates_legacy_json -q`

Expected: PASS.

### Task 7: Retire JSON as the runtime store and align docs and tracking

**Files:**
- Modify: `crates/application/src/persistence/mod.rs`
- Modify: `tests/json_app_state_store_smoke.rs`
- Modify: `docs/plans/INDEX.md`
- Modify: `progress.txt`
- Modify: `lesson.md`

**Step 1: Write the failing migration-boundary test**

Update `tests/json_app_state_store_smoke.rs` so it verifies JSON loading remains available only for migration input and is no longer presented as the default runtime persistence path.

**Step 2: Run test to verify it fails**

Run: `cargo test --test json_app_state_store_smoke -q`

Expected: FAIL because the old test still assumes JSON is the default runtime store.

**Step 3: Write minimal implementation**

1. Remove any remaining runtime-oriented default-path exports that point to JSON.
2. Keep the JSON store reachable only through migration-specific APIs needed by the SQLite bootstrap.
3. Update `docs/plans/INDEX.md` and the implementation-plan status after verification succeeds.
4. Record the milestone completion in `progress.txt`.
5. If any debugging or iteration was required during implementation, append the root cause and final fix to `lesson.md`.

**Step 4: Run tests to verify they pass**

Run: `cargo test --test json_app_state_store_smoke -q`

Expected: PASS with the revised migration-only expectation.

### Task 8: Run full verification and close the milestone

**Files:**
- Modify: `docs/plans/2026-04-06-sqlite-persistence-auto-migration-implementation.md`
- Modify: `docs/plans/INDEX.md`
- Modify: `progress.txt`
- Modify: `lesson.md`

**Step 1: Run focused regression commands**

Run:
- `cargo test --test sqlite_app_state_store_smoke -q`
- `cargo test --test sqlite_app_state_store_migrates_json -q`
- `cargo test --test import_orchestrator_queues_ocr_sqlite -q`
- `cargo test --test local_ocr_executor_persisted_success_sqlite -q`
- `cargo test --test cli_auto_migrates_legacy_json -q`

Expected: PASS.

**Step 2: Run repository-wide verification**

Run:
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`

Expected: PASS.

**Step 3: Mark the milestone complete**

1. Change this document's `**Status:**` from `Planned` to `Done`.
2. Update the topic row in `docs/plans/INDEX.md` from `Planned` to `Done`.
3. Append the verification results and next-step summary to `progress.txt`.
4. If debugging occurred, append the retrospective to `lesson.md`.
