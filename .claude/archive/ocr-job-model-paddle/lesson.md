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
