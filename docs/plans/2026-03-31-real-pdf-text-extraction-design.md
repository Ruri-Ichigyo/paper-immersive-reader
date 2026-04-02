# Real PDF Text Extraction Design

**Status:** Approved
**Date:** 2026-03-31

## Goal
Replace the current deterministic PDF extraction fake with a real embedded-text extraction path for `ImportRequest::PdfDocument`, while keeping the existing import pipeline structure and OCR boundary behavior simple.

This milestone only supports PDFs that already contain extractable embedded text. If the pipeline cannot obtain usable text, it should return `OcrRequired`.

## Architecture
Keep the current `PdfTextExtractor` seam in `crates/importing` and continue to let `DefaultImportPipeline` own import orchestration.

Do not add a new crate in this milestone. Instead, add the real PDF extraction implementation in a dedicated module inside `crates/importing`, so `lib.rs` remains the pipeline entrypoint and the PDF-specific extraction details stay isolated.

Recommended shape:
- `crates/importing/src/lib.rs`
  - owns `ImportRequest`, `ImportPipeline`, `DefaultImportPipeline`, and orchestration
- `crates/importing/src/pdf.rs` or `crates/importing/src/pdf/mod.rs`
  - owns the real embedded-text extraction implementation
  - implements the existing `PdfTextExtractor` seam

This follows the user-approved direction: choose the minimal in-crate implementation approach, but absorb some structure from a layered design so extraction details are not scattered through the pipeline entrypoint.

## Scope
In scope:
- real extraction of embedded text from PDF bytes
- wiring the real extractor into the existing PDF import path
- keeping paste-text and PDF post-extraction handling shared
- mapping all “no usable text” outcomes to `OcrRequired`
- adding real-path integration coverage

Out of scope:
- OCR execution
- scanned PDF rasterization
- multi-column reading-order repair
- advanced layout cleanup
- metadata extraction beyond what naturally falls out of the current pipeline
- introducing a dedicated PDF crate

## Data Flow
For `ImportRequest::PdfDocument`:

1. `DefaultImportPipeline` receives PDF bytes.
2. The pipeline calls the real `PdfTextExtractor` implementation.
3. The extractor returns extracted text as a `String`.
4. The pipeline applies the same minimal text handling already used for paste import:
   - `trim`
   - split on double newlines
   - trim segments
   - drop empty segments
5. If blocks are produced:
   - create `Paper` with `source_type = Pdf`
   - set `status = ReadyText`
   - build `Block[]`
   - build initial `ReadingProgress`
6. If no usable text is available after extraction and segmentation:
   - return `ImportPipelineError::OcrRequired`

Important design choice: the extractor is only responsible for obtaining raw embedded text. Text usability and block formation stay in the pipeline so the post-extraction behavior for PDF remains aligned with paste import.

## Error Handling
Keep the PDF error model intentionally simple for this milestone.

Use these rules:
- extractor fails to obtain text -> `OcrRequired`
- extractor returns empty or whitespace-only text -> `OcrRequired`
- segmentation yields zero usable blocks -> `OcrRequired`
- domain construction failures still map through existing `Domain(...)` handling

This means the milestone does not distinguish between “PDF parsing failed” and “text exists only via OCR.” Both are treated as the same OCR boundary for now.

Why this is the right trade-off now:
- matches the approved scope
- removes unnecessary branching from the import pipeline
- keeps room to add richer reasons later without redesigning the full flow

## Testing Strategy
Use integration-test-first development and add real-path coverage, not only fake seam coverage.

Required coverage:
1. Real PDF extraction happy path
   - use a stable fixture PDF with embedded text
   - assert `ReadyText`, `PaperSourceType::Pdf`, and non-empty blocks
2. Real PDF extraction with no usable text
   - use a fixture that yields no usable embedded text, or an equivalent real extraction outcome
   - assert `OcrRequired`
3. Keep existing seam-level tests where useful
   - deterministic seam tests may remain for fast boundary checks
   - but real extraction tests must exist so the milestone is not validated only by fakes
4. Full verification
   - `cargo fmt --check`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo test --workspace`

## Implementation Notes
Recommended implementation direction:
- preserve `PdfTextExtractor` as the abstraction seam
- add one public constructor path for the real extractor, while keeping test-oriented constructors if they remain useful
- keep PDF-specific extraction code in a dedicated module
- avoid introducing a second abstraction layer unless the library choice forces it

## Deliverables
This milestone is complete when the repo contains:
- a real embedded-text extractor implementation in `crates/importing`
- PDF import wired to the real extractor path
- unified `OcrRequired` handling when no usable text is obtained
- integration tests proving the real PDF path works
- passing fmt, clippy, and full test verification
