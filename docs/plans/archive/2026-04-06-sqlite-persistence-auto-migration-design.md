# SQLite Persistence Auto-Migration Design

**Status:** Approved
**Date:** 2026-04-06

## Goal
Replace the current JSON snapshot persistence with SQLite at `database/app-state.db`, while keeping the existing CLI workflow unchanged and automatically migrating a legacy `database/app-state.json` on first startup.

## Architecture
Keep `crates/core` domain-focused, keep `crates/importing` responsible for extraction and segmentation, and move all runtime persistence responsibility in `crates/application` to a new SQLite-backed store. The CLI remains a thin entrypoint and continues to expose only `ocr run-next` and `ocr run <job-id>`, but it loads SQLite by default instead of JSON.

SQLite becomes the only runtime source of truth for persisted reader state. The legacy JSON snapshot is supported only as a one-time migration input and is renamed to `database/app-state.migrated.json` after a successful import into SQLite.

## Scope
- Add a SQLite-backed application store under `crates/application`.
- Change the default persistence file from `database/app-state.json` to `database/app-state.db`.
- Automatically migrate a legacy `database/app-state.json` into SQLite on first open when the `.db` file does not yet exist.
- Rename the successfully migrated JSON file to `database/app-state.migrated.json`.
- Preserve the current managed PDF asset layout under `database/assets/`.
- Keep existing CLI commands, exit-code behavior, and human-readable output unchanged.
- Rework orchestration code so runtime persistence no longer depends on `JsonAppStateStore`.

## Non-Goals
- No runtime dual-backend support where JSON and SQLite are both active stores.
- No change to the CLI command surface.
- No background polling or daemon worker in this milestone.
- No gateway HTTP API in this milestone.
- No remote OCR service integration.
- No sync or Zotero work.
- No schema versioning or multi-step migration framework beyond the initial JSON-to-SQLite bootstrap.

## Storage Paths
Primary runtime database:

- `database/app-state.db`

Legacy migration input:

- `database/app-state.json`

Migration backup output:

- `database/app-state.migrated.json`

Managed OCR assets remain at:

- `database/assets/<paper-id>.pdf`

## Runtime Open and Migration Flow
### Normal startup with SQLite present
1. If `database/app-state.db` exists, open SQLite directly.
2. Do not inspect or consume `database/app-state.json` for runtime state.
3. If a legacy JSON file still exists beside the database, ignore it and continue using SQLite.

### Fresh startup with no prior state
1. If neither `database/app-state.db` nor `database/app-state.json` exists, create the parent directory if needed.
2. Initialize an empty SQLite database and create the required schema.
3. Return the SQLite store as the runtime persistence owner.

### First startup with legacy JSON only
1. If `database/app-state.db` does not exist but `database/app-state.json` does exist, read the JSON snapshot.
2. Create the SQLite database and initialize the schema.
3. In a single transaction, import all persisted entities from the JSON snapshot into SQLite.
4. Commit the transaction only if all inserts succeed.
5. Rename `database/app-state.json` to `database/app-state.migrated.json` only after the transaction commits successfully.
6. Return the SQLite store as the runtime persistence owner.

### Migration failure behavior
- If JSON parsing fails, startup fails and the JSON file is not renamed.
- If schema creation or any insert fails, the SQLite transaction is rolled back and the JSON file is not renamed.
- The process must not silently fall back to JSON as a live runtime store after a failed migration.

## Data Model
The initial SQLite schema should stay close to the current domain model to minimize migration risk and keep test behavior stable.

### `papers`
- `id` TEXT PRIMARY KEY
- `title` TEXT NULL
- `source_type` TEXT NOT NULL
- `source_ref` TEXT NULL
- `status` TEXT NOT NULL
- `parse_version` INTEGER NOT NULL

### `blocks`
- `id` TEXT PRIMARY KEY
- `paper_id` TEXT NOT NULL REFERENCES `papers(id)` ON DELETE CASCADE
- `ordinal` INTEGER NOT NULL
- `text` TEXT NOT NULL
- `block_type` TEXT NOT NULL

### `reading_progress`
- `paper_id` TEXT PRIMARY KEY REFERENCES `papers(id)` ON DELETE CASCADE
- `current_block_id` TEXT NULL
- `percent` REAL NOT NULL
- `updated_at_ms` INTEGER NOT NULL

