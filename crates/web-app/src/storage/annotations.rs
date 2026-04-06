use std::collections::BTreeMap;

use paper_immersive_reader_core::domain::{Annotation, PaperId};

#[derive(Debug, Clone, Default)]
pub struct AnnotationRecords {
    by_paper: BTreeMap<String, BTreeMap<String, Annotation>>,
}

impl AnnotationRecords {
    pub fn save(&mut self, annotation: Annotation) {
        self.by_paper
            .entry(annotation.paper_id.0.clone())
            .or_default()
            .insert(annotation.id.0.clone(), annotation);
    }

    pub fn list_for_paper(&self, paper_id: &PaperId) -> Vec<Annotation> {
        self.by_paper
            .get(&paper_id.0)
            .map(|annotations| annotations.values().cloned().collect())
            .unwrap_or_default()
    }
}
