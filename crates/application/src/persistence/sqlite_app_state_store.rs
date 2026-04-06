use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use super::json_app_state_store::AppState;
use paper_immersive_reader_core::domain::{
    Block, BlockId, OcrEngine, OcrJob, OcrJobId, OcrJobResult, OcrJobStatus, Paper, PaperId,
    PaperSourceType, PaperStatus, ReadingProgress,
};
use paper_immersive_reader_importing::{ImportResult, TextImportArtifacts};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};

pub const DEFAULT_APP_DB_PATH: &str = "database/app-state.db";

pub struct SqliteAppStateStore {
    path: PathBuf,
    connection: Connection,
}

impl SqliteAppStateStore {
    pub fn open_or_migrate(path: impl AsRef<Path>) -> Result<Self, SqliteAppStateStoreError> {
        let path = path.as_ref().to_path_buf();

        if path.exists() {
            return Self::open(&path);
        }

        let json_path = sibling_json_path(&path);
        if !json_path.exists() {
            return Self::open(&path);
        }

        let legacy_snapshot =
            super::json_app_state_store::JsonAppStateStore::load_migration_input(&json_path)?
                .snapshot()
                .clone();

        let mut store = Self::open(&path)?;
        store.import_snapshot(legacy_snapshot)?;
        std::fs::rename(&json_path, migrated_json_path(&path))?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, SqliteAppStateStoreError> {
        let path = path.as_ref().to_path_buf();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(&path)?;
        connection.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS papers (
                id TEXT PRIMARY KEY,
                title TEXT NULL,
                source_type TEXT NOT NULL,
                source_ref TEXT NULL,
                status TEXT NOT NULL,
                parse_version INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS blocks (
                id TEXT PRIMARY KEY,
                paper_id TEXT NOT NULL REFERENCES papers(id) ON DELETE CASCADE,
                order_index INTEGER NOT NULL,
                text TEXT NOT NULL,
                block_hash TEXT NULL
            );

            CREATE TABLE IF NOT EXISTS reading_progress (
                paper_id TEXT PRIMARY KEY REFERENCES papers(id) ON DELETE CASCADE,
                current_block_index INTEGER NOT NULL,
                current_block_id TEXT NULL,
                updated_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ocr_jobs (
                id TEXT PRIMARY KEY,
                paper_id TEXT NOT NULL REFERENCES papers(id) ON DELETE CASCADE,
                engine TEXT NOT NULL,
                status TEXT NOT NULL,
                requested_at_ms INTEGER NOT NULL,
                started_at_ms INTEGER NULL,
                finished_at_ms INTEGER NULL,
                error_message TEXT NULL
            );

            CREATE TABLE IF NOT EXISTS ocr_job_results (
                paper_id TEXT PRIMARY KEY REFERENCES papers(id) ON DELETE CASCADE,
                text TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_blocks_paper_order_index
                ON blocks(paper_id, order_index);
            CREATE INDEX IF NOT EXISTS idx_ocr_jobs_status_requested
                ON ocr_jobs(status, requested_at_ms);
            CREATE INDEX IF NOT EXISTS idx_ocr_jobs_paper_id
                ON ocr_jobs(paper_id);
            CREATE INDEX IF NOT EXISTS idx_papers_status
                ON papers(status);
            ",
        )?;

        Ok(Self { path, connection })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn managed_assets_dir(&self) -> PathBuf {
        self.path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("assets")
    }

    pub fn managed_pdf_asset_path(&self, paper_id: &PaperId) -> PathBuf {
        self.managed_assets_dir()
            .join(format!("{}.pdf", paper_id.0))
    }

    pub fn persist_managed_pdf_asset(
        &self,
        paper_id: &PaperId,
        bytes: &[u8],
    ) -> Result<PathBuf, std::io::Error> {
        let asset_path = self.managed_pdf_asset_path(paper_id);
        if let Some(parent) = asset_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&asset_path, bytes)?;
        Ok(asset_path)
    }

    pub fn snapshot(&self) -> Result<AppState, SqliteAppStateStoreError> {
        Ok(AppState {
            papers: self.load_papers()?,
            blocks: self.load_blocks()?,
            reading_progress: self.load_reading_progress()?,
            ocr_jobs: self.load_ocr_jobs()?,
            ocr_job_results: self.load_ocr_job_results()?,
        })
    }

    pub fn save_import_result(
        &mut self,
        result: ImportResult,
    ) -> Result<(), SqliteAppStateStoreError> {
        self.with_transaction(|transaction| {
            upsert_paper(transaction, &result.paper)?;
            replace_blocks(transaction, &result.paper.id, &result.blocks)?;
            replace_reading_progress(transaction, &result.reading_progress)?;
            Ok(())
        })
    }

    pub fn save_paper_and_job(
        &mut self,
        paper: Paper,
        job: OcrJob,
    ) -> Result<(), SqliteAppStateStoreError> {
        self.with_transaction(|transaction| {
            upsert_paper(transaction, &paper)?;
            upsert_job(transaction, &job)?;
            Ok(())
        })
    }

    pub fn get_job(&self, job_id: &OcrJobId) -> Option<OcrJob> {
        self.connection
            .query_row(
                "
                SELECT id, paper_id, engine, status, requested_at_ms, started_at_ms, finished_at_ms, error_message
                FROM ocr_jobs
                WHERE id = ?1
                ",
                [job_id.0.as_str()],
                map_ocr_job_row,
            )
            .optional()
            .ok()
            .flatten()
    }

    pub fn claim_job(
        &mut self,
        job_id: &OcrJobId,
        started_at_ms: u64,
    ) -> Result<Option<OcrJob>, SqliteAppStateStoreError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let queued_job = transaction
            .query_row(
                "
                SELECT id, paper_id, engine, status, requested_at_ms, started_at_ms, finished_at_ms, error_message
                FROM ocr_jobs
                WHERE id = ?1 AND status = 'Queued'
                ",
                [job_id.0.as_str()],
                map_ocr_job_row,
            )
            .optional()?;

        let Some(queued_job) = queued_job else {
            transaction.commit()?;
            return Ok(None);
        };

        let processing_job = queued_job.mark_processing(started_at_ms).ok();
        let Some(processing_job) = processing_job else {
            transaction.commit()?;
            return Ok(None);
        };

        upsert_job(&transaction, &processing_job)?;
        transaction.commit()?;
        Ok(Some(processing_job))
    }

    pub fn first_queued_job(&self) -> Option<OcrJob> {
        self.connection
            .query_row(
                "
                SELECT id, paper_id, engine, status, requested_at_ms, started_at_ms, finished_at_ms, error_message
                FROM ocr_jobs
                WHERE status = 'Queued'
                ORDER BY requested_at_ms ASC, id ASC
                LIMIT 1
                ",
                [],
                map_ocr_job_row,
            )
            .optional()
            .ok()
            .flatten()
    }

    pub fn claim_next_queued_job(
        &mut self,
        started_at_ms: u64,
    ) -> Result<Option<OcrJob>, SqliteAppStateStoreError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let queued_job = transaction
            .query_row(
                "
                SELECT id, paper_id, engine, status, requested_at_ms, started_at_ms, finished_at_ms, error_message
                FROM ocr_jobs
                WHERE status = 'Queued'
                ORDER BY requested_at_ms ASC, id ASC
                LIMIT 1
                ",
                [],
                map_ocr_job_row,
            )
            .optional()?;

        let Some(queued_job) = queued_job else {
            transaction.commit()?;
            return Ok(None);
        };

        let processing_job = queued_job.mark_processing(started_at_ms).ok();
        let Some(processing_job) = processing_job else {
            transaction.commit()?;
            return Ok(None);
        };

        upsert_job(&transaction, &processing_job)?;
        transaction.commit()?;
        Ok(Some(processing_job))
    }

    pub fn get_paper(&self, paper_id: &PaperId) -> Option<Paper> {
        self.connection
            .query_row(
                "
                SELECT id, title, source_type, source_ref, status, parse_version
                FROM papers
                WHERE id = ?1
                ",
                [paper_id.0.as_str()],
                map_paper_row,
            )
            .optional()
            .ok()
            .flatten()
    }

    pub fn save_job(&mut self, job: OcrJob) -> Result<(), SqliteAppStateStoreError> {
        self.with_transaction(|transaction| upsert_job(transaction, &job))
    }

    pub fn save_completed_ocr(
        &mut self,
        completed_job: OcrJob,
        ocr_result: OcrJobResult,
        artifacts: TextImportArtifacts,
        updated_paper: Paper,
    ) -> Result<(), SqliteAppStateStoreError> {
        self.with_transaction(|transaction| {
            upsert_paper(transaction, &updated_paper)?;
            upsert_job(transaction, &completed_job)?;
            upsert_ocr_job_result(transaction, &ocr_result)?;
            replace_blocks(transaction, &updated_paper.id, &artifacts.blocks)?;
            replace_reading_progress(transaction, &artifacts.reading_progress)?;
            Ok(())
        })
    }

    fn with_transaction<T>(
        &mut self,
        operation: impl FnOnce(&Transaction<'_>) -> Result<T, SqliteAppStateStoreError>,
    ) -> Result<T, SqliteAppStateStoreError> {
        let transaction = self.connection.transaction()?;
        let result = operation(&transaction)?;
        transaction.commit()?;
        Ok(result)
    }

    fn import_snapshot(&mut self, snapshot: AppState) -> Result<(), SqliteAppStateStoreError> {
        self.with_transaction(|transaction| {
            for paper in &snapshot.papers {
                upsert_paper(transaction, paper)?;
            }
            for block in &snapshot.blocks {
                upsert_block(transaction, block)?;
            }
            for progress in &snapshot.reading_progress {
                upsert_reading_progress(transaction, progress)?;
            }
            for job in &snapshot.ocr_jobs {
                upsert_job(transaction, job)?;
            }
            for result in &snapshot.ocr_job_results {
                upsert_ocr_job_result(transaction, result)?;
            }
            Ok(())
        })
    }

    fn load_papers(&self) -> Result<Vec<Paper>, SqliteAppStateStoreError> {
        let mut statement = self.connection.prepare(
            "
            SELECT id, title, source_type, source_ref, status, parse_version
            FROM papers
            ORDER BY id ASC
            ",
        )?;
        let papers = statement
            .query_map([], map_paper_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(papers)
    }

    fn load_blocks(&self) -> Result<Vec<Block>, SqliteAppStateStoreError> {
        let mut statement = self.connection.prepare(
            "
            SELECT id, paper_id, order_index, text, block_hash
            FROM blocks
            ORDER BY paper_id ASC, order_index ASC, id ASC
            ",
        )?;
        let blocks = statement
            .query_map([], map_block_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(blocks)
    }

    fn load_reading_progress(&self) -> Result<Vec<ReadingProgress>, SqliteAppStateStoreError> {
        let mut statement = self.connection.prepare(
            "
            SELECT paper_id, current_block_index, current_block_id, updated_at_ms
            FROM reading_progress
            ORDER BY paper_id ASC
            ",
        )?;
        let progress = statement
            .query_map([], map_reading_progress_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(progress)
    }

    fn load_ocr_jobs(&self) -> Result<Vec<OcrJob>, SqliteAppStateStoreError> {
        let mut statement = self.connection.prepare(
            "
            SELECT id, paper_id, engine, status, requested_at_ms, started_at_ms, finished_at_ms, error_message
            FROM ocr_jobs
            ORDER BY requested_at_ms ASC, id ASC
            ",
        )?;
        let jobs = statement
            .query_map([], map_ocr_job_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(jobs)
    }

    fn load_ocr_job_results(&self) -> Result<Vec<OcrJobResult>, SqliteAppStateStoreError> {
        let mut statement = self.connection.prepare(
            "
            SELECT paper_id, text
            FROM ocr_job_results
            ORDER BY paper_id ASC
            ",
        )?;
        let results = statement
            .query_map([], map_ocr_job_result_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(results)
    }
}

#[derive(Debug)]
pub enum SqliteAppStateStoreError {
    Io(std::io::Error),
    Sqlite(rusqlite::Error),
    ValueOutOfRange(&'static str),
}

impl Display for SqliteAppStateStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Sqlite(error) => write!(f, "SQLite error: {error}"),
            Self::ValueOutOfRange(field) => {
                write!(f, "value out of range for SQLite integer field `{field}`")
            }
        }
    }
}

impl std::error::Error for SqliteAppStateStoreError {}

impl From<std::io::Error> for SqliteAppStateStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for SqliteAppStateStoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

fn sibling_json_path(db_path: &Path) -> PathBuf {
    db_path.with_extension("json")
}

fn migrated_json_path(db_path: &Path) -> PathBuf {
    db_path.with_extension("migrated.json")
}

fn upsert_paper(
    transaction: &Transaction<'_>,
    paper: &Paper,
) -> Result<(), SqliteAppStateStoreError> {
    transaction.execute(
        "
        INSERT INTO papers (id, title, source_type, source_ref, status, parse_version)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            source_type = excluded.source_type,
            source_ref = excluded.source_ref,
            status = excluded.status,
            parse_version = excluded.parse_version
        ",
        params![
            paper.id.0.as_str(),
            paper.title.as_deref(),
            paper_source_type_as_str(&paper.source_type),
            paper.source_ref.as_deref(),
            paper_status_as_str(&paper.status),
            i64::from(paper.parse_version),
        ],
    )?;
    Ok(())
}

fn upsert_block(
    transaction: &Transaction<'_>,
    block: &Block,
) -> Result<(), SqliteAppStateStoreError> {
    transaction.execute(
        "
        INSERT INTO blocks (id, paper_id, order_index, text, block_hash)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(id) DO UPDATE SET
            paper_id = excluded.paper_id,
            order_index = excluded.order_index,
            text = excluded.text,
            block_hash = excluded.block_hash
        ",
        params![
            block.id.0.as_str(),
            block.paper_id.0.as_str(),
            i64::from(block.order_index),
            block.text.as_str(),
            block.block_hash.as_deref(),
        ],
    )?;
    Ok(())
}

fn replace_blocks(
    transaction: &Transaction<'_>,
    paper_id: &PaperId,
    blocks: &[Block],
) -> Result<(), SqliteAppStateStoreError> {
    transaction.execute(
        "DELETE FROM blocks WHERE paper_id = ?1",
        [paper_id.0.as_str()],
    )?;
    for block in blocks {
        upsert_block(transaction, block)?;
    }
    Ok(())
}

fn upsert_reading_progress(
    transaction: &Transaction<'_>,
    progress: &ReadingProgress,
) -> Result<(), SqliteAppStateStoreError> {
    transaction.execute(
        "
        INSERT INTO reading_progress (paper_id, current_block_index, current_block_id, updated_at_ms)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(paper_id) DO UPDATE SET
            current_block_index = excluded.current_block_index,
            current_block_id = excluded.current_block_id,
            updated_at_ms = excluded.updated_at_ms
        ",
        params![
            progress.paper_id.0.as_str(),
            i64::from(progress.current_block_index),
            progress.current_block_id.as_ref().map(|block_id| block_id.0.as_str()),
            to_sql_i64(progress.updated_at_ms, "reading_progress.updated_at_ms")?,
        ],
    )?;
    Ok(())
}

fn replace_reading_progress(
    transaction: &Transaction<'_>,
    progress: &ReadingProgress,
) -> Result<(), SqliteAppStateStoreError> {
    upsert_reading_progress(transaction, progress)
}

fn upsert_job(transaction: &Transaction<'_>, job: &OcrJob) -> Result<(), SqliteAppStateStoreError> {
    transaction.execute(
        "
        INSERT INTO ocr_jobs (
            id, paper_id, engine, status, requested_at_ms, started_at_ms, finished_at_ms, error_message
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
            paper_id = excluded.paper_id,
            engine = excluded.engine,
            status = excluded.status,
            requested_at_ms = excluded.requested_at_ms,
            started_at_ms = excluded.started_at_ms,
            finished_at_ms = excluded.finished_at_ms,
            error_message = excluded.error_message
        ",
        params![
            job.id.0.as_str(),
            job.paper_id.0.as_str(),
            ocr_engine_as_str(&job.engine),
            ocr_job_status_as_str(&job.status),
            to_sql_i64(job.requested_at_ms, "ocr_jobs.requested_at_ms")?,
            job.started_at_ms
                .map(|value| to_sql_i64(value, "ocr_jobs.started_at_ms"))
                .transpose()?,
            job.finished_at_ms
                .map(|value| to_sql_i64(value, "ocr_jobs.finished_at_ms"))
                .transpose()?,
            job.error_message.as_deref(),
        ],
    )?;
    Ok(())
}

fn upsert_ocr_job_result(
    transaction: &Transaction<'_>,
    result: &OcrJobResult,
) -> Result<(), SqliteAppStateStoreError> {
    transaction.execute(
        "
        INSERT INTO ocr_job_results (paper_id, text)
        VALUES (?1, ?2)
        ON CONFLICT(paper_id) DO UPDATE SET
            text = excluded.text
        ",
        params![result.paper_id.0.as_str(), result.text.as_str()],
    )?;
    Ok(())
}

fn map_paper_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Paper> {
    Ok(Paper {
        id: PaperId(row.get::<_, String>(0)?),
        title: row.get(1)?,
        source_type: parse_paper_source_type(&row.get::<_, String>(2)?)?,
        source_ref: row.get(3)?,
        status: parse_paper_status(&row.get::<_, String>(4)?)?,
        parse_version: get_u32(row, 5)?,
    })
}

fn map_block_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Block> {
    Ok(Block {
        id: BlockId(row.get::<_, String>(0)?),
        paper_id: PaperId(row.get::<_, String>(1)?),
        order_index: get_u32(row, 2)?,
        text: row.get(3)?,
        block_hash: row.get(4)?,
    })
}

fn map_reading_progress_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReadingProgress> {
    Ok(ReadingProgress {
        paper_id: PaperId(row.get::<_, String>(0)?),
        current_block_index: get_u32(row, 1)?,
        current_block_id: row.get::<_, Option<String>>(2)?.map(BlockId),
        updated_at_ms: get_u64(row, 3)?,
    })
}

fn map_ocr_job_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OcrJob> {
    Ok(OcrJob {
        id: OcrJobId(row.get::<_, String>(0)?),
        paper_id: PaperId(row.get::<_, String>(1)?),
        engine: parse_ocr_engine(&row.get::<_, String>(2)?)?,
        status: parse_ocr_job_status(&row.get::<_, String>(3)?)?,
        requested_at_ms: get_u64(row, 4)?,
        started_at_ms: get_optional_u64(row, 5)?,
        finished_at_ms: get_optional_u64(row, 6)?,
        error_message: row.get(7)?,
    })
}

fn map_ocr_job_result_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OcrJobResult> {
    Ok(OcrJobResult {
        paper_id: PaperId(row.get::<_, String>(0)?),
        text: row.get(1)?,
    })
}

fn get_u32(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u32> {
    let value = row.get::<_, i64>(index)?;
    u32::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })
}

fn get_u64(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value = row.get::<_, i64>(index)?;
    u64::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })
}

fn get_optional_u64(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<Option<u64>> {
    row.get::<_, Option<i64>>(index)?
        .map(|value| {
            u64::try_from(value).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    index,
                    rusqlite::types::Type::Integer,
                    Box::new(error),
                )
            })
        })
        .transpose()
}

fn to_sql_i64(value: u64, field: &'static str) -> Result<i64, SqliteAppStateStoreError> {
    i64::try_from(value).map_err(|_| SqliteAppStateStoreError::ValueOutOfRange(field))
}

fn paper_source_type_as_str(source_type: &PaperSourceType) -> &'static str {
    match source_type {
        PaperSourceType::Pdf => "Pdf",
        PaperSourceType::Paste => "Paste",
        PaperSourceType::Url => "Url",
        PaperSourceType::Doi => "Doi",
        PaperSourceType::Zotero => "Zotero",
    }
}

