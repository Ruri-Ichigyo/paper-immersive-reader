# Archived Plans Index

## Purpose

This directory stores completed planning artifacts that were moved out of `docs/plans/` after the work landed.

- Archived on: 2026-04-06
- Source directory: [`../`](../)
- Active-plan index: [`../INDEX.md`](../INDEX.md)

## Archived Topics

| Topic | Date | Design | Implementation | Archived Status | Notes |
| --- | --- | --- | --- | --- | --- |
| agents-governance | 2026-03-27 | [design](./2026-03-27-agents-governance-design.md) | [implementation](./2026-03-27-agents-governance-implementation.md) | Completed | Root `AGENTS.md` governance artifact is in place |
| rust-workspace-scaffold | 2026-03-27 | [design](./2026-03-27-rust-workspace-scaffold-design.md) | [implementation](./2026-03-27-rust-workspace-scaffold-implementation.md) | Completed | Cargo workspace, crate scaffold, and logging bootstrap landed |
| import-pipeline-skeleton | 2026-03-28 | [design](./2026-03-28-import-pipeline-skeleton-design.md) | [implementation](./2026-03-28-import-pipeline-skeleton-implementation.md) | Completed | Paste-text import path is implemented and covered by tests |
| reader-domain-models | 2026-03-28 | [design](./2026-03-28-reader-domain-models-design.md) | [implementation](./2026-03-28-reader-domain-models-implementation.md) | Completed | Core domain types and invariants are implemented in `crates/core` |
| pdf-extraction-skeleton | 2026-03-28 | [design](./2026-03-28-pdf-extraction-skeleton-design.md) | [implementation](./2026-03-28-pdf-extraction-skeleton-implementation.md) | Completed | PDF seam and `OcrRequired` boundary are implemented |
| plans-governance | 2026-03-31 | [design](./2026-03-31-plans-governance-design.md) | [implementation](./2026-03-31-plans-governance-implementation.md) | Completed | `INDEX.md` and plan metadata normalization rules are active |
| real-pdf-text-extraction | 2026-03-31 | [design](./2026-03-31-real-pdf-text-extraction-design.md) | [implementation](./2026-03-31-real-pdf-text-extraction-implementation.md) | Completed | Real embedded-text extraction fixtures and default path are implemented |
| ocr-job-model-paddle | 2026-04-02 | [design](./2026-04-02-ocr-job-model-paddle-design.md) | [implementation](./2026-04-02-ocr-job-model-paddle-implementation.md) | Completed | OCR job model, transitions, and repository seam are implemented |
| ocr-enqueue-local-executor | 2026-04-03 | [design](./2026-04-03-ocr-enqueue-local-executor-design.md) | [implementation](./2026-04-03-ocr-enqueue-local-executor-implementation.md) | Completed | OCR enqueue orchestration and local executor boundary are implemented |
| ocr-json-persistence-local-queue | 2026-04-05 | [design](./2026-04-05-ocr-json-persistence-local-queue-design.md) | [implementation](./2026-04-05-ocr-json-persistence-local-queue-implementation.md) | Completed | Application crate migration and JSON-backed OCR queue closure are implemented |
| cli-ocr-managed-assets | 2026-04-05 | [design](./2026-04-05-cli-ocr-managed-assets-design.md) | [implementation](./2026-04-05-cli-ocr-managed-assets-implementation.md) | Completed | Managed PDF assets, script-backed OCR runner, and CLI OCR commands are implemented |
| sqlite-persistence-auto-migration | 2026-04-06 | [design](./2026-04-06-sqlite-persistence-auto-migration-design.md) | [implementation](./2026-04-06-sqlite-persistence-auto-migration-implementation.md) | Completed | SQLite is the runtime store and auto-migrates legacy JSON on first open |

## Archive Rules

- Keep archived files in their original names.
- Add newly completed topics here when moving them out of `docs/plans/`.
- Leave unfinished work in `docs/plans/` until the topic is complete.
