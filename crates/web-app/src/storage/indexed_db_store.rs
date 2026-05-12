use js_sys::JSON;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbRequest, IdbTransactionMode};

use paper_immersive_reader_core::domain::{
    Annotation, AnnotationId, Block, PaperId, ReadingProgress, VocabularyItem,
};

use super::schema::{AiRecord, Settings, StoredPaper};
use super::StorageError;

const DB_NAME: &str = "paper-immersive-reader";
const DB_VERSION: u32 = 1;

fn show_js_err(e: &JsValue) -> String {
    format!("{:?}", e)
}

fn to_json<T: serde::Serialize>(v: &T) -> Result<JsValue, StorageError> {
    let json = serde_json::to_string(v).map_err(|e| StorageError::IoError(e.to_string()))?;
    Ok(JsValue::from_str(&json))
}

fn from_json<T: serde::de::DeserializeOwned>(val: &JsValue) -> Result<T, StorageError> {
    if val.is_undefined() || val.is_null() {
        return Err(StorageError::NotFound);
    }
    let json = JSON::stringify(val)
        .map(|j| j.as_string().unwrap_or_default())
        .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
    serde_json::from_str(&json).map_err(|e| StorageError::IoError(e.to_string()))
}

fn from_json_opt<T: serde::de::DeserializeOwned>(val: &JsValue) -> Result<Option<T>, StorageError> {
    if val.is_undefined() || val.is_null() {
        return Ok(None);
    }
    Ok(Some(from_json(val)?))
}

fn arr_from_json<T: serde::de::DeserializeOwned>(val: &JsValue) -> Result<Vec<T>, StorageError> {
    if val.is_undefined() || val.is_null() {
        return Ok(vec![]);
    }
    let arr: js_sys::Array = val.clone().into();
    let mut out = Vec::with_capacity(arr.length() as usize);
    for i in 0..arr.length() {
        if let Ok(item) = from_json(&arr.get(i)) {
            out.push(item);
        }
    }
    Ok(out)
}

fn get_store(
    txn: &web_sys::IdbTransaction,
    name: &str,
) -> Result<web_sys::IdbObjectStore, StorageError> {
    txn.object_store(name)
        .map_err(|e| StorageError::IoError(show_js_err(&e)))
}

fn get_index(
    store: &web_sys::IdbObjectStore,
    name: &str,
) -> Result<web_sys::IdbIndex, StorageError> {
    store
        .index(name)
        .map_err(|e| StorageError::IoError(show_js_err(&e)))
}

/// Wrap an IdbRequest (or IdbOpenDbRequest) into a future.
async fn await_idb<T>(req: &IdbRequest) -> Result<T, StorageError>
where
    T: wasm_bindgen::JsCast + 'static,
{
    let promise =
        js_sys::Promise::new(&mut |resolve: js_sys::Function, reject: js_sys::Function| {
            let req_c = req.clone();
            let on_ok = wasm_bindgen::closure::Closure::once_into_js(move || {
                let r: JsValue = req_c.result().unwrap_or_else(|_| JsValue::null());
                let _ = resolve.call1(&JsValue::null(), &r);
            });
            let req_e = req.clone();
            let on_err = wasm_bindgen::closure::Closure::once_into_js(move || {
                let e = req_e
                    .error()
                    .unwrap_or_else(|_| JsValue::from_str("IndexedDB error"));
                let _ = reject.call1(&JsValue::null(), &e);
            });
            req.set_onsuccess(Some(on_ok.as_ref().unchecked_ref()));
            req.set_onerror(Some(on_err.as_ref().unchecked_ref()));
        });
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| StorageError::IoError(format!("IndexedDB: {}", show_js_err(&e))))?;
    if result.is_null() || result.is_undefined() {
        // Return a "null" typed value; caller checks for null.
        return Err(StorageError::NotFound);
    }
    result
        .dyn_into::<T>()
        .map_err(|_| StorageError::IoError("unexpected IDB result type".into()))
}

/// Await an IdbRequest, returning raw JsValue.
async fn await_val(req: &IdbRequest) -> Result<JsValue, StorageError> {
    await_idb::<JsValue>(req).await.or_else(|e| {
        // NotFound just means the key doesn't exist; return undefined.
        if matches!(e, StorageError::NotFound) {
            Ok(JsValue::undefined())
        } else {
            Err(e)
        }
    })
}

fn txn(
    db: &IdbDatabase,
    stores: &[&str],
    mode: IdbTransactionMode,
) -> Result<web_sys::IdbTransaction, StorageError> {
    let list = js_sys::Array::new();
    for s in stores {
        list.push(&JsValue::from_str(s));
    }
    db.transaction_with_str_sequence_and_mode(&list, mode)
        .map_err(|e| StorageError::IoError(show_js_err(&e)))
}

fn key_range_only(key: &str) -> Result<web_sys::IdbKeyRange, StorageError> {
    web_sys::IdbKeyRange::only(&JsValue::from_str(key))
        .map_err(|e| StorageError::IoError(show_js_err(&e)))
}

