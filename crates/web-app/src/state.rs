#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

use leptos::prelude::*;
use paper_immersive_reader_core::domain::PaperId;

use crate::storage::memory_store::InMemoryStore;
use crate::storage::schema::Settings;
use crate::storage::AppStorage;

/// Global app state provided via Leptos context.
///
/// `RwSignal<InMemoryStore>` provides interior mutability that is
/// `Send + Sync`.  Components touch `data_version` to create a reactive
/// dependency, then call `store.with(|s| …)` for reads or
/// `store.update(|s| …)` for writes.
#[derive(Copy, Clone)]
pub struct AppState {
    pub store: RwSignal<InMemoryStore>,
    pub data_version: RwSignal<u64>,
    pub current_paper_id: RwSignal<Option<PaperId>>,
    pub settings: RwSignal<Settings>,
    pub gateway_url: RwSignal<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// --- IndexedDB persistence (wasm32 only) ---

#[cfg(target_arch = "wasm32")]
thread_local! {
    static IDB: RefCell<Option<crate::storage::indexed_db_store::IndexedDbStore>> =
        const { RefCell::new(None) };
}

impl AppState {
    pub fn new() -> Self {
        Self {
            store: RwSignal::new(InMemoryStore::new()),
            data_version: RwSignal::new(0),
            current_paper_id: RwSignal::new(None),
            settings: RwSignal::new(Settings::default()),
            gateway_url: RwSignal::new("http://127.0.0.1:3000".to_string()),
        }
    }

    /// Increment after any mutation so dependent views re-render.
    pub fn notify(&self) {
        self.data_version.set(self.data_version.get() + 1);
    }

    // -- convenience readers (each touches `data_version` for reactivity) --

    pub fn papers(&self) -> Vec<crate::storage::schema::StoredPaper> {
        self.data_version.get();
        self.store.with(|s| s.list_papers().unwrap_or_default())
    }

    pub fn blocks(&self, paper_id: &PaperId) -> Vec<paper_immersive_reader_core::domain::Block> {
        self.data_version.get();
        self.store
            .with(|s| s.load_blocks(paper_id).unwrap_or_default())
    }

    pub fn progress(
        &self,
        paper_id: &PaperId,
    ) -> Option<paper_immersive_reader_core::domain::ReadingProgress> {
        self.data_version.get();
        self.store
            .with(|s| s.load_progress(paper_id).unwrap_or(None))
    }

    pub fn annotations(
        &self,
        paper_id: &PaperId,
    ) -> Vec<paper_immersive_reader_core::domain::Annotation> {
        self.data_version.get();
        self.store
            .with(|s| s.load_annotations(paper_id).unwrap_or_default())
    }

    pub fn vocabulary(&self) -> Vec<paper_immersive_reader_core::domain::VocabularyItem> {
        self.data_version.get();
        self.store.with(|s| s.list_vocabulary().unwrap_or_default())
    }

    pub fn ai_records(&self, paper_id: &PaperId) -> Vec<crate::storage::schema::AiRecord> {
        self.data_version.get();
        self.store
            .with(|s| s.load_ai_records(paper_id).unwrap_or_default())
    }

    pub fn settings(&self) -> Settings {
        self.settings.get()
    }

    // -- mutation helpers --

    pub fn save_paper(&self, paper: &crate::storage::schema::StoredPaper) {
        self.store.update(|s| {
            s.save_paper(paper).ok();
        });
        self.notify();
        self.persist_paper(paper.clone());
    }

    pub fn save_blocks(&self, blocks: &[paper_immersive_reader_core::domain::Block]) {
        self.store.update(|s| {
            s.save_blocks(blocks).ok();
        });
        self.notify();
        self.persist_blocks(blocks.to_vec());
    }

    pub fn save_progress(&self, progress: &paper_immersive_reader_core::domain::ReadingProgress) {
        self.store.update(|s| {
            s.save_progress(progress).ok();
        });
        self.notify();
        self.persist_progress(progress.clone());
    }

    pub fn save_annotation(&self, annotation: &paper_immersive_reader_core::domain::Annotation) {
        self.store.update(|s| {
            s.save_annotation(annotation).ok();
        });
        self.notify();
        self.persist_annotation(annotation.clone());
    }

    pub fn upsert_vocabulary(&self, item: &paper_immersive_reader_core::domain::VocabularyItem) {
        self.store.update(|s| {
            s.upsert_vocabulary(item).ok();
        });
        self.notify();
        self.persist_vocabulary(item.clone());
    }

