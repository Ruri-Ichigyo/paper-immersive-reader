use js_sys::JSON;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{DomStringList, IdbDatabase, IdbRequest, IdbTransactionMode};

use paper_immersive_reader_core::domain::{
    Annotation, AnnotationId, Block, PaperId, ReadingProgress, VocabularyItem,
};

use super::schema::{AiRecord, Settings, StoredPaper};
use super::StorageError;

const DB_NAME: &str = "paper-immersive-reader";
const DB_VERSION: u32 = 1;

fn to_json<T: serde::Serialize>(v: &T) -> Result<JsValue, StorageError> {
    let json = serde_json::to_string(v).map_err(|e| StorageError::IoError(e.to_string()))?;
    Ok(JsValue::from_str(&json))
}

fn from_json<T: serde::de::DeserializeOwned>(val: &JsValue) -> Result<T, StorageError> {
    if val.is_undefined() || val.is_null() {
        return Err(StorageError::NotFound);
    }
    let json = JSON::stringify(val)
        .ok()
        .and_then(|j| j.as_string())
        .unwrap_or_default();
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
    let arr: js_sys::Array = val.clone().unchecked_into();
    let mut out = Vec::with_capacity(arr.length() as usize);
    for i in 0..arr.length() {
        if let Ok(item) = from_json(&arr.get(i)) {
            out.push(item);
        }
    }
    Ok(out)
}

/// Wrap an IdbRequest into a JS promise and await it, returning raw JsValue.
async fn await_val(req: &IdbRequest) -> Result<JsValue, StorageError> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let on_ok = Closure::wrap(Box::new(move || {
            let _ = resolve.call0(&JsValue::null());
        }) as Box<dyn FnMut()>);
        let on_err = Closure::wrap(Box::new(move || {
            let _ = reject.call0(&JsValue::null());
        }) as Box<dyn FnMut()>);
        req.set_onsuccess(Some(on_ok.as_ref().unchecked_ref()));
        req.set_onerror(Some(on_err.as_ref().unchecked_ref()));
        on_ok.forget();
        on_err.forget();
    });
    let _ = JsFuture::from(promise)
        .await
        .map_err(|e| StorageError::IoError(format!("IndexedDB request failed: {:?}", e)))?;
    req.result()
        .map_err(|e| StorageError::IoError(format!("IndexedDB result error: {:?}", e)))
}