fn create_stores(db: &IdbDatabase) {
    let names: Vec<String> = db
        .object_store_names()
        .iter()
        .map(|n| n.as_string().unwrap_or_default())
        .collect();

    if !names.contains(&"papers".to_string()) {
        let _ = db.create_object_store("papers");
    }
    if !names.contains(&"blocks".to_string()) {
        let store = db.create_object_store("blocks");
        let _ = store.create_index_with_str("paper_id", "paper_id");
    }
    if !names.contains(&"progress".to_string()) {
        let _ = db.create_object_store("progress");
    }
    if !names.contains(&"annotations".to_string()) {
        let store = db.create_object_store("annotations");
        let _ = store.create_index_with_str("paper_id", "paper_id");
    }
    if !names.contains(&"vocabulary".to_string()) {
        let store = db.create_object_store("vocabulary");
        let _ = store.create_index_with_str("term_lower", "term_lower");
    }
    if !names.contains(&"ai_records".to_string()) {
        let store = db.create_object_store("ai_records");
        let _ = store.create_index_with_str("paper_id", "paper_id");
    }
    if !names.contains(&"settings".to_string()) {
        let _ = db.create_object_store("settings");
    }
}

// --- Public API ---

pub struct IndexedDbStore {
    db: IdbDatabase,
}

impl IndexedDbStore {
    pub async fn open() -> Result<Self, StorageError> {
        let factory = web_sys::window()
            .ok_or_else(|| StorageError::IoError("no window".into()))?
            .indexed_db()
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;

        let request = factory
            .open_with_u32(DB_NAME, DB_VERSION)
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;

        request.set_onupgradeneeded(Some(&wasm_bindgen::closure::Closure::once_into_js(
            move |event: web_sys::IdbVersionChangeEvent| {
                let db: Option<IdbDatabase> = event
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::IdbOpenDbRequest>().ok())
                    .and_then(|r| r.result().ok())
                    .and_then(|r| r.dyn_into::<IdbDatabase>().ok());
                if let Some(db) = db {
                    create_stores(&db);
                }
            },
        )));

