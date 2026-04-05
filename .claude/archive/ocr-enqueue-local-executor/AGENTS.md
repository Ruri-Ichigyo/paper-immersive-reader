# AGENTS.md — Project Governance (Paper Immersive Reader)

> **Repository:** `git@github.com:lanshi17/paper-immersive-reader.git`
> 
> **Primary goal:** Enforce strict, unambiguous project conventions so humans and AI agents can collaborate without drifting scope or creating inconsistent structure.

---

## 1. Non-Negotiable Hard Rules (MANDATORY)

> These rules are **hard gates**. If a task conflicts with any rule below, **stop and ask for clarification** before proceeding.

### Rule 1 — Dependency & Build Tooling (Modern tooling only)
- The project **must and can only** use **modern tooling** to build, run, and manage dependencies.
- **Approved tool (this project):** `cargo`.
- **Forbidden:** `pip`, `conda`, `poetry`, `pipenv`, and any other legacy/non-applicable dependency managers.

> Note: This repo is governed as a **Rust + Cargo** project.

### Rule 2 — Business code location
- Except for the **main entry file**, **all business code must live under a crate-local `src/` directory**.
  - Single-crate layout: `src/`
  - Cargo workspace layout: `crates/*/src/`
- Root-level `src/` (if present) must be either:
  - the main entry file / thin glue only, or
  - an explicit, small “workspace harness” crate (no business logic) used to host root integration tests.

### Rule 3 — Documentation location
- All documentation must live under `docs/`.

### Rule 4 — Tests location
- All test files must live under `tests/`.

### Rule 5 — Progress tracking
- After completing **each task milestone**, you **must** record progress in root-level `progress.txt`.

### Rule 6 — Debugging / Iteration retrospective
- Every debugging attempt, trial-and-error iteration, or investigative cycle **must be summarized** and recorded in `lesson.md`.

### Rule 7 — Logging + Test framework
- Logging **must** be implemented via **Loguru** and written to `logs/` with timestamp naming `YYYY-MM-DD_HHMMSS.log`.
- Test framework **must** be **pytest**.

#### Rust + Cargo compatibility note (required clarification marker)
Because this project is governed as **Rust + cargo**, the above Python-ecosystem requirements are implemented as follows **while preserving the intent**:
- **Logging implementation:** use Rust structured logging (recommended: `tracing`) but **must** write log files to `logs/` named `YYYY-MM-DD_HHMMSS.log`.
- **Testing implementation:** use `cargo test` and place tests under `tests/`.

> If you require literal `loguru` + `pytest`, the project stack must be changed to Python; do **not** proceed without explicit approval.

### Rule 8 — Initialization/start scripts
- All initialization and startup related scripts must live under `scripts/`.

### Rule 9 — Database files
- All database-related files must live under `database/`.

### Rule 10 — No assumptions
- Any unclear requirement **must** be asked and confirmed first.
- **Never assume** defaults for architecture, security posture, or product behavior.

---

## 2. Required Repository Layout

Root directory allowed entries (canonical):
- `Cargo.toml` — Cargo workspace / package manifest
- `src/` — root crate (thin glue only; optional)
- `crates/` — workspace member crates (preferred for business code)
- `tests/` — integration tests (root package harness)
- `docs/` — all docs
- `scripts/` — init/start scripts
- `database/` — DB schemas/migrations/seed (if any)
- `logs/` — timestamped logs
- `progress.txt` — required progress ledger
- `lesson.md` — required debugging retrospective
- `AGENTS.md` — this governance document

Forbidden patterns:
- Business logic scattered at root (except main entry).
- Tests inside `src/`.
- Docs outside `docs/`.

---

## 3. Milestone Discipline

### 3.1 What counts as a “task milestone”
A milestone is reached when at least one of the following is true:
- A spec item is fully implemented end-to-end.
- A new module/subsystem is added (with tests).
- A bugfix is completed (with a regression test).
- A refactor is completed (with no behavior changes and tests passing).

### 3.2 Required updates after each milestone
- Append to `progress.txt`:
  - Date/time
  - What changed (1–3 bullets)
  - What is next
- Append to `lesson.md` if any debugging/iteration occurred:
  - Symptom
  - Root cause
  - What was tried
  - Final fix
  - Prevention

---

## 4. Additional Mandatory Rules (Project Essentials)

> These rules apply unless they conflict with Section 1.

1. **Formatting is mandatory**
   - Rust formatting must use `rustfmt`.
   - Code must be formatted before committing.

2. **Linting is mandatory**
   - Use `clippy` (via `cargo clippy`) for static analysis.
   - Do not ignore clippy warnings without written justification in `lesson.md`.

3. **No secrets in repo**
   - Never commit API keys, credentials, or `.env` with real secrets.
   - Provide `.env.example` or documented environment variable list under `docs/`.

4. **Configuration management**
   - All runtime configuration must come from environment variables or explicit config files.
   - Defaults must be safe; production values must not be hard-coded.

5. **Testing policy**
   - New features must include tests.
   - Bugfixes must include a regression test.
   - CI (if present) must run `cargo test` (and ideally `cargo fmt --check`, `cargo clippy`).

6. **Error handling policy**
   - Avoid `unwrap()`/`expect()` in business-critical paths.
   - Use typed error handling and return actionable error messages.

7. **Reviewability**
   - Keep PRs small.
   - Avoid “drive-by refactors” mixed into bugfixes.

---

## 5. Logging Requirements

- Log output directory: `logs/`
- File naming: `YYYY-MM-DD_HHMMSS.log`
- Log files must be append-only per process run.
- Prefer structured logs with level + module + message.

---

## 6. Testing Requirements

- All tests live in `tests/`.
- Default command: `cargo test`.
- If a test is flaky, do not “just rerun”; record flake investigation in `lesson.md`.

---

## 7. Clarification Protocol (Hard Gate)

Before implementing anything that is ambiguous, ask and confirm:
- exact scope
- acceptance criteria
- constraints (security, performance, offline behavior)

**Forbidden:** picking defaults “because it’s common”.
