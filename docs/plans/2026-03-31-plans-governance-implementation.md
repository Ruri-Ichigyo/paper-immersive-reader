# Docs Plans Governance Implementation Plan

**Status:** Planned
**Date:** 2026-03-31

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add lightweight governance for `docs/plans/` by introducing an index file, normalizing document headers, and keeping the directory easy to maintain without rewriting historical technical content.

**Architecture:** Treat `docs/plans/` as a governed planning artifact directory with two document types: `*-design.md` and `*-implementation.md`. Keep historical content in place, add a single `INDEX.md` as the governance entrypoint, and apply only lightweight normalization to existing files so future additions stay consistent.

**Tech Stack:** Markdown documentation, repository conventions in `docs/plans/`, manual review via file reads

---

### Task 1: Add plans governance index

**Files:**
- Create: `docs/plans/INDEX.md`
- Test: `docs/plans/INDEX.md`

**Step 1: Write the missing governance artifact**

Create the index file with sections for:

```md
# Plans Index

## Purpose
- explain `*-design.md` vs `*-implementation.md`

## Naming Rules
- YYYY-MM-DD-topic-design.md
- YYYY-MM-DD-topic-implementation.md

## Status Rules
- design: Draft / Approved
- implementation: Planned / In Progress / Done

## Minimum Expectations
- required frontmatter-style summary fields at top of each document

## Current Topics
- a table listing topic, date, design, implementation, status, notes

## Maintenance Rules
- update the index when adding a new topic or pair
```

**Step 2: Read the file to verify it exists and is complete**

Run:
```bash
ls docs/plans
```
Expected: `INDEX.md` is present in the directory listing.

Then read `docs/plans/INDEX.md` and confirm all required sections are present.

**Step 3: Commit**

```bash
git add docs/plans/INDEX.md
git commit -m "Add docs plans governance index"
```

### Task 2: Normalize design document headers

**Files:**
- Modify: `docs/plans/2026-03-27-agents-governance-design.md`
- Modify: `docs/plans/2026-03-27-rust-workspace-scaffold-design.md`
- Modify: `docs/plans/2026-03-28-import-pipeline-skeleton-design.md`
- Modify: `docs/plans/2026-03-28-reader-domain-models-design.md`
- Test: `docs/plans/*.md`

**Step 1: Identify design docs missing status/date normalization**

Read the existing design docs and compare their openings.

Expected findings before edits:
- some design docs start directly with `## Goal`
- some already include `**Status:**` and `**Date:**`

**Step 2: Apply minimal header normalization**

For each design doc that is missing them, add:

```md
**Status:** Approved
**Date:** YYYY-MM-DD
```

immediately below the H1 title.

Do not rewrite the body. Do not change technical decisions.

**Step 3: Read the edited files to verify consistency**

Read the first 10-20 lines of each edited design doc.

Expected: each now starts with:

```md
# ...

**Status:** Approved
**Date:** ...
```

followed by the existing content such as `## Goal` or `**Project:**`.

**Step 4: Commit**

```bash
git add \
  docs/plans/2026-03-27-agents-governance-design.md \
  docs/plans/2026-03-27-rust-workspace-scaffold-design.md \
  docs/plans/2026-03-28-import-pipeline-skeleton-design.md \
  docs/plans/2026-03-28-reader-domain-models-design.md
git commit -m "Normalize plan design headers"
```

### Task 3: Normalize implementation document headers

**Files:**
- Modify: `docs/plans/2026-03-27-agents-governance-implementation.md`
- Modify: `docs/plans/2026-03-27-rust-workspace-scaffold-implementation.md`
- Modify: `docs/plans/2026-03-28-import-pipeline-skeleton-implementation.md`
- Modify: `docs/plans/2026-03-28-reader-domain-models-implementation.md`
- Modify: `docs/plans/2026-03-28-pdf-extraction-skeleton-implementation.md`
- Test: `docs/plans/*.md`

