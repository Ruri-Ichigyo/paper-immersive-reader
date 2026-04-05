# AGENTS Governance Implementation Plan

**Status:** Done
**Date:** 2026-03-27

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create and validate a strict `AGENTS.md` governance document for the repository, aligned to Rust + cargo and the user’s mandatory project rules.

**Architecture:** The work is documentation-first. First inspect current repository context, then resolve requirement conflicts explicitly, then write the governance document, and finally verify the resulting files and placement. The implementation is intentionally narrow: produce project-governance artifacts only, with no application code scaffolding.

**Tech Stack:** Markdown documentation, Rust governance conventions, Cargo-oriented process rules

---

### Task 1: Inspect repository context

**Files:**
- Read: `AGENTS.md` (if present)
- Read: `docs/`
- Read: `PRD.md`
- Read: `TECH_STACK.md`
- Read: `IMPLEMENTATION_PLAN.md`

**Step 1: Read current top-level repository state**
Run: `ls -alh`
Expected: root files and docs directory are visible.

**Step 2: Check existing docs structure**
Run: `ls -alh docs`
Expected: existing docs subdirectories are visible.

**Step 3: Identify stack/governance signals**
Check for: `Cargo.toml`, `pyproject.toml`, `package.json`, `CMakeLists.txt`.
Expected: confirm there is not yet an implemented runtime stack in repo files, so the governance choice must be driven by the user’s explicit instruction.

**Step 4: Record constraints**
Document these facts before writing:
- user selected `AGENTS.md`
- user selected Rust + cargo
- user selected strong enforcement style
- user required pytest/loguru text, creating a cross-stack conflict that must be resolved explicitly in the document

### Task 2: Resolve governance conflict in writing strategy

**Files:**
- Modify: `docs/plans/2026-03-27-agents-governance-design.md`
- Modify: `AGENTS.md`

**Step 1: Write the design resolution**
Add a design note that preserves the original required rule wording while adding a Rust compatibility note.

**Step 2: Ensure no hidden assumptions remain**
Verify that the document states the project is governed as Rust + cargo and explains how testing/logging are implemented in that ecosystem.

**Step 3: Keep the scope narrow**
Do not add application architecture or runtime implementation details beyond what governance requires.

### Task 3: Write `AGENTS.md`

**Files:**
- Create: `AGENTS.md`

**Step 1: Write the document header**
Include repository URL and governance purpose.

**Step 2: Write all 10 mandatory rules**
Make each rule explicit and separately titled.

**Step 3: Add the Rust compatibility note for rule 7**
State how logging and testing are implemented in Rust while preserving the original required rule text.

**Step 4: Add additional mandatory rules**
Include at least the following:
- formatting with `rustfmt`
- linting with `cargo clippy`
- no secrets in repo
- configuration management rules
- testing policy
- error handling policy
- reviewability rules

**Step 5: Add repository layout and milestone discipline sections**
Document canonical root entries, forbidden patterns, milestone definitions, and required updates to `progress.txt` / `lesson.md`.

### Task 4: Write the design document

**Files:**
- Create: `docs/plans/2026-03-27-agents-governance-design.md`

**Step 1: Capture the goal and context**
Explain why this governance document exists and what repository state it assumes.

**Step 2: Capture design decisions**
Describe the selected strong-enforcement model and the conflict-resolution approach for Python-specific rules.

**Step 3: Define validation criteria**
List what makes the design successful.

### Task 5: Write the implementation plan document

**Files:**
- Create: `docs/plans/2026-03-27-agents-governance-implementation.md`

**Step 1: Write the standard plan header**
Include goal, architecture, tech stack, and the required executing-plans note.

**Step 2: Break work into bite-sized tasks**
Cover repository inspection, conflict handling, AGENTS drafting, and verification.

**Step 3: Keep the plan exact**
Use exact file paths and explicit validation actions.

### Task 6: Verify outputs

**Files:**
- Read: `AGENTS.md`
- Read: `docs/plans/2026-03-27-agents-governance-design.md`
- Read: `docs/plans/2026-03-27-agents-governance-implementation.md`

**Step 1: Verify file placement**
Run: `ls -alh` and `ls -alh docs/plans`
Expected: all three files exist in the intended locations.

**Step 2: Verify document completeness**
Read `AGENTS.md` and confirm:
- 10 mandatory rules included
- at least 5 additional rules included
- Rust + cargo alignment present
- compatibility note for pytest/loguru present

**Step 3: Verify no scope drift**
Ensure no application code was created and no unrelated files were changed.

### Task 7: Handoff

**Files:**
- None

**Step 1: Report artifacts created**
List the three documentation artifacts and their paths.

**Step 2: Offer execution choice**
If further work is needed, offer:
1. Subagent-Driven (this session)
2. Parallel Session (separate)
