# CLI OCR Managed Assets Design

**Status:** Approved
**Date:** 2026-04-05

## Goal
Expose the new persisted local OCR workflow through a minimal CLI entrypoint, while ensuring OCR jobs always execute against managed PDF assets stored under `database/` instead of fragile original file paths.

## Architecture
Keep the current crate layering intact and add only the minimum required pieces to make OCR execution runnable from the command line:

- `crates/application`
  - manages OCR state persistence
  - manages PDF asset persistence under `database/assets/`
  - provides a real `PaddleOcrRunner` implementation backed by a local script
- `crates/cli`
  - provides a tiny human-readable command interface
  - supports only `ocr run-next` and `ocr run <job-id>`
- `scripts/`
  - contains the OCR invocation script used by the real runner

The root of the system remains local-first. The CLI is only an entrypoint and must not become a second application layer.

## Scope
- Persist PDF source assets under `database/assets/<paper-id>.pdf`
- Add a real script-backed OCR runner implementation
- Add CLI commands:
  - `ocr run-next`
  - `ocr run <job-id>`
- Keep the default state file fixed at `database/app-state.json`
- Keep CLI output human-readable text only
- Use standard Rust library argument parsing for this milestone

## Non-Goals
- No `ocr list`
- No JSON CLI output
- No background polling or daemon mode
- No gateway HTTP endpoint
- No storage-engine upgrade beyond the current JSON snapshot
- No real PaddleOCR installer/bootstrap automation
- No generic configuration system

## Managed Asset Model
When a PDF import is processed in the application layer, the PDF bytes are copied into a managed location:

- `database/assets/<paper-id>.pdf`

This asset path becomes the authoritative OCR input source for that paper.

Design intent:
- queued OCR jobs must not depend on the original file still existing
- OCR execution must remain stable even if the user moves or deletes the original source file
- `Paper.source_ref` may still retain source metadata, but the OCR runner reads the managed asset

## CLI Command Surface
### `ocr run-next`
- reads `database/app-state.json`
- finds the first `Queued` OCR job
- executes it through the application layer
- prints a short success/failure message
- exits non-zero on failure or if no queued job exists

### `ocr run <job-id>`
- reads `database/app-state.json`
- executes the specified job if it exists and is queued
- prints a short success/failure message
- exits non-zero on failure

No other commands are included in this milestone.

## Runner Contract
The real OCR runner is implemented in Rust but delegates OCR execution to a local script:

- `scripts/run_paddle_ocr.sh`

Runner behavior:
1. resolve managed PDF path from the paper id
2. invoke the script with the PDF path
3. read OCR text from stdout
4. treat non-zero exit as execution failure

The script remains an adapter boundary. The application layer owns persistence and job transitions; the script owns external OCR command invocation.

## Data Flow
### Import path for PDFs
1. application layer creates a real `Paper`
2. PDF bytes are written to `database/assets/<paper-id>.pdf`
3. if embedded-text import succeeds:
- persist `Paper(status = ReadyText)`
- persist `Block[]`
- persist `ReadingProgress`
4. if import returns `OcrRequired`:
- persist `Paper(status = ReadyOcr)`
- persist `OcrJob(status = Queued)`

### CLI execution path
1. CLI loads the app-state snapshot from `database/app-state.json`
2. CLI selects a queued job or a requested job id
3. application layer resolves `database/assets/<paper-id>.pdf`
4. real runner invokes `scripts/run_paddle_ocr.sh <pdf-path>`
5. OCR text is returned through stdout
6. existing text segmentation is reused
7. application layer persists:
- `OcrJobResult`
- `Block[]`
- `ReadingProgress`
- `Paper.status = ReadyText`
- `OcrJob.status = Completed`

### Failure path
- missing managed asset file -> job `Failed`
- script exits non-zero -> job `Failed`
- OCR text empty or invalid for segmentation -> job `Failed`
- in all failure cases, `Paper.status` remains `ReadyOcr`

## Error Handling
- CLI prints short plain-text status lines only
- success exits with code `0`
- failure exits non-zero
- missing job, missing queued work, missing asset file, or OCR script failure are all visible as CLI failure outcomes
- no structured JSON error payloads are added in this milestone

## Testing Strategy
Required coverage:

1. managed asset persistence
- importing a PDF writes `database/assets/<paper-id>.pdf`

2. real runner behavior
- script stdout is captured as OCR text
- script failure becomes runner failure

3. CLI success paths
- `ocr run-next`
- `ocr run <job-id>`

4. CLI failure paths
- no queued jobs
- missing job id
- missing managed asset
- OCR script failure

5. regression coverage
- existing JSON persistence behavior still passes
- existing embedded-text import flow still passes
- existing executor state-transition behavior still passes

## Acceptance Criteria
- PDF OCR execution no longer depends on fragile original file paths
- managed PDF assets are stored under `database/assets/`
- `crates/cli` exposes a runnable `ocr run-next` command
- `crates/cli` exposes a runnable `ocr run <job-id>` command
- real OCR execution is delegated through `scripts/run_paddle_ocr.sh`
- successful CLI execution updates persisted OCR and reading state
- failures produce non-zero exit codes and keep `Paper.status = ReadyOcr`
