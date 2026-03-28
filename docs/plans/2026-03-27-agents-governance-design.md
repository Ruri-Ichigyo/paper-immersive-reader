# AGENTS Governance Design

## Goal
Create a strict, self-consistent `AGENTS.md` governance document for the `paper-immersive-reader` repository so future human and AI contributors follow the same repository layout, tooling, logging, testing, clarification, and delivery discipline.

## Context
Current repository state is documentation-heavy and does not yet contain an implemented application scaffold. The user explicitly requested a Markdown `AGENTS.md` document with 10 mandatory rules, plus at least 5 additional project-essential rules. The user also chose:
- project style: **Rust + cargo**
- governance style: **A — strong enforcement**

A direct conflict existed in the requested hard rules because rule 7 required Python-specific `loguru` and `pytest`, while the selected stack is Rust + cargo. To avoid hidden assumptions, the design resolves this by preserving the original rule text and adding an explicit Rust compatibility note that defines the required equivalent implementation behavior.

## Design Decision
Use a **strong-constraint governance document** with three layers:

1. **Hard rules**
   - non-negotiable repository governance
   - stop-work behavior on ambiguity or rule conflicts
   - explicit stack/tooling policy (`cargo` only)

2. **Canonical layout + process discipline**
   - required repository directories
   - milestone definitions
   - mandatory updates to `progress.txt` and `lesson.md`

3. **Project-essential engineering rules**
   - formatting, linting, config, secrets, testing, error handling, reviewability

## Why this design
This repository is still early enough that governance can be installed before code sprawl begins. A permissive contributor guide would leave too much room for drift. A strict governance document is a better fit because the user explicitly asked for hard rules and anti-assumption behavior.

## Key Design Sections

### 1. Repository identity
- repository URL
- goal of the governance doc
- statement that the project is governed as Rust + cargo

### 2. Non-negotiable rules
Must include all user-required rules, rewritten clearly and sequentially:
- modern tooling only
- code under `src/`
- docs under `docs/`
- tests under `tests/`
- progress in `progress.txt`
- debugging retrospective in `lesson.md`
- logging/testing rule with Rust compatibility note
- scripts under `scripts/`
- database files under `database/`
- ask before assuming anything unclear

### 3. Canonical repository layout
Document allowed root-level entries and forbidden anti-patterns so future contributors do not scatter files arbitrarily.

### 4. Milestone discipline
Define what counts as a milestone and exactly what must be recorded after each milestone.

### 5. Additional essential rules
At least 5 must be added. This design uses 7:
- formatting via `rustfmt`
- linting via `cargo clippy`
- no secrets in repo
- explicit configuration management
- mandatory tests for new features and bugfixes
- error handling policy
- reviewability / small PR discipline

### 6. Logging requirements
Even though the stack is Rust, the user-required logging intent is preserved by requiring timestamped files in `logs/` and recommending structured logging.

### 7. Testing requirements
Preserve the user’s intent for mandatory testing while adapting execution to `cargo test` for Rust.

### 8. Clarification protocol
A dedicated section reinforcing that ambiguous requirements must be clarified before implementation.

## Trade-offs

### Option rejected: rewrite rule 7 fully into Rust terms
Rejected because the user explicitly required the hard rules to be written into the document. Rewriting the original text entirely would lose fidelity.

### Option chosen: preserve original wording + compatibility note
Chosen because it keeps the user’s requested rule visible while making the document operational for a Rust project.

## Validation Criteria
The design is successful if the resulting `AGENTS.md`:
- contains all 10 required user rules
- includes at least 5 additional rules
- is internally consistent for a Rust + cargo project
- clearly resolves the pytest/loguru conflict without hiding it
- gives future contributors concrete directory and process guidance

## Output Files
- `AGENTS.md`
- `docs/plans/2026-03-27-agents-governance-design.md`
- `docs/plans/2026-03-27-agents-governance-implementation.md`