    pub fn update_vocabulary(&self, item: &paper_immersive_reader_core::domain::VocabularyItem) {
        self.store.update(|s| {
            s.update_vocabulary(item).ok();
        });
        self.notify();
        self.persist_vocabulary_update(item.clone());
    }

    pub fn save_ai_record(&self, record: &crate::storage::schema::AiRecord) {
        self.store.update(|s| {
            s.save_ai_record(record).ok();
        });
        self.notify();
        self.persist_ai_record(record.clone());
    }

    pub fn save_settings(&self, settings: &Settings) {
        self.store.update(|s| {
            s.save_settings(settings).ok();
        });
        self.settings.set(settings.clone());
        self.notify();
        self.persist_settings(settings.clone());
    }

    /// Replace the underlying store entirely (used after WebImportService
    /// consumes and returns a new store).
    pub fn replace_store(&self, new_store: InMemoryStore) {
        // Extract data to persist before moving the store
        #[cfg(target_arch = "wasm32")]
        {
            let papers = new_store.list_papers().unwrap_or_default();
            let vocab = new_store.list_vocabulary().unwrap_or_default();
            let settings = new_store.load_settings().unwrap_or_default();
            let mut paper_data: Vec<(
                crate::storage::schema::StoredPaper,
                Vec<paper_immersive_reader_core::domain::Block>,
                Option<paper_immersive_reader_core::domain::ReadingProgress>,
                Vec<paper_immersive_reader_core::domain::Annotation>,
                Vec<crate::storage::schema::AiRecord>,
            )> = Vec::new();
            for sp in &papers {
                let blocks = new_store.load_blocks(&sp.paper.id).unwrap_or_default();
                let progress = new_store.load_progress(&sp.paper.id).unwrap_or(None);
                let anns = new_store.load_annotations(&sp.paper.id).unwrap_or_default();
                let ai = new_store.load_ai_records(&sp.paper.id).unwrap_or_default();
                paper_data.push((sp.clone(), blocks, progress, anns, ai));
            }
            self.persist_all(paper_data, vocab, settings);
        }

        self.store.set(new_store);
        self.notify();
    }

    /// Clone the current store (used to seed a WebImportService).
    pub fn clone_store(&self) -> InMemoryStore {
        self.store.get()
    }

    // -- IndexedDB persistence (wasm32 only) --