fn create_stores(db: &IdbDatabase) {
    let names: DomStringList = db.object_store_names();

    if !names.contains("papers") {
        db.create_object_store("papers")
            .expect("create papers store");
    }
    if !names.contains("blocks") {
        let store = db
            .create_object_store("blocks")
            .expect("create blocks store");
        store
            .create_index_with_str("paper_id", "paper_id")
            .expect("create blocks index");
    }
    if !names.contains("progress") {
        db.create_object_store("progress")
            .expect("create progress store");
    }
    if !names.contains("annotations") {
        let store = db
            .create_object_store("annotations")
            .expect("create annotations store");
        store
            .create_index_with_str("paper_id", "paper_id")
            .expect("create annotations index");
    }
    if !names.contains("vocabulary") {
        let store = db
            .create_object_store("vocabulary")
            .expect("create vocabulary store");
        store
            .create_index_with_str("term_lower", "term_lower")
            .expect("create vocabulary index");
    }
    if !names.contains("ai_records") {
        let store = db
            .create_object_store("ai_records")
            .expect("create ai_records store");
        store
            .create_index_with_str("paper_id", "paper_id")
            .expect("create ai_records index");
    }
    if !names.contains("settings") {
        db.create_object_store("settings")
            .expect("create settings store");
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
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))
            .and_then(|factory| {
                if factory.is_none() {
                    Err(StorageError::IoError("indexedDB unavailable".into()))
                } else {
                    Ok(factory.unwrap())
                }
            })?;

        let request = factory
            .open_with_u32(DB_NAME, DB_VERSION)
            .map_err(|e| StorageError::IoError(format!("open failed: {:?}", e)))?;

        let on_upgrade = Closure::wrap(Box::new(move |event: web_sys::IdbVersionChangeEvent| {
            let db: Option<IdbDatabase> = event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::IdbOpenDbRequest>().ok())
                .and_then(|r| r.result().ok())
                .and_then(|r| r.dyn_into::<IdbDatabase>().ok());
            if let Some(db) = db {
                create_stores(&db);
            }
        }) as Box<dyn FnMut(_)>);
        request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
        on_upgrade.forget();

        let db = await_idb_db(request.as_ref()).await?;
        Ok(Self { db })
    }

    // Papers
    pub async fn save_paper(&self, paper: &StoredPaper) -> Result<(), StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("papers")),
                IdbTransactionMode::Readwrite,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("papers")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = to_json(paper)?;
        let req = store
            .put_with_key(&val, &JsValue::from_str(&paper.paper.id.0))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        await_val(&req).await?;
        Ok(())
    }

    pub async fn load_paper(&self, id: &PaperId) -> Result<Option<StoredPaper>, StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("papers")),
                IdbTransactionMode::Readonly,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("papers")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let req = store
            .get(&JsValue::from_str(&id.0))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = await_val(&req).await?;
        from_json_opt(&val)
    }

    pub async fn list_papers(&self) -> Result<Vec<StoredPaper>, StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("papers")),
                IdbTransactionMode::Readonly,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("papers")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let req = store
            .get_all()
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = await_val(&req).await?;
        arr_from_json(&val)
    }

    // Blocks
    pub async fn save_blocks(&self, blocks: &[Block]) -> Result<(), StorageError> {
        if blocks.is_empty() {
            return Ok(());
        }
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("blocks")),
                IdbTransactionMode::Readwrite,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("blocks")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        for block in blocks {
            let val = to_json(block)?;
            let req = store
                .put_with_key(&val, &JsValue::from_str(&block.id.0))
                .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
            await_val(&req).await?;
        }
        Ok(())
    }

    pub async fn load_blocks(&self, paper_id: &PaperId) -> Result<Vec<Block>, StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("blocks")),
                IdbTransactionMode::Readonly,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("blocks")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let idx = store
            .index("paper_id")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let range = web_sys::IdbKeyRange::only(&JsValue::from_str(&paper_id.0))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let req = idx
            .get_all_with_key(&range)
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = await_val(&req).await?;
        let mut blocks: Vec<Block> = arr_from_json(&val)?;
        blocks.sort_by_key(|b| b.order_index);
        Ok(blocks)
    }

    // Progress
    pub async fn save_progress(&self, progress: &ReadingProgress) -> Result<(), StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("progress")),
                IdbTransactionMode::Readwrite,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("progress")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = to_json(progress)?;
        let req = store
            .put_with_key(&val, &JsValue::from_str(&progress.paper_id.0))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        await_val(&req).await?;
        Ok(())
    }

    pub async fn load_progress(
        &self,
        paper_id: &PaperId,
    ) -> Result<Option<ReadingProgress>, StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("progress")),
                IdbTransactionMode::Readonly,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("progress")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let req = store
            .get(&JsValue::from_str(&paper_id.0))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = await_val(&req).await?;
        from_json_opt(&val)
    }

    // Annotations
    pub async fn save_annotation(&self, annotation: &Annotation) -> Result<(), StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("annotations")),
                IdbTransactionMode::Readwrite,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("annotations")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = to_json(annotation)?;
        let req = store
            .put_with_key(&val, &JsValue::from_str(&annotation.id.0))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        await_val(&req).await?;
        Ok(())
    }

    pub async fn load_annotations(
        &self,
        paper_id: &PaperId,
    ) -> Result<Vec<Annotation>, StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("annotations")),
                IdbTransactionMode::Readonly,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("annotations")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let idx = store
            .index("paper_id")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let range = web_sys::IdbKeyRange::only(&JsValue::from_str(&paper_id.0))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let req = idx
            .get_all_with_key(&range)
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = await_val(&req).await?;
        arr_from_json(&val)
    }

    pub async fn load_annotation(
        &self,
        id: &AnnotationId,
    ) -> Result<Option<Annotation>, StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("annotations")),
                IdbTransactionMode::Readonly,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("annotations")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let req = store
            .get(&JsValue::from_str(&id.0))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = await_val(&req).await?;
        from_json_opt(&val)
    }

    // Vocabulary
    pub async fn upsert_vocabulary(&self, item: &VocabularyItem) -> Result<(), StorageError> {
        let term_lower = item.term.to_lowercase();
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("vocabulary")),
                IdbTransactionMode::Readwrite,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("vocabulary")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let idx = store
            .index("term_lower")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let range = web_sys::IdbKeyRange::only(&JsValue::from_str(&term_lower))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let req = idx
            .get(&range)
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = await_val(&req).await?;

        if val.is_undefined() || val.is_null() {
            let mut new_item = item.clone();
            new_item.term = term_lower;
            let jv = to_json(&new_item)?;
            let req = store
                .put_with_key(&jv, &JsValue::from_str(&item.id.0))
                .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
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
                .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
            await_val(&req).await?;
        }
        Ok(())
    }

    pub async fn list_vocabulary(&self) -> Result<Vec<VocabularyItem>, StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("vocabulary")),
                IdbTransactionMode::Readonly,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("vocabulary")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let req = store
            .get_all()
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = await_val(&req).await?;
        arr_from_json(&val)
    }

    // AI records
    pub async fn save_ai_record(&self, record: &AiRecord) -> Result<(), StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("ai_records")),
                IdbTransactionMode::Readwrite,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("ai_records")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = to_json(record)?;
        let req = store
            .put_with_key(&val, &JsValue::from_str(&record.id))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        await_val(&req).await?;
        Ok(())
    }

    pub async fn load_ai_records(&self, paper_id: &PaperId) -> Result<Vec<AiRecord>, StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("ai_records")),
                IdbTransactionMode::Readonly,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("ai_records")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let idx = store
            .index("paper_id")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let range = web_sys::IdbKeyRange::only(&JsValue::from_str(&paper_id.0))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let req = idx
            .get_all_with_key(&range)
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = await_val(&req).await?;
        arr_from_json(&val)
    }

    // Settings
    pub async fn save_settings(&self, settings: &Settings) -> Result<(), StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("settings")),
                IdbTransactionMode::Readwrite,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("settings")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = to_json(settings)?;
        let req = store
            .put_with_key(&val, &JsValue::from_str("app_settings"))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        await_val(&req).await?;
        Ok(())
    }

    pub async fn load_settings(&self) -> Result<Settings, StorageError> {
        let txn = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of1(&JsValue::from_str("settings")),
                IdbTransactionMode::Readonly,
            )
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let store = txn
            .object_store("settings")
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let req = store
            .get(&JsValue::from_str("app_settings"))
            .map_err(|e| StorageError::IoError(format!("{:?}", e)))?;
        let val = await_val(&req).await?;
        if val.is_undefined() || val.is_null() {
            Ok(Settings::default())
        } else {
            from_json(&val)
        }
    }
}

async fn await_idb_db(req: &IdbRequest) -> Result<IdbDatabase, StorageError> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let req_c = req.clone();
        let on_ok = Closure::wrap(Box::new(move || {
            let _ = resolve.call0(&JsValue::null());
        }) as Box<dyn FnMut()>);
        let on_err = Closure::wrap(Box::new(move || {
            let _ = reject.call0(&JsValue::null());
        }) as Box<dyn FnMut()>);
        req_c.set_onsuccess(Some(on_ok.as_ref().unchecked_ref()));
        req_c.set_onerror(Some(on_err.as_ref().unchecked_ref()));
        on_ok.forget();
        on_err.forget();
    });
    let _ = JsFuture::from(promise)
        .await
        .map_err(|e| StorageError::IoError(format!("IDB open failed: {:?}", e)))?;
    let db = req
        .result()
        .map_err(|e| StorageError::IoError(format!("IDB result: {:?}", e)))?;
    Ok(db.unchecked_into())
}
