use paper_immersive_reader_core::domain::{PaperId, VocabularyItem};

use crate::storage::{StorageError, WebStorage};

use super::RouteDescriptor;

pub fn descriptor() -> RouteDescriptor {
    RouteDescriptor {
        path: "/vocabulary",
        label: "Vocabulary",
        description: "Review collected terms and lookup history.",
    }
}

pub struct VocabularyRouteModel {
    storage: WebStorage,
    paper_id: PaperId,
}

impl VocabularyRouteModel {
    pub fn new(storage: WebStorage, paper_id: PaperId) -> Self {
        Self { storage, paper_id }
    }

    pub fn entries(&self) -> Result<Vec<VocabularyItem>, StorageError> {
        self.storage.list_vocabulary_for_paper(&self.paper_id)
    }
}
