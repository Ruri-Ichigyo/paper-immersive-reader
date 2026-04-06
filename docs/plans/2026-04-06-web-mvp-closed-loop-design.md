# Web MVP Closed Loop Design

**Status:** Approved
**Date:** 2026-04-06

## Goal
Deliver the remaining MVP closed loop from `PRD.md` as a web-first, local-first product: import a paper, parse it into readable blocks, read block-by-block, translate a selection, request AI explanation, annotate text, restore progress, and auto-collect vocabulary.

## Architecture
Use a split architecture that preserves the product target while staying inside this repository's Cargo-only governance:

- `crates/web-app`: a Rust/WASM web client built as a local-first PWA shell for library, import, reading, vocabulary, and settings flows.
- `crates/gateway`: a thin Rust HTTP gateway for AI translation/explanation requests and future OCR/sync expansion.
- `crates/core`: shared domain types that remain product-language centric.
- `crates/application`: shared orchestration, persistence contracts, DTOs, and service-level validation reused by the web app and gateway where appropriate.
- `crates/importing`: reusable text normalization and segmentation logic, plus native-side extraction helpers that can be reused by the gateway for fallback processing.

The browser remains the source of truth for day-to-day reading data. The gateway is optional for core reading and required only for AI-backed actions in MVP.

## Constraint Resolution
There is an explicit constraint mismatch between the product-direction documents and repository governance:

- `PRD.md` and the earlier `TECH_STACK.md` describe a web-first product and recommend a JavaScript/TypeScript stack.
- `AGENTS.md` governs this repository as a Rust + Cargo project and names `cargo` as the approved build and dependency entrypoint.

This design resolves that mismatch by preserving the PRD product scope while switching the implementation plan to a Cargo-native stack. The web client may consume vendored browser assets such as prebuilt `pdf.js` files, but the repository does not introduce a second package manager as part of MVP execution.

## MVP Scope
The remaining MVP closed loop includes:

- Library page with imported-paper list and status/progress summary
- Import flow for:
  - local PDF
  - pasted text
- Browser-local persistence for:
  - papers
  - blocks
  - reading progress
  - annotations
  - vocabulary
  - AI records
  - settings
- Immersive reading page with:
  - next/previous block navigation
  - progress display
  - restore last position
- Selection actions:
  - translate word or sentence
  - explain term
  - explain sentence
  - annotate selection
- Vocabulary auto-collection from lookup actions
- Minimum settings surface for explanation language and tag management
- Offline-capable reading, annotation, progress restore, and vocabulary browsing after a paper has been imported

## Non-Goals
The following remain outside this MVP plan:

- URL/DOI ingestion beyond capturing metadata or a stub entry flow
- Zotero import or bi-directional sync
- S3 sync
- paragraph/full-paper summary
- structure extraction
- Q&A
- export flows
- scanned-PDF OCR as a required MVP success path
- multi-device conflict resolution

Current native OCR work in the repository remains useful as a future gateway-side extension, but it is not a gating requirement for the MVP success criteria in `PRD.md`.

## Product Flow
### 1. Import
1. User opens the import view from the library.
2. User chooses either PDF upload or paste-text import.
3. The app creates a local `Paper` record and transitions it through ingesting states.
4. For PDF:
   - the browser stores the original file locally
   - the app uses vendored `pdf.js` assets through a Rust/WASM bridge to extract text
   - the extracted text is normalized and segmented into canonical blocks
5. For pasted text:
   - the app normalizes the text and segments it into canonical blocks directly
6. On success the paper enters `ReadyText` and opens in the reading view.

### 2. Reading
1. The reading view loads the persisted block list and reading progress.
2. The reader navigates with previous/next controls or keyboard shortcuts.
3. Each movement updates persisted reading progress immediately.
4. Reloading the app restores the last block and progress state for the paper.

### 3. Selection and Assistance
1. The reader selects text in the current block.
2. A selection popover offers translate, explain, and annotate actions.
3. Translation and explanation requests go to the gateway when online.
4. Results render in the right-side assistance panel and are stored locally as AI records.
5. If the selected span is a word or short phrase, the term is added or updated in vocabulary storage.

### 4. Annotation
1. The reader highlights, underlines, tags, or adds a note to a selected range.
2. The app stores the annotation with resilient text anchors:
   - exact quote
   - prefix/suffix context
   - block hint
   - best-effort offsets
3. Reopening the paper rehydrates annotations from local storage.

## Cargo-Native Stack
### Web App
- `leptos` for the web UI
- `wasm-bindgen` / `web-sys` for browser integration
- `rexie` or equivalent Rust IndexedDB crate for local persistence
- `serde` for entity serialization
- vendored `pdf.js` worker/runtime assets loaded from a crate-local static directory

### Gateway
- `axum` for HTTP endpoints
- `tokio` for async runtime
- `tracing` for structured logs written to `logs/YYYY-MM-DD_HHMMSS.log`

