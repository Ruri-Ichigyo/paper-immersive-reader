use std::collections::HashMap;

use paper_immersive_reader_core::domain::{Block, Paper, PaperId, ReadingProgress};

use super::schema::PaperBundle;

#[derive(Debug, Clone, Default)]
pub struct PaperRecords {
    papers: HashMap<String, Paper>,
    blocks_by_paper: HashMap<String, Vec<Block>>,
    progress_by_paper: HashMap<String, ReadingProgress>,
    pdf_assets_by_paper: HashMap<String, Vec<u8>>,
}

impl PaperRecords {
    pub fn save_bundle(&mut self, bundle: PaperBundle) {
        let paper_key = bundle.paper.id.0.clone();
        let mut blocks = bundle.blocks;
        blocks.sort_by_key(|block| block.order_index);

        self.papers.insert(paper_key.clone(), bundle.paper);
        self.blocks_by_paper.insert(paper_key.clone(), blocks);
        self.progress_by_paper
            .insert(paper_key, bundle.reading_progress);
    }

    pub fn load_bundle(&self, paper_id: &PaperId) -> Option<PaperBundle> {
        let paper_key = &paper_id.0;
        Some(PaperBundle {
            paper: self.papers.get(paper_key)?.clone(),
            blocks: self.blocks_by_paper.get(paper_key)?.clone(),
            reading_progress: self.progress_by_paper.get(paper_key)?.clone(),
        })
    }

    pub fn list_papers(&self) -> Vec<Paper> {
        let mut papers = self.papers.values().cloned().collect::<Vec<_>>();
        papers.sort_by(|left, right| left.id.0.cmp(&right.id.0));
        papers
    }

    pub fn save_pdf_asset(&mut self, paper_id: &PaperId, bytes: Vec<u8>) {
        self.pdf_assets_by_paper.insert(paper_id.0.clone(), bytes);
    }

    pub fn load_pdf_asset(&self, paper_id: &PaperId) -> Option<Vec<u8>> {
        self.pdf_assets_by_paper.get(&paper_id.0).cloned()
    }
}