### `ocr_jobs`
- `id` TEXT PRIMARY KEY
- `paper_id` TEXT NOT NULL REFERENCES `papers(id)` ON DELETE CASCADE
- `engine` TEXT NOT NULL
- `status` TEXT NOT NULL
- `requested_at_ms` INTEGER NOT NULL
- `started_at_ms` INTEGER NULL
- `finished_at_ms` INTEGER NULL
- `error_message` TEXT NULL

### `ocr_job_results`
- `paper_id` TEXT PRIMARY KEY REFERENCES `papers(id)` ON DELETE CASCADE
- `text` TEXT NOT NULL

## Query and Index Strategy
The initial schema only needs minimal supporting indexes:

- `CREATE INDEX idx_blocks_paper_ordinal ON blocks(paper_id, ordinal);`
- `CREATE INDEX idx_ocr_jobs_status_requested ON ocr_jobs(status, requested_at_ms);`
- `CREATE INDEX idx_ocr_jobs_paper_id ON ocr_jobs(paper_id);`
- `CREATE INDEX idx_papers_status ON papers(status);`

This is enough for the current CLI and prepares for a later worker that claims queued jobs.

## Consistency Rules
1. `papers.id` is the aggregate anchor for blocks, progress, OCR jobs, and OCR results.
2. OCR success writes must occur inside one transaction:
- update the job to `Completed`
- upsert the OCR result
- replace the paper's existing blocks
- replace the paper's reading progress
- update `papers.status` to `ReadyText`
3. OCR failure writes must occur inside one transaction:
- update the job to `Failed`
- keep `papers.status` at `ReadyOcr`
- do not write partial blocks or reading progress
4. `ImportOrchestrator` must persist `Paper + OcrJob` together for `OcrRequired`.
5. Re-segmentation for an existing paper must replace older blocks and progress rather than append duplicates.
6. `run_next_job()` must claim queued work transactionally so a future background worker milestone can build on the same semantics.

## Cross-Crate Responsibilities
### `crates/core`
- Keep domain types and lifecycle validation only.
- Continue to own OCR state-transition rules.

### `crates/importing`
- Continue to own PDF extraction and text segmentation.
- Continue to expose reusable text segmentation for an existing paper.

### `crates/application`
- Own the SQLite schema, open-or-migrate behavior, and all persistence operations.
- Own JSON snapshot loading only as a migration input.
- Own transactional writes for import success, OCR enqueue, and OCR completion.

### `crates/cli`
- Keep command parsing and user-facing messages only.
- Load the SQLite-backed store through the application persistence entrypoint.
- Do not embed migration logic or SQL details.

## Error Handling
- Database open, schema creation, or migration failures should surface as startup errors from the application persistence layer.
- CLI output should remain short plain text and preserve current non-zero exit behavior on failure.
- The migration path should be explicit about the failing path in its error message so operators can see whether the failure came from `.db` creation, JSON parsing, insert failure, or rename failure.

## Testing Strategy
Required coverage for this milestone:

1. SQLite store initialization
- empty database creation
- schema creation on first open

2. JSON-to-SQLite migration
- migrates all persisted entity sets
- renames `database/app-state.json` to `database/app-state.migrated.json`
- leaves the source JSON in place on failure

3. Orchestration regression coverage
- embedded-text import persistence still works
- `OcrRequired` still queues a persisted OCR job for a real paper
- OCR success still writes text artifacts and marks the paper readable
- OCR failure still leaves the paper in `ReadyOcr`

4. CLI compatibility coverage
- `ocr run-next` still succeeds against the default runtime path
- `ocr run <job-id>` still succeeds against the default runtime path
- first CLI run can trigger JSON-to-SQLite migration when only legacy JSON exists

5. Edge coverage
- existing `.db` causes legacy `.json` to be ignored
- invalid JSON blocks migration and preserves the original file
- managed-asset-missing failure behavior remains unchanged

## Acceptance Criteria
- `database/app-state.db` is the runtime source of truth.
- Existing CLI behavior remains unchanged from the user's perspective.
- First startup can automatically migrate `database/app-state.json` into SQLite.
- Successful migration renames the source file to `database/app-state.migrated.json`.
- Runtime persistence no longer depends on `JsonAppStateStore`.
- OCR/import workflows continue to pass full workspace verification after the migration.