fn parse_paper_source_type(value: &str) -> rusqlite::Result<PaperSourceType> {
    match value {
        "Pdf" => Ok(PaperSourceType::Pdf),
        "Paste" => Ok(PaperSourceType::Paste),
        "Url" => Ok(PaperSourceType::Url),
        "Doi" => Ok(PaperSourceType::Doi),
        "Zotero" => Ok(PaperSourceType::Zotero),
        _ => Err(rusqlite::Error::InvalidColumnType(
            2,
            "source_type".to_string(),
            rusqlite::types::Type::Text,
        )),
    }
}

fn paper_status_as_str(status: &PaperStatus) -> &'static str {
    match status {
        PaperStatus::Created => "Created",
        PaperStatus::Ingesting => "Ingesting",
        PaperStatus::ReadyText => "ReadyText",
        PaperStatus::ReadyOcr => "ReadyOcr",
        PaperStatus::Failed => "Failed",
    }
}

fn parse_paper_status(value: &str) -> rusqlite::Result<PaperStatus> {
    match value {
        "Created" => Ok(PaperStatus::Created),
        "Ingesting" => Ok(PaperStatus::Ingesting),
        "ReadyText" => Ok(PaperStatus::ReadyText),
        "ReadyOcr" => Ok(PaperStatus::ReadyOcr),
        "Failed" => Ok(PaperStatus::Failed),
        _ => Err(rusqlite::Error::InvalidColumnType(
            4,
            "status".to_string(),
            rusqlite::types::Type::Text,
        )),
    }
}

