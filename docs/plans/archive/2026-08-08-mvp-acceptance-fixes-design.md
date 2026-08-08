# MVP Acceptance Fixes Design

**Status:** Approved
**Date:** 2026-08-08

## Goal

Fix the three critical gaps blocking MVP release and address four polish-level issues identified in the 2026-08-08 product acceptance review, bringing the web app to full PRD §6 MVP compliance and improving UX toward IMPLEMENTATION_PLAN Phase 2 readiness.

## Context

The acceptance review found all four verification gates passing (fmt, clippy, 89 tests, wasm32 build) and 6 of 7 PRD §6 success criteria fully met. However, three issues block browser usability and four UX gaps remain within MVP scope.

## Architecture

No new crates. All changes are within `crates/web-app` (and minimal `crates/core` touches for vocabulary update support). The architecture stays split: browser-local state via `InMemoryStore` + `IndexedDbStore`, gateway as thin AI proxy.

## Scope

### Phase 1 — Critical Fixes (block release)

| ID | Gap | FR / Flow ref | Summary |
|---|---|---|---|
| G1 | pdf.js assets not deployed | FR-03, Flow A | `index.html` has no pdf.js script; `assets/pdfjs/` is empty. PDF import fails in browser. |
| G2 | Paper ID collision on reload | FR-02 | `WebImportService` uses counter from 1; after reload, new imports overwrite existing papers in IndexedDB. |
| G3 | gateway_url not restored | FR-17, Settings flow | `init_persistence` loads settings from IndexedDB but does not sync `gateway_url` signal. |

### Phase 2 — MVP Polish (UX gaps within MVP scope)

| ID | Gap | FR / Flow ref | Summary |
|---|---|---|---|
| G4 | No inline annotation decoration | Flow G §3.7 | Annotations shown only as list below block; no in-text highlight/underline. |
| G5 | No keyboard navigation | APP_FLOW §4.3 | Arrow keys / Page Up-Down not wired to block navigation. |
| G6 | Vocabulary management incomplete | FR-30 | Cannot mark as learned, add note, or tag from UI. |
| G7 | No header/footer removal | FR-05 | PDF import does no noise filtering; repeated headers/footers pollute blocks. |

### Phase 3 — Feature Completeness (toward full PRD MVP)

| ID | Gap | FR ref | Summary |
|---|---|---|---|
| G8 | No paragraph translation | FR-14 | Only selection-level translation; no "translate this block" button. |
| G9 | Naive segmentation | FR-12 (partial) | Current `split("\n\n")` misses single-newline paragraphs; no sentence-level fallback. |

## Non-goals

- AI adaptive chunking via LLM (FR-12 full) — deferred to a later milestone
- OCR in browser (FR-06) — CLI path exists; browser OCR is Phase 4
- Export, S3 sync, Zotero (FR-35~42) — P1/P2
- URL/DOI/Zotero import (FR-01.3/4/5) — P1/P2

## Constraints

- All changes must pass the four verification gates: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features`
- No new crate dependencies beyond what's already in `Cargo.toml` (pdf.js is a JS asset, not a Rust dep)
- IndexedDB schema version must not break existing user data (bump version only if schema change is additive)
- Browser-only features (`#[cfg(target_arch = "wasm32")]`) must have native stubs so tests still pass

## Decisions

1. **pdf.js integration**: Use Trunk's `data-trunk rel="copy-file"` to ship pdf.js assets, and add an inline `<script>` in `index.html` that loads pdf.js and registers the `pdfjs_extract_text` global bridge function. This avoids adding a JS build toolchain.

2. **Paper ID**: Replace counter with `short_id("paper")` (UUID v4 first segment), consistent with how annotation, vocabulary, and AI record IDs are already generated.

3. **Inline annotation decoration**: Render block text as segments split by annotation offsets, wrapping annotated ranges in `<mark>` / `<u>` / `<span>` tags. Recompute on each render from the annotation list — no DOM mutation needed.

4. **Vocabulary update**: Add `update_vocabulary` method to `AppStorage` trait and both stores. The IndexedDB `upsert_vocabulary` already handles note updates; extend it to also accept `learned` flag changes.

5. **Header/footer removal**: Implement a lightweight heuristic in `segment_text` — detect lines that repeat across multiple "pages" (separated by form-feed `\x0c` from pdf.js) and strip them. Keep it simple; full multi-column detection is Phase 2.

6. **Segmentation improvement**: In addition to `\n\n`, also split on single `\n` when the resulting segments are non-trivial (> 20 chars). This catches PDFs where paragraphs are separated by single newlines.
