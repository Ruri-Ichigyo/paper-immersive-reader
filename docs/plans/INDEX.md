# Plans Index

## Purpose

This directory stores planning artifacts for the repository.

- `*-design.md` documents the intended design, scope, constraints, and decisions for a topic.
- `*-implementation.md` documents the execution plan for that topic.

## Naming Rules

Use:

- `YYYY-MM-DD-<topic>-design.md`
- `YYYY-MM-DD-<topic>-implementation.md`

Where `<topic>` is a short kebab-case identifier shared by both files.

## Status Rules

Recommended document-level status values:

- Design documents: `Draft`, `Approved`
- Implementation documents: `Planned`, `In Progress`, `Done`

Completed work should stay in this directory. Update status instead of moving files.

## Minimum Expectations

Keep these expectations lightweight. Prefer light normalization of the top summary fields for new or updated docs rather than retrofitting older plans into a strict template.

### Design documents
Should usually include a short top summary with fields such as:
- Status
- Date
- Goal
- Architecture or equivalent design summary
- Scope / non-goals / constraints as needed

### Implementation documents
Should usually include a short top summary with fields such as:
- Status
- Date
- Goal
- Architecture
- Tech Stack
- Ordered tasks or execution steps
- Verification commands where applicable

## Current Topics

| Topic | Date | Design | Implementation | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| agents-governance | 2026-03-27 | yes | yes | Done | Root `AGENTS.md` governance artifact is in place |
| rust-workspace-scaffold | 2026-03-27 | yes | yes | Done | Cargo workspace, crate scaffold, and logging bootstrap landed |
| import-pipeline-skeleton | 2026-03-28 | yes | yes | Done | Paste-text import path is implemented and covered by tests |
| reader-domain-models | 2026-03-28 | yes | yes | Done | Core domain types and invariants are implemented in `crates/core` |
| pdf-extraction-skeleton | 2026-03-28 | yes | yes | Done | PDF seam and `OcrRequired` boundary are implemented |
| plans-governance | 2026-03-31 | yes | yes | Done | `INDEX.md` and plan metadata normalization rules are active |
| real-pdf-text-extraction | 2026-03-31 | yes | yes | Done | Real embedded-text extraction fixtures and default path are implemented |
| ocr-job-model-paddle | 2026-04-02 | yes | yes | Done | OCR job model, transitions, and repository seam are implemented |
| ocr-enqueue-local-executor | 2026-04-03 | yes | yes | Done | OCR enqueue orchestration and local executor boundary are implemented |
| ocr-json-persistence-local-queue | 2026-04-05 | yes | yes | Done | Application crate migration and JSON-backed OCR queue closure are implemented |
| cli-ocr-managed-assets | 2026-04-05 | yes | yes | Done | Managed PDF assets, script-backed OCR runner, and CLI OCR commands are implemented |
| sqlite-persistence-auto-migration | 2026-04-06 | yes | yes | Done | SQLite is the runtime store and auto-migrates legacy JSON on first open |

## Maintenance Rules

- When adding a new design doc, use the shared `<topic>` naming immediately.
- When adding an implementation doc, reuse the same `<topic>` and add or update the matching row here.
- If only one file exists for a topic, mark the missing pair explicitly in this index.
- Prefer lightweight normalization over rewriting historical technical content.
