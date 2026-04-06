use paper_immersive_reader_core::domain::{PaperId, PaperStatus};

use crate::storage::{PaperBundle, StorageError, WebStorage};

use super::RouteDescriptor;

pub fn descriptor() -> RouteDescriptor {
    RouteDescriptor {
        path: "/library",
        label: "Library",
        description: "Browse imported papers and resume reading.",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryPaperSummary {
    pub paper_id: PaperId,
    pub title: Option<String>,
    pub status: PaperStatus,
    pub current_block_index: u32,
}

#[derive(Debug, Clone)]
pub struct LibraryRouteModel {
    storage: WebStorage,
}

impl LibraryRouteModel {
    pub fn new(storage: WebStorage) -> Self {
        Self { storage }
    }

    pub fn papers(&self) -> Result<Vec<LibraryPaperSummary>, StorageError> {
        self.storage
            .list_papers()?
            .into_iter()
            .map(|paper| {
                let bundle = self.storage.load_paper_bundle(&paper.id)?;
                Ok(build_summary(paper, bundle))
            })
            .collect()
    }
}

fn build_summary(
    paper: paper_immersive_reader_core::domain::Paper,
    bundle: Option<PaperBundle>,
) -> LibraryPaperSummary {
    let current_block_index = bundle
        .map(|loaded| loaded.reading_progress.current_block_index)
        .unwrap_or(0);

    LibraryPaperSummary {
        paper_id: paper.id,
        title: paper.title,
        status: paper.status,
        current_block_index,
    }
}
