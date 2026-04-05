# lesson.md — Debugging / Iteration Retrospective

## 2026-03-28
- Symptom: `cargo clippy --workspace --all-targets --all-features -- -D warnings` failed on `tests/workspace_smoke.rs`.
- Root cause: the placeholder test used `assert!(true)`, which triggers Clippy's `assertions_on_constants` lint when warnings are denied.
- What was tried: ran full verification, inspected the failing test file, and traced the failure to the constant assertion rather than the workspace scaffold.
- Final fix: replaced the placeholder smoke test with a real assertion against `paper_immersive_reader::WORKSPACE`.
- Prevention: avoid placeholder assertions in scaffold tests; use real structural assertions that survive strict lint settings.

## 2026-03-28 (reader domain models)
- Symptom: `cargo test --test domain_progress_smoke -q` failed after adding `ReadingProgress`, but the failure was duplicate module/re-export definitions inside `crates/core/src/domain/mod.rs` rather than the new type itself.
- Root cause: incremental edits to `crates/core/src/domain/mod.rs` duplicated `pub mod block;` and repeated `pub use` lines for `DomainError` and paper types.
- What was tried: reran the failing test, read the module file directly, identified the duplicated lines, and removed the redundant declarations.
- Final fix: normalized `crates/core/src/domain/mod.rs` to a single declaration/re-export block for `block`, `error`, `paper`, and `reading_progress`.
- Prevention: when incrementally editing module export files, reread the whole file before appending new declarations so duplicate module/re-export lines are not introduced.

## 2026-03-28 (import pipeline push recovery)
- Symptom: pre-push verification failed with `could not find domain in paper_immersive_reader_core` while compiling `paper-immersive-reader-importing`.
- Root cause: `crates/core/src/lib.rs` had lost `pub mod domain;`, so the importing crate could no longer import `paper_immersive_reader_core::domain::*`; the root `Cargo.toml` also needed the importing crate re-wired into workspace members and root dev-dependencies after reset/recovery work.
- What was tried: reran focused verification, read the current `crates/core/src/lib.rs` and importing crate source, compared against the earlier domain-model commit state, and confirmed the missing export mismatch rather than a test-only problem.
- Final fix: restored `pub mod domain;` in `crates/core/src/lib.rs` and re-added `crates/importing` plus `paper-immersive-reader-importing` to the root `Cargo.toml` workspace/test wiring.
- Prevention: after history recovery or partial resets, rerun full workspace verification before pushing and compare key public API files against the intended earlier commit when symptoms suggest a missing export.

## 2026-04-02 (real PDF extraction fixtures and clippy)
- Symptom: the first real embedded-text PDF fixture was accepted by some PDF tools but rejected by the selected extraction library, which pushed the implementation toward an unnecessary handwritten fallback parser.
- Root cause: the fixture itself was malformed enough for `pdf-extract` to reject it, so the apparent extractor limitation was actually bad test input rather than missing product behavior.
- What was tried: verified the failure on the real extractor path, compared fixture behavior against the library, then regenerated a small valid embedded-text fixture and removed the fallback parser to get back to the intended design.
- Final fix: replaced the malformed fixture with a valid minimal embedded-text PDF and kept the real extractor path limited to the chosen library plus the existing OCR-boundary mapping.
- Prevention: treat fixture validity as part of root-cause analysis when adding format-specific tests; fix the fixture before adding production fallbacks.
-
- Symptom: `cargo clippy --workspace --all-targets --all-features -- -D warnings` failed after adding `DefaultImportPipeline::new()`.
- Root cause: Clippy's `new_without_default` lint requires a matching `Default` implementation when a parameterless `new()` constructor is introduced.
- What was tried: reran clippy, read the exact lint output, and confirmed the failure was an API-shape issue rather than extractor logic.
- Final fix: added `impl Default for DefaultImportPipeline { fn default() -> Self { Self::new() } }`.
- Prevention: when adding zero-argument constructors in this repo, immediately add `Default` or explicitly justify why the type should not implement it.

## 2026-04-05 (plans directory status reconciliation)
- Symptom: `docs/plans` still showed several already-implemented milestones as `Planned`, and `docs/plans/INDEX.md` did not list the later PDF/OCR topics.
- Root cause: implementation milestones were delivered incrementally, but the plan metadata and index table were not updated in the same pass; later implementation docs also missed the normalized `Status` / `Date` header.
- What was tried: audited all plan headers, compared directory contents against `INDEX.md`, and cross-checked milestone completion against the existing test files and `progress.txt` ledger.
- Final fix: marked implemented `*-implementation.md` documents as `Done`, added the missing normalized headers, and expanded `docs/plans/INDEX.md` to cover every current topic with accurate status notes.
- Prevention: whenever a milestone is verified and recorded in `progress.txt`, update the matching implementation-plan status and the plans index in that same change.

