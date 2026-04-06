# Import Pipeline Skeleton Design (M1.2 — Minimal Runnable Paste Text Import)

**Status:** Approved
**Date:** 2026-03-28
**Project:** Paper Immersive Reader

**Scope:** Milestone M1.2 — Import pipeline skeleton with a runnable `paste text` path.

**Goal:** Define a minimal import pipeline abstraction and one concrete importer (`paste text`) that produces existing `crates/core` domain outputs (`Paper`, `Block`, `ReadingProgress`) and transitions `Paper.status` through the expected lifecycle.

**Tech Stack:** Rust, Cargo workspace, root integration tests in `/tests`.

---

## 1. Constraints (source of truth)

**Import sources required by product docs:**
- PDF
- Paste text
- URL
- DOI
- Zotero

However, this milestone is explicitly a **skeleton**, and we will only make **Paste Text** runnable.

**Where requirements are defined (must stay consistent):**
- `PRD.md` (import sources / pipeline requirements)
- `APP_FLOW.md` (import wizard + lifecycle state machine)
- `BACKEND_STRUCTURE.md` (pipeline responsibilities and future entities)

**Core domain constraints (already implemented):**
- `crates/core/src/domain/paper.rs` provides:
  - `PaperSourceType::{Pdf,Paste,Url,Doi,Zotero}`
  - `PaperStatus::{Created,Ingesting,ReadyText,ReadyOcr,Failed}`
- `crates/core/src/domain/block.rs` provides `Block::try_new(...) -> Result<Block, DomainError>` enforcing non-empty trimmed text.

**Architectural boundary:**
- `crates/core` stays persistence-agnostic and import-logic-free.
- Import pipeline logic lives outside `crates/core`.

---

## 2. What is in scope

### 2.1 Import pipeline abstraction
- Request/Result types describing what an importer consumes and produces.
- Error type for pipeline-level failures.
- A trait representing a pipeline entrypoint.

### 2.2 Runnable importer: Paste Text
A concrete implementation that:
1. Accepts plain text (+ optional title/source_ref)
2. Constructs a `Paper` with:
   - `source_type = PaperSourceType::Paste`
   - `status` transitions: `Created/ Ingesting -> ReadyText`
3. Splits text into blocks with a minimal, low-regret segmentation rule
4. Constructs `Block` values via `Block::try_new`
5. Produces an initial `ReadingProgress`

---

## 3. What is explicitly out of scope

- PDF parsing / PDF.js extraction
- Header/footer removal
- OCR
- URL fetching
- DOI resolution
- Zotero API integration
- Any persistence/storage layer
- Gateway jobs/queues
- AI chunking
- UI / import wizard implementation

---

## 4. Placement & crate layout

Create a new crate:
- `crates/importing`

Suggested package name:
- `paper-immersive-reader-importing`

Dependencies:
- `paper-immersive-reader-core` (path dependency)

---

## 5. Types and interfaces

### 5.1 Request

```rust
pub enum ImportRequest {
    PasteText {
        text: String,
        title: Option<String>,
        source_ref: Option<String>,
    },
}
```

### 5.2 Result

```rust
pub struct ImportResult {
    pub paper: Paper,
    pub blocks: Vec<Block>,
    pub reading_progress: ReadingProgress,
}
```

### 5.3 Errors

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportPipelineError {
    EmptyInput,
    NoBlocksProduced,
    Domain(DomainError),
}
```

Notes:
- `DomainError` comes from `paper-immersive-reader-core`.
- Avoid `anyhow` in core skeleton; keep error surface simple.

### 5.4 Pipeline trait

```rust
pub trait ImportPipeline {
    fn import(&self, request: ImportRequest) -> Result<ImportResult, ImportPipelineError>;
}
```

---

## 6. Minimal segmentation rule (Paste Text)

Goal: produce stable blocks without overfitting.

Algorithm:
1. Split on **double newlines** (`"\n\n"`)
2. `trim()` each segment
3. Drop empty segments
4. If no segments remain: `NoBlocksProduced`

This matches typical paragraph boundaries for pasted text, without requiring NLP.

---

## 7. Lifecycle mapping

This milestone uses the already-defined states:
- Start: `PaperStatus::Ingesting` (or `Created` immediately followed by `Ingesting`)
- Success: `PaperStatus::ReadyText`
- Failure: `PaperStatus::Failed` (optional for now; may remain future enhancement)

For the skeleton, we can represent failure by returning `ImportPipelineError` and letting the caller set `Paper.status` if needed.

---

## 8. Testing strategy

All tests are root integration tests under `tests/`.

### Tests
1. `tests/import_paste_smoke.rs`
   - imports a multi-paragraph text
   - asserts:
     - `paper.source_type == Paste`
     - `paper.status == ReadyText`
     - `blocks.len() > 0`
     - `blocks[0].order_index == 0`

2. `tests/import_paste_segmentation.rs`
   - verifies split by `\n\n`
   - empty paragraphs are ignored

3. `tests/import_paste_empty.rs`
   - empty/whitespace-only text returns `EmptyInput` (or `NoBlocksProduced`, depending on final choice)

---

## 9. Future extension points (non-binding)

The `ImportRequest` enum can later grow variants:
- `Pdf { ... }`
- `Url { ... }`
- `Doi { ... }`
- `Zotero { ... }`

The pipeline may later return richer artifacts (e.g. `ExtractedPage`, `PaperAsset`), but those are deferred to later phases.
