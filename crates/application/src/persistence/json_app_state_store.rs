use std::fs;
use std::path::{Path, PathBuf};

use paper_immersive_reader_core::domain::{
    Block, OcrJob, OcrJobId, OcrJobResult, OcrJobStatus, Paper, PaperId, ReadingProgress,
};
use paper_immersive_reader_importing::{ImportResult, TextImportArtifacts};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AppState {
    pub papers: Vec<Paper>,
    pub blocks: Vec<Block>,
    pub reading_progress: Vec<ReadingProgress>,
    pub ocr_jobs: Vec<OcrJob>,
    pub ocr_job_results: Vec<OcrJobResult>,
}

pub struct JsonAppStateStore {
    path: PathBuf,
    snapshot: AppState,
}

impl JsonAppStateStore {
    pub fn load_migration_input(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref().to_path_buf();

        let snapshot = match fs::read_to_string(&path) {
            Ok(contents) if contents.trim().is_empty() => AppState::default(),
            Ok(contents) => serde_json::from_str(&contents)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => AppState::default(),
            Err(error) => return Err(error),
        };

        Ok(Self { path, snapshot })
    }

    pub fn snapshot(&self) -> &AppState {
        &self.snapshot
    }

    pub fn snapshot_mut(&mut self) -> &mut AppState {
        &mut self.snapshot
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
            fs::create_dir_all(parent)?;
        }
        fs::write(&asset_path, bytes)?;
        Ok(asset_path)
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = self.path.with_extension("json.tmp");
        let contents = serde_json::to_string_pretty(&self.snapshot)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;

        fs::write(&temp_path, contents)?;
        fs::rename(temp_path, &self.path)?;
        Ok(())
    }

    pub fn save_import_result(&mut self, result: ImportResult) -> Result<(), std::io::Error> {
        self.replace_paper(result.paper.clone());
        self.snapshot
            .blocks
            .retain(|block| block.paper_id != result.paper.id);
        self.snapshot
            .reading_progress
            .retain(|progress| progress.paper_id != result.paper.id);
        self.snapshot.blocks.extend(result.blocks);
        self.snapshot.reading_progress.push(result.reading_progress);
        self.save()
    }

    pub fn save_paper_and_job(&mut self, paper: Paper, job: OcrJob) -> Result<(), std::io::Error> {
        self.replace_paper(paper);
        self.replace_job(job);
        self.save()
    }

    pub fn get_job(&self, job_id: &OcrJobId) -> Option<OcrJob> {
        self.snapshot
            .ocr_jobs
            .iter()
            .find(|job| &job.id == job_id)
            .cloned()
    }

    pub fn first_queued_job(&self) -> Option<OcrJob> {
        self.snapshot
            .ocr_jobs
            .iter()
            .find(|job| job.status == OcrJobStatus::Queued)
            .cloned()
    }

    pub fn get_paper(&self, paper_id: &PaperId) -> Option<Paper> {
        self.snapshot
            .papers
            .iter()
            .find(|paper| &paper.id == paper_id)
            .cloned()
    }

    pub fn save_job(&mut self, job: OcrJob) -> Result<(), std::io::Error> {
        self.replace_job(job);
        self.save()
    }

    pub fn save_completed_ocr(
        &mut self,
        completed_job: OcrJob,
        ocr_result: OcrJobResult,
        artifacts: TextImportArtifacts,
        updated_paper: Paper,
    ) -> Result<(), std::io::Error> {
        self.replace_paper(updated_paper.clone());
        self.replace_job(completed_job);
        self.snapshot
            .ocr_job_results
            .retain(|result| result.paper_id != updated_paper.id);
        self.snapshot.ocr_job_results.push(ocr_result);
        self.snapshot
            .blocks
            .retain(|block| block.paper_id != updated_paper.id);
        self.snapshot.blocks.extend(artifacts.blocks);
        self.snapshot
            .reading_progress
            .retain(|progress| progress.paper_id != updated_paper.id);
        self.snapshot
            .reading_progress
            .push(artifacts.reading_progress);
        self.save()
    }

    fn replace_paper(&mut self, paper: Paper) {
        if let Some(existing) = self
            .snapshot
            .papers
            .iter_mut()
            .find(|existing| existing.id == paper.id)
        {
            *existing = paper;
        } else {
            self.snapshot.papers.push(paper);
        }
    }

    fn replace_job(&mut self, job: OcrJob) {
        if let Some(existing) = self
            .snapshot
            .ocr_jobs
            .iter_mut()
            .find(|existing| existing.id == job.id)
        {
            *existing = job;
        } else {
            self.snapshot.ocr_jobs.push(job);
        }
    }
}
