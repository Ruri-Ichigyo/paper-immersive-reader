# Web MVP Closed Loop Implementation Plan

**Status:** Planned
**Date:** 2026-04-06

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver the remaining PRD MVP loop as a Cargo-governed web-first, local-first product: local import, immersive reading, selection translation, AI explanation, annotations, restored progress, and vocabulary auto-collection.

**Architecture:** Add a new Rust/WASM `crates/web-app` crate for the browser UI, expand `crates/gateway` into a thin AI HTTP service, keep shared domain logic in `crates/core`, and reuse `crates/application` plus `crates/importing` for shared orchestration boundaries. The browser stays authoritative for local reader data and the gateway is used only for online AI actions in MVP.

**Tech Stack:** Rust, Cargo workspace, Leptos, wasm-bindgen/web-sys, IndexedDB via a Rust crate, vendored `pdf.js` assets, Axum, Tokio, Tracing

---

### Task 1: Scaffold the Cargo-native web MVP surface

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/web-app/Cargo.toml`
- Create: `crates/web-app/src/main.rs`
- Create: `crates/web-app/src/app.rs`
- Create: `crates/web-app/src/routes/mod.rs`
- Create: `crates/web-app/src/routes/library.rs`
- Create: `crates/web-app/src/routes/import.rs`
- Create: `crates/web-app/src/routes/read.rs`
- Create: `crates/web-app/src/routes/vocabulary.rs`
- Create: `crates/web-app/src/routes/settings.rs`
- Create: `scripts/run_web_app.sh`
- Test: `tests/web_app_workspace_smoke.rs`

**Step 1: Write the failing test**

Create `tests/web_app_workspace_smoke.rs`:

```rust
#[test]
fn web_app_crate_is_linkable_from_workspace() {
    let crate_name = env!("CARGO_PKG_NAME");
    assert!(!crate_name.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test web_app_workspace_smoke -q`

Expected: FAIL because `crates/web-app` is not yet part of the workspace.

**Step 3: Write minimal implementation**

1. Add `crates/web-app` to the workspace in `Cargo.toml`.
2. Create the new crate with route modules for library, import, reading, vocabulary, and settings.
3. Keep the initial app shell thin: route placeholders, navigation shell, and no business logic beyond wiring.
4. Add `scripts/run_web_app.sh` as the standard startup helper under `scripts/`.

**Step 4: Run test to verify it passes**

Run: `cargo test --test web_app_workspace_smoke -q`

Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml crates/web-app scripts/run_web_app.sh tests/web_app_workspace_smoke.rs
git commit -m "feat: scaffold cargo-native web app crate"
```

### Task 2: Add browser-local persistence for the MVP entities

**Files:**
- Create: `crates/web-app/src/storage/mod.rs`
- Create: `crates/web-app/src/storage/schema.rs`
- Create: `crates/web-app/src/storage/indexed_db.rs`
- Create: `crates/web-app/src/storage/papers.rs`
- Create: `crates/web-app/src/storage/annotations.rs`
- Create: `crates/web-app/src/storage/vocabulary.rs`
- Create: `crates/web-app/src/storage/settings.rs`
- Modify: `crates/core/src/domain/mod.rs`
- Create: `crates/core/src/domain/annotation.rs`
- Create: `crates/core/src/domain/vocabulary.rs`
- Test: `tests/web_storage_round_trip.rs`
- Test: `tests/web_vocabulary_round_trip.rs`

**Step 1: Write the failing persistence test**

Create `tests/web_storage_round_trip.rs` that persists one `Paper`, `Block`, and `ReadingProgress` record through the web storage boundary and asserts they can be reloaded by `paper_id`.

**Step 2: Run test to verify it fails**

Run: `cargo test --test web_storage_round_trip -q`

Expected: FAIL because the web app has no local storage layer or shared annotation/vocabulary domain types yet.

**Step 3: Write the failing vocabulary test**

Create `tests/web_vocabulary_round_trip.rs` that saves one looked-up term, saves the same term again, and verifies the lookup count is incremented instead of duplicating rows.

**Step 4: Run test to verify it fails**

Run: `cargo test --test web_vocabulary_round_trip -q`

Expected: FAIL because vocabulary persistence and merge rules are not implemented.

**Step 5: Write minimal implementation**

1. Add browser-local storage modules in `crates/web-app/src/storage/`.
2. Add shared `Annotation` and `VocabularyItem` domain types in `crates/core`.
3. Implement the MVP schema for papers, blocks, progress, annotations, vocabulary, AI records, and settings.
4. Keep the storage API keyed by `paper_id` and stable entity IDs so later sync work can build on the same shape.

**Step 6: Run tests to verify they pass**

Run:
- `cargo test --test web_storage_round_trip -q`
- `cargo test --test web_vocabulary_round_trip -q`

Expected: PASS.

**Step 7: Commit**

```bash
git add crates/web-app/src/storage crates/core/src/domain tests/web_storage_round_trip.rs tests/web_vocabulary_round_trip.rs
git commit -m "feat: add web local persistence for mvp entities"
```

### Task 3: Implement the import flow for pasted text and local PDFs

**Files:**
- Create: `crates/web-app/src/import/mod.rs`
- Create: `crates/web-app/src/import/paste.rs`
- Create: `crates/web-app/src/import/pdf.rs`
- Create: `crates/web-app/src/import/pdf_bridge.rs`
- Create: `crates/web-app/assets/pdfjs/README.md`
- Modify: `crates/importing/src/lib.rs`
- Modify: `crates/importing/src/pdf.rs`
- Modify: `crates/web-app/src/routes/import.rs`
- Modify: `crates/web-app/src/routes/library.rs`
- Test: `tests/web_import_paste_flow.rs`
- Test: `tests/web_import_pdf_flow.rs`

**Step 1: Write the failing paste-import test**

Create `tests/web_import_paste_flow.rs` that submits pasted text through the import boundary and verifies:
- one `Paper` is created
- one or more `Block` rows are created
- the initial `ReadingProgress` starts at block index `0`

**Step 2: Run test to verify it fails**

Run: `cargo test --test web_import_paste_flow -q`

Expected: FAIL because the import route is still a shell and no import service persists entities.

**Step 3: Write the failing PDF-import test**

Create `tests/web_import_pdf_flow.rs` that exercises the PDF import seam with a stubbed extraction bridge and verifies:
- the original PDF asset is stored locally
- extracted text is normalized into blocks
- a readable paper opens with `ReadyText`

**Step 4: Run test to verify it fails**

Run: `cargo test --test web_import_pdf_flow -q`

Expected: FAIL because there is no browser PDF bridge or import orchestration yet.

**Step 5: Write minimal implementation**

1. Add import services for paste text and PDF.
2. Reuse `crates/importing` normalization and segmentation logic where it is browser-compatible.
3. Add a small Rust bridge to vendored `pdf.js` assets for browser-side extraction.
4. Persist the original asset plus the derived paper/blocks/progress records into browser-local storage.
5. Surface new papers in the library view with status and progress.

**Step 6: Run tests to verify they pass**

Run:
- `cargo test --test web_import_paste_flow -q`
- `cargo test --test web_import_pdf_flow -q`

Expected: PASS.

**Step 7: Commit**

```bash
git add crates/web-app/src/import crates/web-app/assets/pdfjs crates/importing/src crates/web-app/src/routes/import.rs crates/web-app/src/routes/library.rs tests/web_import_paste_flow.rs tests/web_import_pdf_flow.rs
git commit -m "feat: add local paper import flows for web mvp"
```

### Task 4: Build the immersive reading loop and progress restore

**Files:**
- Create: `crates/web-app/src/reading/mod.rs`
- Create: `crates/web-app/src/reading/session.rs`
- Create: `crates/web-app/src/reading/navigation.rs`
- Create: `crates/web-app/src/components/progress_indicator.rs`
- Create: `crates/web-app/src/components/block_navigator.rs`
- Modify: `crates/web-app/src/routes/read.rs`
- Test: `tests/web_reading_progress_restore.rs`
- Test: `tests/web_block_navigation.rs`

**Step 1: Write the failing navigation test**

Create `tests/web_block_navigation.rs` that loads a paper with multiple blocks, advances to the next block, goes back once, and verifies the stored `current_block_index` tracks each move.

**Step 2: Run test to verify it fails**

Run: `cargo test --test web_block_navigation -q`

Expected: FAIL because the reading route does not yet manage reading state.

**Step 3: Write the failing restore test**

Create `tests/web_reading_progress_restore.rs` that seeds a non-zero `ReadingProgress`, reloads the reading session, and verifies the same block is restored instead of reopening block `0`.

**Step 4: Run test to verify it fails**

Run: `cargo test --test web_reading_progress_restore -q`

Expected: FAIL because progress rehydration is not implemented.

**Step 5: Write minimal implementation**

1. Add a reading-session module that loads blocks and progress by `paper_id`.
2. Implement previous/next navigation and immediate progress persistence.
3. Render block index and percentage through reusable reading components.
4. Keep keyboard navigation and reload behavior inside the session layer so later annotation and AI actions reuse the same state owner.

**Step 6: Run tests to verify they pass**

Run:
- `cargo test --test web_block_navigation -q`
- `cargo test --test web_reading_progress_restore -q`

Expected: PASS.

**Step 7: Commit**

```bash
git add crates/web-app/src/reading crates/web-app/src/components/progress_indicator.rs crates/web-app/src/components/block_navigator.rs crates/web-app/src/routes/read.rs tests/web_block_navigation.rs tests/web_reading_progress_restore.rs
git commit -m "feat: add immersive reading loop and progress restore"
```

### Task 5: Add selection annotations and vocabulary auto-collection

**Files:**
- Create: `crates/web-app/src/selection/mod.rs`
- Create: `crates/web-app/src/selection/popover.rs`
- Create: `crates/web-app/src/annotations/mod.rs`
- Create: `crates/web-app/src/annotations/anchors.rs`
- Create: `crates/web-app/src/components/selection_popover.rs`
- Create: `crates/web-app/src/components/annotation_renderer.rs`
- Modify: `crates/web-app/src/routes/read.rs`
- Modify: `crates/web-app/src/routes/vocabulary.rs`
- Test: `tests/web_annotation_round_trip.rs`
- Test: `tests/web_vocabulary_auto_collect.rs`

**Step 1: Write the failing annotation test**

Create `tests/web_annotation_round_trip.rs` that stores a highlight with quote, prefix, suffix, and offsets, reloads the paper, and verifies the annotation is still attached to the same paper/block pair.

**Step 2: Run test to verify it fails**

Run: `cargo test --test web_annotation_round_trip -q`

Expected: FAIL because the reading view has no selection or annotation behavior yet.

**Step 3: Write the failing vocabulary auto-collection test**

Create `tests/web_vocabulary_auto_collect.rs` that simulates a lookup event for the same term twice and verifies:
- the term exists only once
- `lookup_count` increments
- the vocabulary view can list the saved entry

**Step 4: Run test to verify it fails**

Run: `cargo test --test web_vocabulary_auto_collect -q`

Expected: FAIL because lookup events are not yet wired to vocabulary persistence.

**Step 5: Write minimal implementation**

1. Add selection-state handling for the reading route.
2. Implement annotation creation with resilient anchors.
3. Render annotations in the reading view and list vocabulary entries in the vocabulary route.
4. Wire successful lookup events to vocabulary upsert logic.

**Step 6: Run tests to verify they pass**

Run:
- `cargo test --test web_annotation_round_trip -q`
- `cargo test --test web_vocabulary_auto_collect -q`

Expected: PASS.

**Step 7: Commit**

```bash
git add crates/web-app/src/selection crates/web-app/src/annotations crates/web-app/src/components/selection_popover.rs crates/web-app/src/components/annotation_renderer.rs crates/web-app/src/routes/read.rs crates/web-app/src/routes/vocabulary.rs tests/web_annotation_round_trip.rs tests/web_vocabulary_auto_collect.rs
git commit -m "feat: add web annotations and vocabulary capture"
```

### Task 6: Expand the gateway into the MVP AI service boundary

**Files:**
- Modify: `crates/gateway/Cargo.toml`
- Create: `crates/gateway/src/config.rs`
- Create: `crates/gateway/src/logging.rs`
- Create: `crates/gateway/src/routes/mod.rs`
- Create: `crates/gateway/src/routes/translate.rs`
- Create: `crates/gateway/src/routes/explain.rs`
- Create: `crates/gateway/src/services/mod.rs`
- Create: `crates/gateway/src/services/ai_client.rs`
- Modify: `crates/gateway/src/main.rs`
- Create: `docs/runtime-config.md`
- Create: `scripts/run_gateway.sh`
- Test: `tests/gateway_translate_contract.rs`
- Test: `tests/gateway_explain_contract.rs`

**Step 1: Write the failing translate-contract test**

Create `tests/gateway_translate_contract.rs` that posts a translation request to the gateway app, stubs the upstream AI client, and verifies a structured JSON response with translation text and a normalized term field.

**Step 2: Run test to verify it fails**

Run: `cargo test --test gateway_translate_contract -q`

Expected: FAIL because the gateway still has no AI routes or service layer.

**Step 3: Write the failing explain-contract test**

Create `tests/gateway_explain_contract.rs` that posts a term-explanation request and verifies a structured JSON response containing `explanation`, optional `background`, and optional `caution`.

**Step 4: Run test to verify it fails**

Run: `cargo test --test gateway_explain_contract -q`

Expected: FAIL because the explain route and DTOs do not exist yet.

**Step 5: Write minimal implementation**

1. Add config loading for gateway URL, bind address, token, and upstream AI key.
2. Add tracing bootstrap that writes structured logs under `logs/YYYY-MM-DD_HHMMSS.log`.
3. Implement `POST /api/translate` and `POST /api/explain` with an injectable AI client seam.
4. Document the runtime environment variables under `docs/runtime-config.md`.
5. Add `scripts/run_gateway.sh` as the standard startup entrypoint.

**Step 6: Run tests to verify they pass**

Run:
- `cargo test --test gateway_translate_contract -q`
- `cargo test --test gateway_explain_contract -q`

Expected: PASS.

**Step 7: Commit**

```bash
git add crates/gateway docs/runtime-config.md scripts/run_gateway.sh tests/gateway_translate_contract.rs tests/gateway_explain_contract.rs
git commit -m "feat: add gateway ai service boundary"
```

### Task 7: Wire translation and explanation into the reading experience

**Files:**
- Create: `crates/web-app/src/ai/mod.rs`
- Create: `crates/web-app/src/ai/client.rs`
- Create: `crates/web-app/src/components/ai_panel.rs`
- Create: `crates/web-app/src/components/translation_panel.rs`
- Modify: `crates/web-app/src/routes/read.rs`
- Modify: `crates/web-app/src/routes/settings.rs`
- Test: `tests/web_selection_translate_flow.rs`
- Test: `tests/web_explain_panel_flow.rs`
- Test: `tests/web_ai_offline_state.rs`

**Step 1: Write the failing translation-flow test**

Create `tests/web_selection_translate_flow.rs` that simulates a selection translate action and verifies:
- the web client calls the gateway boundary with selected text and context
- the translation result is persisted as an AI record
- the translation panel displays the bilingual result

**Step 2: Run test to verify it fails**

Run: `cargo test --test web_selection_translate_flow -q`

Expected: FAIL because the web app does not yet have an AI client or translation panel.

**Step 3: Write the failing explain-flow test**

Create `tests/web_explain_panel_flow.rs` that requests a term explanation and verifies the AI panel renders the response in the configured explanation language.

**Step 4: Run test to verify it fails**

Run: `cargo test --test web_explain_panel_flow -q`

Expected: FAIL because the explanation panel and settings integration are not implemented.

**Step 5: Write the failing offline-state test**

Create `tests/web_ai_offline_state.rs` that simulates an unavailable gateway and verifies the reading view stays usable while the AI controls surface an explicit disabled or retry state.

**Step 6: Run test to verify it fails**

Run: `cargo test --test web_ai_offline_state -q`

Expected: FAIL because AI failure handling is not yet modeled in the UI.

**Step 7: Write minimal implementation**

1. Add a web AI client with explicit request and error states.
2. Add translation and explanation panels to the reading route.
3. Persist successful AI responses as local AI records.
4. Add settings-backed explanation language handling.
5. Keep offline failure states non-blocking for the core reading flow.

**Step 8: Run tests to verify they pass**

Run:
- `cargo test --test web_selection_translate_flow -q`
- `cargo test --test web_explain_panel_flow -q`
- `cargo test --test web_ai_offline_state -q`

Expected: PASS.

**Step 9: Commit**

```bash
git add crates/web-app/src/ai crates/web-app/src/components/ai_panel.rs crates/web-app/src/components/translation_panel.rs crates/web-app/src/routes/read.rs crates/web-app/src/routes/settings.rs tests/web_selection_translate_flow.rs tests/web_explain_panel_flow.rs tests/web_ai_offline_state.rs
git commit -m "feat: add translation and explanation flows to reading view"
```

### Task 8: Harden the MVP and close the delivery loop

**Files:**
- Modify: `README.md`
- Modify: `docs/plans/INDEX.md`
- Modify: `progress.txt`
- Modify: `lesson.md`
- Modify: `crates/web-app/src/routes/library.rs`
- Modify: `crates/web-app/src/routes/import.rs`
- Modify: `crates/web-app/src/routes/read.rs`
- Test: `tests/web_library_listing.rs`
- Test: `tests/web_import_error_state.rs`

**Step 1: Write the failing library-listing test**

Create `tests/web_library_listing.rs` that seeds multiple papers and verifies the library view can show title, source, status, and progress summary for each entry.

**Step 2: Run test to verify it fails**

Run: `cargo test --test web_library_listing -q`

Expected: FAIL because the initial library route only exposes placeholder rendering.

**Step 3: Write the failing import-error test**

Create `tests/web_import_error_state.rs` that forces a PDF extraction failure and verifies the UI leaves a visible failed paper state with a retry action instead of dropping the import.

**Step 4: Run test to verify it fails**

Run: `cargo test --test web_import_error_state -q`

Expected: FAIL because the import flow does not yet model an end-user error state.

**Step 5: Write minimal implementation**

1. Finish library summaries and import/read empty states.
2. Add user-facing import failure handling and retry entrypoints.
3. Update `README.md` with MVP startup instructions and the split web/gateway architecture.
4. Keep `docs/plans/INDEX.md`, `progress.txt`, and `lesson.md` in sync with milestone completion.

**Step 6: Run repository verification**

Run:
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`

Expected: PASS.

**Step 7: Commit**

```bash
git add README.md docs/plans/INDEX.md progress.txt lesson.md crates/web-app/src/routes/library.rs crates/web-app/src/routes/import.rs crates/web-app/src/routes/read.rs tests/web_library_listing.rs tests/web_import_error_state.rs
git commit -m "feat: harden cargo-native web mvp"
```

