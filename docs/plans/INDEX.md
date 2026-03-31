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
| agents-governance | 2026-03-27 | yes | yes | Planned | Governance design approved; implementation plan present |
| rust-workspace-scaffold | 2026-03-27 | yes | yes | Planned | Initial workspace scaffold planning pair |
| import-pipeline-skeleton | 2026-03-28 | yes | yes | Planned | Paste-text import milestone planning pair |
| reader-domain-models | 2026-03-28 | yes | yes | Planned | Core reading domain planning pair |
| pdf-extraction-skeleton | 2026-03-28 | yes | yes | Planned | PDF extraction milestone planning pair |
| plans-governance | 2026-03-31 | yes | yes | Planned | Lightweight governance planning pair for the plans directory |

## Maintenance Rules

- When adding a new design doc, use the shared `<topic>` naming immediately.
- When adding an implementation doc, reuse the same `<topic>` and add or update the matching row here.
- If only one file exists for a topic, mark the missing pair explicitly in this index.
- Prefer lightweight normalization over rewriting historical technical content.