**Step 1: Identify implementation docs missing status/date normalization**

Read the existing implementation docs and compare their openings.

Expected findings before edits:
- some implementation docs start directly with the required Claude execution note
- some are missing `**Status:** Planned`
- some are missing `**Date:** YYYY-MM-DD`

**Step 2: Apply minimal header normalization**

For each implementation doc that is missing them, add:

```md
**Status:** Planned
**Date:** YYYY-MM-DD
```

between the H1 title and the required execution note.

Do not rewrite task bodies.

**Step 3: Read the edited files to verify consistency**

Read the first 12-20 lines of each edited implementation doc.

Expected: each now starts with:

```md
# ... Implementation Plan

**Status:** Planned
**Date:** ...

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans ...
```

and still retains `Goal`, `Architecture`, and `Tech Stack` sections.

**Step 4: Commit**

```bash
git add \
  docs/plans/2026-03-27-agents-governance-implementation.md \
  docs/plans/2026-03-27-rust-workspace-scaffold-implementation.md \
  docs/plans/2026-03-28-import-pipeline-skeleton-implementation.md \
  docs/plans/2026-03-28-reader-domain-models-implementation.md \
  docs/plans/2026-03-28-pdf-extraction-skeleton-implementation.md
git commit -m "Normalize plan implementation headers"
```

### Task 4: Reconcile index entries with actual files

**Files:**
- Modify: `docs/plans/INDEX.md`
- Test: `docs/plans/INDEX.md`

**Step 1: Compare index topics to directory contents**

Check the `docs/plans/` directory listing against the Current Topics table.

Expected topics:
- `agents-governance`
- `rust-workspace-scaffold`
- `import-pipeline-skeleton`
- `reader-domain-models`
- `pdf-extraction-skeleton`

Each should have both a design and implementation file.

**Step 2: Update the table if any row is wrong**

If needed, update the table to reflect the actual files present and the intended status wording.

Keep notes short, for example:

```md
| pdf-extraction-skeleton | 2026-03-28 | yes | yes | planned | PDF extraction milestone planning pair |
```

**Step 3: Read the index to verify the table is accurate**

Read `docs/plans/INDEX.md`.

Expected: all listed topics correspond to real files in the directory and no current topic is missing.

**Step 4: Commit**

```bash
git add docs/plans/INDEX.md
git commit -m "Reconcile plans index with directory contents"
```

### Task 5: Final verification sweep

**Files:**
- Modify: none unless a mismatch is found
- Test: `docs/plans/*.md`

**Step 1: Verify all plan filenames follow the naming rule**

Check that every markdown file in `docs/plans/` matches one of:

```text
YYYY-MM-DD-<topic>-design.md
YYYY-MM-DD-<topic>-implementation.md
INDEX.md
```

Expected: PASS.

**Step 2: Verify each topic is paired**

Check that every non-index topic has both a design and an implementation document.

Expected: PASS.

**Step 3: Verify normalized openings**

Sample-read the top of all plan documents.

Expected:
- design docs have title + status + date
- implementation docs have title + status + date + required execution note
- index has governance sections and current topics table

**Step 4: Commit only if fixes were needed**

If the verification sweep finds a mismatch and you correct it, create a small follow-up commit. Otherwise stop.

```bash
git add docs/plans
git commit -m "Fix plans governance verification issues"
```

### Task 6: Stop unless the user asks to execute the plan

**Files:**
- Modify: none

**Step 1: Do not implement additional documentation rewrites**

Do not rewrite all historical plan bodies into a new template.

**Step 2: Offer execution choice**

After saving this plan, tell the user:

Plan complete and saved to `docs/plans/2026-03-31-plans-governance-implementation.md`. Two execution options:

1. Subagent-Driven (this session) - I dispatch fresh subagent per task, review between tasks, fast iteration
2. Parallel Session (separate) - Open new session with executing-plans, batch execution with checkpoints

Which approach?
