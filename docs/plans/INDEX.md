# Plans Index

## Purpose

This directory now tracks active planning artifacts only.

- `*-design.md` documents the intended design, scope, constraints, and decisions for an unfinished topic.
- `*-implementation.md` documents the execution plan for that unfinished topic.
- Completed topics move to [`archive/`](./archive/) and are listed in [`archive/INDEX.md`](./archive/INDEX.md).

## Naming Rules

Use:

- `YYYY-MM-DD-<topic>-design.md`
- `YYYY-MM-DD-<topic>-implementation.md`

Where `<topic>` is a short kebab-case identifier shared by both files.

## Status Rules

Recommended document-level status values:

- Design documents: `Draft`, `Approved`
- Implementation documents: `Planned`, `In Progress`, `Blocked`, `Done`

Archiving rule:

- Keep unfinished topics in `docs/plans/`.
- When a topic is complete, move the paired design and implementation files to `docs/plans/archive/`.
- Update both this index and `docs/plans/archive/INDEX.md` in the same change.

## Minimum Expectations

Keep these expectations lightweight. Prefer light normalization of the top summary fields for new or updated docs rather than retrofitting older technical content into a strict template.

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

## Current Active Topics

_No active topics. All planned work is complete._

## Archive Summary

- Archived completed topics: 14
- Archived plan documents: 28
- Archive manifest: [`archive/INDEX.md`](./archive/INDEX.md)

## Maintenance Rules

- When adding a new design doc, use the shared `<topic>` naming immediately.
- When adding an implementation doc, reuse the same `<topic>` and add or update the matching row here.
- If only one file exists for a topic, mark the missing pair explicitly in this index.
- When a topic is marked `Done`, move its paired files into `docs/plans/archive/` and refresh both indexes.
- Prefer lightweight normalization over rewriting historical technical content.
