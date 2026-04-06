use std::collections::BTreeMap;

use paper_immersive_reader_core::domain::{PaperId, VocabularyItem};

#[derive(Debug, Clone, Default)]
pub struct VocabularyRecords {
    by_paper: BTreeMap<String, BTreeMap<String, VocabularyItem>>,
}

impl VocabularyRecords {
    pub fn upsert_lookup(&mut self, mut incoming: VocabularyItem) -> VocabularyItem {
        let paper_key = incoming.paper_id_first_seen.0.clone();
        let term_key = normalize_term(&incoming.term);
        incoming.term = term_key.clone();

        let paper_terms = self.by_paper.entry(paper_key).or_default();

        match paper_terms.get_mut(&term_key) {
            Some(existing) => {
                existing.lookup_count = existing.lookup_count.saturating_add(incoming.lookup_count);
                existing.last_seen_at_ms = existing.last_seen_at_ms.max(incoming.last_seen_at_ms);
                if existing.note.is_none() {
                    existing.note = incoming.note;
                }
                existing.learned = existing.learned || incoming.learned;
                existing.clone()
            }
            None => {
                paper_terms.insert(term_key, incoming.clone());
                incoming
            }
        }
    }

    pub fn list_for_paper(&self, paper_id: &PaperId) -> Vec<VocabularyItem> {
        self.by_paper
            .get(&paper_id.0)
            .map(|terms| terms.values().cloned().collect())
            .unwrap_or_default()
    }
}

fn normalize_term(term: &str) -> String {
    term.trim().to_lowercase()
}