fn ocr_engine_as_str(engine: &OcrEngine) -> &'static str {
    match engine {
        OcrEngine::PaddleOcr => "PaddleOcr",
    }
}

fn parse_ocr_engine(value: &str) -> rusqlite::Result<OcrEngine> {
    match value {
        "PaddleOcr" => Ok(OcrEngine::PaddleOcr),
        _ => Err(rusqlite::Error::InvalidColumnType(
            2,
            "engine".to_string(),
            rusqlite::types::Type::Text,
        )),
    }
}

fn ocr_job_status_as_str(status: &OcrJobStatus) -> &'static str {
    match status {
        OcrJobStatus::Queued => "Queued",
        OcrJobStatus::Processing => "Processing",
        OcrJobStatus::Completed => "Completed",
        OcrJobStatus::Failed => "Failed",
    }
}

fn parse_ocr_job_status(value: &str) -> rusqlite::Result<OcrJobStatus> {
    match value {
        "Queued" => Ok(OcrJobStatus::Queued),
        "Processing" => Ok(OcrJobStatus::Processing),
        "Completed" => Ok(OcrJobStatus::Completed),
        "Failed" => Ok(OcrJobStatus::Failed),
        _ => Err(rusqlite::Error::InvalidColumnType(
            3,
            "status".to_string(),
            rusqlite::types::Type::Text,
        )),
    }
}
