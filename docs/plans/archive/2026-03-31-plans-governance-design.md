# Docs Plans Governance Design

**Status:** Approved
**Date:** 2026-03-31

## Goal
Define lightweight governance for `docs/plans/` so planning artifacts stay easy to scan, consistently named, and simple to maintain without rewriting older technical content.

## Architecture
Treat `docs/plans/` as a small governed directory with one shared index (`INDEX.md`) and paired topic documents using `*-design.md` and `*-implementation.md` naming. Keep normalization limited to top summary fields and index maintenance so historical plan bodies can remain intact.

## Scope
- Maintain a single index that explains naming, status, and maintenance rules.
- Expect non-index topics to use paired design and implementation documents.
- Normalize top-level summary fields for newly added or updated plan documents.

## Non-Goals
- Rewriting historical plan bodies into a strict template.
- Adding automation or enforcement outside the repository's markdown governance.
- Expanding `docs/plans/` into a broader documentation system.
