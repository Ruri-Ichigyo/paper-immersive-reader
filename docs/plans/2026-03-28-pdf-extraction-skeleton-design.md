# PDF Extraction Skeleton Design

**Status:** Approved
**Date:** 2026-03-28

## Goal
Add the next import milestone after paste-text: a minimal PDF extraction path with an explicit OCR boundary.

This milestone must let the system ingest a PDF source through the importing layer, attempt embedded-text extraction first, and then either:
- succeed with `READY_TEXT` and normal `Paper` / `Block` / `ReadingProgress` output, or
- stop cleanly with a structured OCR-required outcome.

Real OCR is explicitly out of scope.

## Architecture
Keep the current crate boundaries intact:

- `crates/core`
  - remains domain-only
  - no PDF parsing or import orchestration logic
- `crates/importing`
  - owns import orchestration
  - gains PDF request handling, PDF extractor seam, and OCR-boundary signaling
- future OCR milestone
  - plugs into the importing seam without redesigning the public API

This matches the current repo direction:
- core already owns `Paper`, `Block`, `ReadingProgress`, `PaperSourceType`, and `PaperStatus`
- importing already owns conversion from raw input into domain objects

## Public API Changes
Extend `crates/importing/src/lib.rs` with:

- `ImportRequest::PdfDocument { bytes, title, source_ref }`
- additional `ImportPipelineError` variants for PDF flow, such as:
  - `PdfExtractionFailed`
  - `OcrRequired`
- a small extractor seam:
  - `trait PdfTextExtractor`
  - deterministic test implementation hooks

Important constraint: OCR is represented now as a boundary/result, not implemented now.

## Import Flow
For PDF import:
1. Receive PDF bytes plus optional `title` and `source_ref`.
2. Attempt embedded-text extraction through `PdfTextExtractor`.
3. Inspect extraction output.
   - If usable text exists:
     - segment text into blocks using the same minimal rule as paste import
     - create `Paper` with `source_type = Pdf` and `status = ReadyText`
     - create `ReadingProgress` at block index `0`
     - return `ImportResult`
   - If extracted text is empty or unusable:
     - return structured OCR-boundary error/result
     - do not attempt OCR in this milestone

## Usability Threshold
Keep “usable text” deterministic and intentionally small for this milestone.

Suggested rule:
- concatenate extracted text
- trim whitespace
- if the resulting text is empty, or segmentation yields zero blocks, treat it as OCR-required

More advanced heuristics can be added later.

## OCR Boundary
Reserve the next step without implementing it.

Recommended baseline shape:
- `ImportPipelineError::OcrRequired`

A richer reason enum can be added later if needed, but is not required for this milestone.

## Testing Strategy
Use integration-test-first development, consistent with the current importing milestone.

Planned tests:
1. PDF request type / importer API smoke test
2. Successful PDF import through a deterministic stub extractor
3. Empty extracted text maps to `OcrRequired`
4. Extractor failure maps to `PdfExtractionFailed`
5. Successful PDF extraction produces:
   - `PaperSourceType::Pdf`
   - `PaperStatus::ReadyText`
   - non-empty blocks
   - `ReadingProgress` at block `0`

## Dependency Choice
Do not commit the public import pipeline directly to a concrete PDF parser crate in this milestone.

Instead:
- define the seam now
- use a deterministic stub/fake extractor in tests
- leave the real extractor implementation for the next focused milestone

Why:
- keeps this milestone bounded
- avoids dragging layout parsing details into the API too early
- keeps later crate selection/replacement cheap

## Out of Scope
Explicitly deferred:
- real OCR
- scanned-PDF rasterization
- multi-column reading-order repair
- header/footer removal for PDF pages
- raw page extraction persistence
- gateway jobs
- UI/upload plumbing
- URL / DOI / Zotero PDF acquisition

## Deliverables
This milestone is complete when the repo contains:
- updated importing API for PDF requests
- a PDF extractor seam in `crates/importing`
- OCR-boundary signaling
- deterministic integration tests
- passing `cargo fmt --check`
- passing `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- passing `cargo test --workspace`
- a milestone entry in `progress.txt`
- `lesson.md` update only if actual debugging occurs