    /// Open IndexedDB and merge persisted data into the in-memory store.
    /// Called once on app startup.  Merges (not replaces) so that data
    /// written between app start and load completion is preserved.
    #[cfg(target_arch = "wasm32")]
    pub fn init_persistence(&self) {
        let app = *self;
        wasm_bindgen_futures::spawn_local(async move {
            match crate::storage::indexed_db_store::IndexedDbStore::open().await {
                Ok(db) => {
                    IDB.with(|cell| *cell.borrow_mut() = Some(db.clone()));

                    // Merge loaded data into the existing in-memory store
                    // (not replace) to avoid losing data written during load.
                    if let Ok(papers) = db.list_papers().await {
                        for sp in &papers {
                            let blocks = db.load_blocks(&sp.paper.id).await.unwrap_or_default();
                            let progress = db.load_progress(&sp.paper.id).await.ok().flatten();
                            let anns = db.load_annotations(&sp.paper.id).await.unwrap_or_default();
                            let records =
                                db.load_ai_records(&sp.paper.id).await.unwrap_or_default();

                            app.store.update(|s| {
                                s.save_paper(sp).ok();
                                s.save_blocks(&blocks).ok();
                                if let Some(p) = &progress {
                                    s.save_progress(p).ok();
                                }
                                for a in &anns {
                                    s.save_annotation(a).ok();
                                }
                                for r in &records {
                                    s.save_ai_record(r).ok();
                                }
                            });
                        }
                    }

                    if let Ok(vocab) = db.list_vocabulary().await {
                        for v in &vocab {
                            app.store.update(|s| {
                                s.upsert_vocabulary(v).ok();
                            });
                        }
                    }

                    if let Ok(settings) = db.load_settings().await {
                        app.store.update(|s| {
                            s.save_settings(&settings).ok();
                        });
                        app.settings.set(settings.clone());
                        // Sync gateway_url from persisted settings so AI
                        // requests use the user-configured URL after reload.
                        if let Some(ref url) = settings.ai_gateway_url {
                            if !url.is_empty() {
                                app.gateway_url.set(url.clone());
                            }
                        }
                    }

                    app.notify();
                }
                Err(e) => {
                    web_sys::console::warn_1(
                        &format!("IndexedDB unavailable, running in memory: {:?}", e).into(),
                    );
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn init_persistence(&self) {
        // No-op on native
    }

    #[cfg(target_arch = "wasm32")]
    fn persist_paper(&self, paper: crate::storage::schema::StoredPaper) {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(db) = IDB.with(|cell| cell.borrow().as_ref().cloned()) {
                db.save_paper(&paper).await.ok();
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn persist_paper(&self, _paper: crate::storage::schema::StoredPaper) {}

    #[cfg(target_arch = "wasm32")]
    fn persist_blocks(&self, blocks: Vec<paper_immersive_reader_core::domain::Block>) {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(db) = IDB.with(|cell| cell.borrow().as_ref().cloned()) {
                db.save_blocks(&blocks).await.ok();
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn persist_blocks(&self, _blocks: Vec<paper_immersive_reader_core::domain::Block>) {}

    #[cfg(target_arch = "wasm32")]
    fn persist_progress(&self, progress: paper_immersive_reader_core::domain::ReadingProgress) {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(db) = IDB.with(|cell| cell.borrow().as_ref().cloned()) {
                db.save_progress(&progress).await.ok();
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn persist_progress(&self, _progress: paper_immersive_reader_core::domain::ReadingProgress) {}

    #[cfg(target_arch = "wasm32")]
    fn persist_annotation(&self, annotation: paper_immersive_reader_core::domain::Annotation) {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(db) = IDB.with(|cell| cell.borrow().as_ref().cloned()) {
                db.save_annotation(&annotation).await.ok();
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn persist_annotation(&self, _annotation: paper_immersive_reader_core::domain::Annotation) {}

    #[cfg(target_arch = "wasm32")]
    fn persist_vocabulary(&self, item: paper_immersive_reader_core::domain::VocabularyItem) {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(db) = IDB.with(|cell| cell.borrow().as_ref().cloned()) {
                db.upsert_vocabulary(&item).await.ok();
            }
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn persist_vocabulary_update(&self, item: paper_immersive_reader_core::domain::VocabularyItem) {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(db) = IDB.with(|cell| cell.borrow().as_ref().cloned()) {
                db.update_vocabulary(&item).await.ok();
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn persist_vocabulary(&self, _item: paper_immersive_reader_core::domain::VocabularyItem) {}

    #[cfg(not(target_arch = "wasm32"))]
    fn persist_vocabulary_update(
        &self,
        _item: paper_immersive_reader_core::domain::VocabularyItem,
    ) {
    }

    #[cfg(target_arch = "wasm32")]
    fn persist_ai_record(&self, record: crate::storage::schema::AiRecord) {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(db) = IDB.with(|cell| cell.borrow().as_ref().cloned()) {
                db.save_ai_record(&record).await.ok();
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn persist_ai_record(&self, _record: crate::storage::schema::AiRecord) {}

    #[cfg(target_arch = "wasm32")]
    fn persist_settings(&self, settings: Settings) {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(db) = IDB.with(|cell| cell.borrow().as_ref().cloned()) {
                db.save_settings(&settings).await.ok();
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn persist_settings(&self, _settings: Settings) {}

    /// Persist all data from a new store to IndexedDB (used after replace_store).
    #[cfg(target_arch = "wasm32")]
    fn persist_all(
        &self,
        paper_data: Vec<(
            crate::storage::schema::StoredPaper,
            Vec<paper_immersive_reader_core::domain::Block>,
            Option<paper_immersive_reader_core::domain::ReadingProgress>,
            Vec<paper_immersive_reader_core::domain::Annotation>,
            Vec<crate::storage::schema::AiRecord>,
        )>,
        vocab: Vec<paper_immersive_reader_core::domain::VocabularyItem>,
        settings: Settings,
    ) {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(db) = IDB.with(|cell| cell.borrow().as_ref().cloned()) {
                for (paper, blocks, progress, anns, ai_records) in &paper_data {
                    db.save_paper(paper).await.ok();
                    db.save_blocks(blocks).await.ok();
                    if let Some(p) = progress {
                        db.save_progress(p).await.ok();
                    }
                    for a in anns {
                        db.save_annotation(a).await.ok();
                    }
                    for r in ai_records {
                        db.save_ai_record(r).await.ok();
                    }
                }
                for v in &vocab {
                    db.upsert_vocabulary(v).await.ok();
                }
                db.save_settings(&settings).await.ok();
            }
        });
    }
}

/// Convenience to grab the `AppState` from context.
pub fn use_app_state() -> AppState {
    expect_context::<AppState>()
}