## 2026-04-05 (OCR JSON persistence design investigation)
- Symptom: the existing OCR queue path only persisted in-memory test data and generated a placeholder `paper_id`, so OCR work could not reliably attach back to a real paper record.
- Root cause: earlier milestones intentionally stopped at the queue boundary, leaving no persistence layer for `Paper`, `Block`, `ReadingProgress`, and OCR artifacts, and leaving business orchestration under the root `src/` tree.
- What was tried: reviewed the current orchestration code, import segmentation path, and repository layout; compared JSON snapshot, multi-file JSON, and SQLite approaches; then constrained the design to the smallest consistent local-first option.
- Final fix: chose a single-file JSON snapshot at `database/app-state.json`, required real paper creation before OCR enqueue, and planned a new `crates/application` business crate so persistence and orchestration can grow without violating repository governance.
- Prevention: when an intermediate milestone introduces queue boundaries or placeholder IDs, define the persistence owner and stable aggregate identity in the very next design pass instead of carrying placeholders forward.

## 2026-04-05 (application crate migration and persistence refactor)
- Symptom: once OCR persistence was wired in, the old in-memory orchestrator/executor tests no longer fit the implementation because `OcrJobService` and `LocalOcrExecutor` had to stop depending on placeholder repositories and start operating on persisted app state.
- Root cause: the earlier milestone APIs were deliberately optimized for seam-level tests and placeholder queueing, but the new milestone needed real paper creation, JSON snapshot writes, and executor updates that touched multiple aggregates together.
- What was tried: first migrated orchestration into `crates/application`, then added reusable text segmentation for existing papers, then introduced `JsonAppStateStore`, and finally updated the older queue/executor tests from in-memory repository assertions to persisted snapshot assertions.
- Final fix: made `crates/application` the business-layer owner for orchestration and persistence, introduced `PersistenceFailed` as a typed import error, and aligned both new and existing tests around JSON store snapshots instead of placeholder in-memory queues.
- Prevention: when a milestone promotes a seam into a real persisted workflow, update the test surface to assert persisted state directly rather than preserving old placeholder repository assumptions.

## 2026-04-05 (CLI OCR managed-assets design investigation)
- Symptom: after adding persisted OCR jobs, the next obvious entrypoint was CLI execution, but the current model still lacked a stable source-of-truth for the PDF bytes that OCR should actually read.
- Root cause: the persistence milestone stored structured OCR and reading state but not the source asset itself, so a CLI command would have had to rely on fragile original file paths or ad hoc command arguments.
- What was tried: reviewed the current JSON store, executor, CLI placeholder crate, and OCR design documents; compared relying on `Paper.source_ref`, passing ad hoc file paths, and introducing managed assets under `database/`.
- Final fix: chose managed PDF assets under `database/assets/<paper-id>.pdf`, kept the CLI scope to `ocr run-next` and `ocr run <job-id>`, and planned a script-backed runner boundary in `scripts/` so the CLI can stay thin.
- Prevention: whenever a new execution surface is added for persisted jobs, define the authoritative data asset location before designing command semantics.

## 2026-04-05 (managed-asset runner boundary)
- Symptom: the original `PaddleOcrRunner` trait only accepted `OcrJob`, which left no stable place to resolve the managed PDF asset path needed by the real script-backed runner.
- Root cause: the managed asset location is derived from the JSON store path, so it belongs to the persistence/executor boundary rather than the job model itself.
- What was tried: reviewed the existing runner and executor tests, compared pushing state-path knowledge into the runner constructor against resolving the asset path in `LocalOcrExecutor`, and checked the planned temp-state test shape for missing assets.
- Final fix: added store helpers for managed asset paths, changed the runner trait to `run(&OcrJob, &Path)`, kept asset lookup in `LocalOcrExecutor`, and implemented `ScriptPaddleOcrRunner` on top of `std::process::Command`.
- Prevention: keep filesystem path resolution attached to the persistence owner; runner traits should receive concrete inputs rather than rediscovering them from partial domain state.

## 2026-04-05 (CLI integration-test helper warnings)
- Symptom: the first CLI integration-test pass left per-test `dead_code` warnings for the shared helper module, which would have become failures under the repo's `cargo clippy --workspace --all-targets --all-features -- -D warnings` policy.
- Root cause: each file under `tests/` compiles as its own crate, so a helper function used by the success-path tests still counted as unused inside the failure-path test crates that imported the same support module.
- What was tried: first considered suppressing the warning on the helper itself, then switched to making the failure-path tests create the same placeholder script layout as the success-path tests so the shared helper stayed genuinely used.
- Final fix: kept `tests/support/cli.rs` warning-free by using `write_executable_script` in all CLI integration tests and reran formatting plus full verification.
- Prevention: when sharing test helpers across integration-test crates, remember that usage is checked per crate; either split helpers more narrowly or ensure each imported helper is exercised in every test crate that includes it.