        let db = await_idb::<IdbDatabase>(request.as_ref()).await?;
        Ok(Self { db })
    }

    // Papers
    pub async fn save_paper(&self, paper: &StoredPaper) -> Result<(), StorageError> {
        let txn = txn(&self.db, &["papers"], IdbTransactionMode::Readwrite)?;
        let store = get_store(&txn, "papers")?;
        let val = to_json(paper)?;
        let req = store
            .put_with_key(&val, &JsValue::from_str(&paper.paper.id.0))
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        await_val(&req).await?;
        Ok(())
    }

    pub async fn load_paper(&self, id: &PaperId) -> Result<Option<StoredPaper>, StorageError> {
        let txn = txn(&self.db, &["papers"], IdbTransactionMode::Readonly)?;
        let store = get_store(&txn, "papers")?;
        let req = store
            .get(&JsValue::from_str(&id.0))
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        let val = await_val(&req).await?;
        from_json_opt(&val)
    }

    pub async fn list_papers(&self) -> Result<Vec<StoredPaper>, StorageError> {
        let txn = txn(&self.db, &["papers"], IdbTransactionMode::Readonly)?;
        let store = get_store(&txn, "papers")?;
        let req = store
            .get_all()
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        let val = await_val(&req).await?;
        arr_from_json(&val)
    }

    // Blocks
    pub async fn save_blocks(&self, blocks: &[Block]) -> Result<(), StorageError> {
        if blocks.is_empty() {
            return Ok(());
        }
        let txn = txn(&self.db, &["blocks"], IdbTransactionMode::Readwrite)?;
        let store = get_store(&txn, "blocks")?;
        for block in blocks {
            let val = to_json(block)?;
            let req = store
                .put_with_key(&val, &JsValue::from_str(&block.id.0))
                .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
            await_val(&req).await?;
        }
        Ok(())
    }

    pub async fn load_blocks(&self, paper_id: &PaperId) -> Result<Vec<Block>, StorageError> {
        let txn = txn(&self.db, &["blocks"], IdbTransactionMode::Readonly)?;
        let store = get_store(&txn, "blocks")?;
        let idx = get_index(&store, "paper_id")?;
        let range = key_range_only(&paper_id.0)?;
        let req = idx
            .get_all_with_key(&range)
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        let val = await_val(&req).await?;
        let mut blocks: Vec<Block> = arr_from_json(&val)?;
        blocks.sort_by_key(|b| b.order_index);
        Ok(blocks)
    }

    // Progress
    pub async fn save_progress(&self, progress: &ReadingProgress) -> Result<(), StorageError> {
        let txn = txn(&self.db, &["progress"], IdbTransactionMode::Readwrite)?;
        let store = get_store(&txn, "progress")?;
        let val = to_json(progress)?;
        let req = store
            .put_with_key(&val, &JsValue::from_str(&progress.paper_id.0))
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        await_val(&req).await?;
        Ok(())
    }

    pub async fn load_progress(
        &self,
        paper_id: &PaperId,
    ) -> Result<Option<ReadingProgress>, StorageError> {
        let txn = txn(&self.db, &["progress"], IdbTransactionMode::Readonly)?;
        let store = get_store(&txn, "progress")?;
        let req = store
            .get(&JsValue::from_str(&paper_id.0))
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        let val = await_val(&req).await?;
        from_json_opt(&val)
    }

    // Annotations
    pub async fn save_annotation(&self, annotation: &Annotation) -> Result<(), StorageError> {
        let txn = txn(&self.db, &["annotations"], IdbTransactionMode::Readwrite)?;
        let store = get_store(&txn, "annotations")?;
        let val = to_json(annotation)?;
        let req = store
            .put_with_key(&val, &JsValue::from_str(&annotation.id.0))
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        await_val(&req).await?;
        Ok(())
    }

    pub async fn load_annotations(
        &self,
        paper_id: &PaperId,
    ) -> Result<Vec<Annotation>, StorageError> {
        let txn = txn(&self.db, &["annotations"], IdbTransactionMode::Readonly)?;
        let store = get_store(&txn, "annotations")?;
        let idx = get_index(&store, "paper_id")?;
        let range = key_range_only(&paper_id.0)?;
        let req = idx
            .get_all_with_key(&range)
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        let val = await_val(&req).await?;
        arr_from_json(&val)
    }

    pub async fn load_annotation(
        &self,
        id: &AnnotationId,
    ) -> Result<Option<Annotation>, StorageError> {
        let txn = txn(&self.db, &["annotations"], IdbTransactionMode::Readonly)?;
        let store = get_store(&txn, "annotations")?;
        let req = store
            .get(&JsValue::from_str(&id.0))
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        let val = await_val(&req).await?;
        from_json_opt(&val)
    }

    // Vocabulary
    pub async fn upsert_vocabulary(&self, item: &VocabularyItem) -> Result<(), StorageError> {
        let term_lower = item.term.to_lowercase();
        let txn = txn(&self.db, &["vocabulary"], IdbTransactionMode::Readwrite)?;
        let store = get_store(&txn, "vocabulary")?;
        let idx = get_index(&store, "term_lower")?;
        let range = key_range_only(&term_lower)?;
        let req = idx
            .get(&range)
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        let val = await_val(&req).await?;

        if val.is_undefined() || val.is_null() {
            let mut new_item = item.clone();
            new_item.term = term_lower;
            let jv = to_json(&new_item)?;
            let req = store
                .put_with_key(&jv, &JsValue::from_str(&item.id.0))
                .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
            await_val(&req).await?;
        } else {
            let mut existing: VocabularyItem = from_json(&val)?;
            existing.lookup_count += 1;
            existing.last_seen_at_ms = item.last_seen_at_ms;
            if item.note.is_some() {
                existing.note = item.note.clone();
            }
            let jv = to_json(&existing)?;
            let req = store
                .put_with_key(&jv, &JsValue::from_str(&existing.id.0))
                .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
            await_val(&req).await?;
        }
        Ok(())
    }

    pub async fn list_vocabulary(&self) -> Result<Vec<VocabularyItem>, StorageError> {
        let txn = txn(&self.db, &["vocabulary"], IdbTransactionMode::Readonly)?;
        let store = get_store(&txn, "vocabulary")?;
        let req = store
            .get_all()
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        let val = await_val(&req).await?;
        arr_from_json(&val)
    }

    // AI records
    pub async fn save_ai_record(&self, record: &AiRecord) -> Result<(), StorageError> {
        let txn = txn(&self.db, &["ai_records"], IdbTransactionMode::Readwrite)?;
        let store = get_store(&txn, "ai_records")?;
        let val = to_json(record)?;
        let req = store
            .put_with_key(&val, &JsValue::from_str(&record.id))
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        await_val(&req).await?;
        Ok(())
    }

    pub async fn load_ai_records(&self, paper_id: &PaperId) -> Result<Vec<AiRecord>, StorageError> {
        let txn = txn(&self.db, &["ai_records"], IdbTransactionMode::Readonly)?;
        let store = get_store(&txn, "ai_records")?;
        let idx = get_index(&store, "paper_id")?;
        let range = key_range_only(&paper_id.0)?;
        let req = idx
            .get_all_with_key(&range)
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        let val = await_val(&req).await?;
        arr_from_json(&val)
    }

    // Settings
    pub async fn save_settings(&self, settings: &Settings) -> Result<(), StorageError> {
        let txn = txn(&self.db, &["settings"], IdbTransactionMode::Readwrite)?;
        let store = get_store(&txn, "settings")?;
        let val = to_json(settings)?;
        let req = store
            .put_with_key(&val, &JsValue::from_str("app_settings"))
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        await_val(&req).await?;
        Ok(())
    }

    pub async fn load_settings(&self) -> Result<Settings, StorageError> {
        let txn = txn(&self.db, &["settings"], IdbTransactionMode::Readonly)?;
        let store = get_store(&txn, "settings")?;
        let req = store
            .get(&JsValue::from_str("app_settings"))
            .map_err(|e| StorageError::IoError(show_js_err(&e)))?;
        let val = await_val(&req).await?;
        if val.is_undefined() || val.is_null() {
            Ok(Settings::default())
        } else {
            from_json(&val)
        }
    }
}