### Shared
- existing Cargo workspace
- `crates/core` for domain types
- `crates/application` for orchestration, DTOs, and persistence contracts

## Proposed Repository Shape
This design keeps business code under crate-local `src/` directories and tests under `tests/`.

- `crates/web-app/`
  - `src/main.rs`
  - `src/app.rs`
  - `src/routes/`
  - `src/components/`
  - `src/storage/`
  - `src/selection/`
  - `src/ai/`
  - `assets/pdfjs/`
- `crates/gateway/`
  - `src/main.rs`
  - `src/config.rs`
  - `src/logging.rs`
  - `src/routes/`
  - `src/services/`
- `docs/`
  - runtime configuration and local-dev setup docs
- `scripts/`
  - startup helpers for the gateway and web app

## Data Model
The MVP browser-local data model should include:

### `Paper`
- `paper_id`
- `title`
- `source_type`
- `source_ref`
- `status`
- `created_at`
- `updated_at`

### `Block`
- `block_id`
- `paper_id`
- `order_index`
- `text`
- `block_hash`

### `ReadingProgress`
- `paper_id`
- `current_block_id`
- `current_block_index`
- `progress_percent`
- `updated_at`

### `Annotation`
- `annotation_id`
- `paper_id`
- `block_id`
- `annotation_type`
- `quote`
- `prefix`
- `suffix`
- `start_offset`
- `end_offset`
- `note`
- `tags`
- `anchoring_status`

### `VocabularyItem`
- `vocab_id`
- `term`
- `paper_id_first_seen`
- `block_id_first_seen`
- `lookup_count`
- `first_seen_at`
- `last_seen_at`
- `note`
- `learned`

### `AiRecord`
- `ai_record_id`
- `paper_id`
- `block_id`
- `record_type`
- `input`
- `output`
- `language`
- `created_at`

### `Settings`
- `explanation_language`
- `custom_tags`
- `ai_gateway_url`
- `ai_gateway_token_ref`

## Gateway Surface
The MVP gateway should stay minimal:

### `POST /api/translate`
Input:
- selected text
- selection type
- source language
- target language
- optional context

Output:
- bilingual translation result
- normalized lookup term when applicable

### `POST /api/explain`
Input:
- selected text
- explanation type (`term` or `sentence`)
- explanation language
- optional surrounding context

Output:
- explanation text
- optional short background section
- caution text when the answer is uncertain

### Not in MVP
- OCR endpoint implementation
- sync endpoints
- Zotero endpoints

## Error Handling
- Import failures must leave a visible paper state and actionable error message.
- Gateway failures must not block reading, annotation, progress, or vocabulary browsing.
- Offline mode must disable or degrade AI actions explicitly instead of failing silently.
- Annotation anchoring failures must preserve the record and mark it as `unanchored` rather than dropping data.

## Testing Strategy
All automated tests remain under `tests/` and run through `cargo test`.

Required test coverage for this plan:

1. Workspace and crate wiring
- web-app crate links into the workspace
- gateway crate still builds after route/service expansion

2. Browser-local persistence behavior
- paper/block/progress round trip
- annotation persistence
- vocabulary persistence and lookup-count updates

3. Import flows
- paste-text import produces blocks and an initial progress record
- PDF import produces blocks from extracted text through the browser bridge seam
- failed extraction yields a recoverable import error state

4. Reading loop
- next/previous block changes persist progress
- reload restores the last saved block

5. AI integration
- gateway request/response contracts
- web client handles success, offline, and upstream-failure states
- successful lookup creates or updates vocabulary entries

## Acceptance Criteria
The MVP plan is successful when the implemented product can demonstrate all of the following:

- A user imports a local PDF and reaches a readable block-by-block page.
- A user pastes text and reaches the same reading flow without a server dependency.
- Reading progress is visible and restored after reload.
- A selected word or sentence can be translated through the gateway.
- A selected term or sentence can be explained in the right-side panel through the gateway.
- An annotation survives reload.
- A looked-up term appears in the vocabulary list with repeat lookups merged.
- If the gateway is unavailable, the user can still read imported content, navigate blocks, view vocabulary, and manage annotations locally.

## Risks and Mitigations
### Browser PDF extraction complexity
- Risk: Rust/WASM PDF handling alone is not sufficient for reliable academic PDF extraction.
- Mitigation: use vendored `pdf.js` browser assets behind a small Rust bridge instead of inventing a parser inside the web app.

### Stack drift between planning docs
- Risk: older docs still reference Next.js/TypeScript and may cause implementation drift.
- Mitigation: treat this design and its paired implementation plan as the active source of truth for MVP execution until governance changes.

### AI dependency on network availability
- Risk: the MVP uses a gateway for AI actions, so translation/explanation are unavailable offline.
- Mitigation: keep the reading loop fully local and make AI degradation explicit in the UI.

