import { openDB, type IDBPDatabase } from "idb";
import type {
  AiRecord,
  Annotation,
  Block,
  Paper,
  ReadingProgress,
  ReadingStats,
  Settings,
  StoredPaper,
  VocabularyItem,
} from "../types";

const DB_NAME = "paper-immersive-reader";
const DB_VERSION = 4;

// Row shapes stored in IndexedDB. `key` is the primary key (keyPath).
// Each row also carries the flattened domain fields.

interface PaperRow {
  key: string;
  paper: Paper;
  created_at_ms: number;
  updated_at_ms: number;
}

interface BlockRow {
  key: string;
  paper_id: Paper["id"];
  paper_id_key: string;
  order_index: number;
  text: string;
  block_hash: string | null;
}

interface ProgressRow {
  key: string;
  paper_id: Paper["id"];
  current_block_index: number;
  current_block_id: Block["id"] | null;
  updated_at_ms: number;
}

interface AnnotationRow {
  key: string;
  paper_id: Paper["id"];
  paper_id_key: string;
  block_id: Block["id"];
  annotation_type: Annotation["annotation_type"];
  quote: string;
  prefix: string | null;
  suffix: string | null;
  start_offset: number | null;
  end_offset: number | null;
  note: string | null;
  tags: string[];
  anchoring_status: Annotation["anchoring_status"];
}

interface VocabularyRow {
  key: string;
  term_lower_key: string;
  term: string;
  term_lower: string;
  paper_id_first_seen: Paper["id"];
  block_id_first_seen: Block["id"];
  lookup_count: number;
  first_seen_at_ms: number;
  last_seen_at_ms: number;
  note: string | null;
  learned: boolean;
}

interface AiRecordRow {
  key: string;
  paper_id: Paper["id"];
  paper_id_key: string;
  block_id: Block["id"];
  record_type: string;
  input: string;
  output: string;
  language: string;
  created_at_ms: number;
}

interface SettingsRow {
  key: string;
  explanation_language: string;
  custom_tags: string[];
  ai_gateway_url: string | null;
  ai_gateway_token_ref: string | null;
  sidebar_collapsed: boolean;
}

interface ReadingStatsRow {
  key: string;
  paper_id: Paper["id"];
  total_reading_ms: number;
  session_started_ms: number | null;
  session_count: number;
}

let dbPromise: Promise<IDBPDatabase> | null = null;

export function getDB(): Promise<IDBPDatabase> {
  if (!dbPromise) {
    dbPromise = openDB(DB_NAME, DB_VERSION, {
      upgrade(db, _oldVersion, _newVersion, _tx) {
        // Delete all existing stores to ensure correct schema
        const storeNames = Array.from(db.objectStoreNames);
        for (const name of storeNames) {
          db.deleteObjectStore(name);
        }
        // Create stores with correct keyPath
        db.createObjectStore("papers", { keyPath: "key" });
        const blocksStore = db.createObjectStore("blocks", { keyPath: "key" });
        blocksStore.createIndex("paper_id", "paper_id_key");
        db.createObjectStore("progress", { keyPath: "key" });
        const annotationsStore = db.createObjectStore("annotations", { keyPath: "key" });
        annotationsStore.createIndex("paper_id", "paper_id_key");
        const vocabularyStore = db.createObjectStore("vocabulary", { keyPath: "key" });
        vocabularyStore.createIndex("term_lower", "term_lower_key");
        const aiRecordsStore = db.createObjectStore("ai_records", { keyPath: "key" });
        aiRecordsStore.createIndex("paper_id", "paper_id_key");
        db.createObjectStore("settings", { keyPath: "key" });
        db.createObjectStore("reading_stats", { keyPath: "key" });
      },
    });
  }
  return dbPromise;
}

// --- Papers ---

export async function savePaper(paper: Paper, createdAt: number, updatedAt: number): Promise<void> {
  const db = await getDB();
  const row: PaperRow = {
    key: paper.id[0],
    paper,
    created_at_ms: createdAt,
    updated_at_ms: updatedAt,
  };
  await db.put("papers", row);
}

export async function listPapers(): Promise<StoredPaper[]> {
  const db = await getDB();
  const rows = (await db.getAll("papers")) as PaperRow[];
  return rows
    .map(({ paper, created_at_ms, updated_at_ms }) => ({ paper, created_at_ms, updated_at_ms }))
    .sort((a, b) => b.updated_at_ms - a.updated_at_ms);
}

export async function getPaper(id: string): Promise<Paper | undefined> {
  const db = await getDB();
  const row = (await db.get("papers", id)) as PaperRow | undefined;
  return row?.paper;
}

export async function deletePaper(id: string): Promise<void> {
  const db = await getDB();
  await db.delete("papers", id);
  await db.delete("progress", id);
  await db.delete("reading_stats", id);

  const deleteByPaper = async (storeName: "blocks" | "annotations" | "ai_records") => {
    const tx = db.transaction(storeName, "readwrite");
    const index = tx.store.index("paper_id");
    for await (const cursor of index.iterate(id)) {
      await tx.store.delete(cursor.value.key);
    }
    await tx.done;
  };
  await deleteByPaper("blocks");
  await deleteByPaper("annotations");
  await deleteByPaper("ai_records");
}

// --- Blocks ---

export async function saveBlock(block: Block): Promise<void> {
  const db = await getDB();
  const row: BlockRow = {
    key: block.id[0],
    paper_id: block.paper_id,
    paper_id_key: block.paper_id[0],
    order_index: block.order_index,
    text: block.text,
    block_hash: block.block_hash,
  };
  await db.put("blocks", row);
}

export async function saveBlocks(blocks: Block[]): Promise<void> {
  const db = await getDB();
  const tx = db.transaction("blocks", "readwrite");
  for (const block of blocks) {
    const row: BlockRow = {
      key: block.id[0],
      paper_id: block.paper_id,
      paper_id_key: block.paper_id[0],
      order_index: block.order_index,
      text: block.text,
      block_hash: block.block_hash,
    };
    await tx.store.put(row);
  }
  await tx.done;
}

export async function loadBlocks(paperId: string): Promise<Block[]> {
  const db = await getDB();
  const rows = (await db.getAllFromIndex("blocks", "paper_id", paperId)) as BlockRow[];
  return rows
    .map((r): Block => ({
      id: { 0: r.key },
      paper_id: r.paper_id,
      order_index: r.order_index,
      text: r.text,
      block_hash: r.block_hash,
    }))
    .sort((a, b) => a.order_index - b.order_index);
}

// --- Progress ---

export async function saveProgress(progress: ReadingProgress): Promise<void> {
  const db = await getDB();
  const row: ProgressRow = {
    key: progress.paper_id[0],
    paper_id: progress.paper_id,
    current_block_index: progress.current_block_index,
    current_block_id: progress.current_block_id,
    updated_at_ms: progress.updated_at_ms,
  };
  await db.put("progress", row);
}

export async function loadProgress(paperId: string): Promise<ReadingProgress | undefined> {
  const db = await getDB();
  const row = (await db.get("progress", paperId)) as ProgressRow | undefined;
  if (!row) return undefined;
  return {
    paper_id: row.paper_id,
    current_block_index: row.current_block_index,
    current_block_id: row.current_block_id,
    updated_at_ms: row.updated_at_ms,
  };
}

// --- Annotations ---

export async function saveAnnotation(ann: Annotation): Promise<void> {
  const db = await getDB();
  const row: AnnotationRow = {
    key: ann.id,
    paper_id: ann.paper_id,
    paper_id_key: ann.paper_id[0],
    block_id: ann.block_id,
    annotation_type: ann.annotation_type,
    quote: ann.quote,
    prefix: ann.prefix,
    suffix: ann.suffix,
    start_offset: ann.start_offset,
    end_offset: ann.end_offset,
    note: ann.note,
    tags: ann.tags,
    anchoring_status: ann.anchoring_status,
  };
  await db.put("annotations", row);
}

function annotationFromRow(r: AnnotationRow): Annotation {
  return {
    id: r.key,
    paper_id: r.paper_id,
    block_id: r.block_id,
    annotation_type: r.annotation_type,
    quote: r.quote,
    prefix: r.prefix,
    suffix: r.suffix,
    start_offset: r.start_offset,
    end_offset: r.end_offset,
    note: r.note,
    tags: r.tags,
    anchoring_status: r.anchoring_status,
  };
}

export async function loadAnnotations(paperId: string): Promise<Annotation[]> {
  const db = await getDB();
  const rows = (await db.getAllFromIndex("annotations", "paper_id", paperId)) as AnnotationRow[];
  return rows.map(annotationFromRow);
}

export async function loadAllAnnotations(): Promise<Annotation[]> {
  const db = await getDB();
  const rows = (await db.getAll("annotations")) as AnnotationRow[];
  return rows.map(annotationFromRow);
}

export async function deleteAnnotation(id: string): Promise<void> {
  const db = await getDB();
  await db.delete("annotations", id);
}

export async function updateAnnotation(ann: Annotation): Promise<void> {
  await saveAnnotation(ann);
}

// --- Vocabulary ---

export async function upsertVocabulary(item: VocabularyItem): Promise<void> {
  const db = await getDB();
  const existing = (await db.getAllFromIndex(
    "vocabulary",
    "term_lower",
    item.term_lower,
  )) as VocabularyRow[];
  if (existing.length > 0) {
    const found = existing[0]!;
    const row: VocabularyRow = {
      ...found,
      lookup_count: found.lookup_count + 1,
      last_seen_at_ms: Date.now(),
    };
    await db.put("vocabulary", row);
  } else {
    const row: VocabularyRow = {
      key: item.id,
      term_lower_key: item.term_lower,
      term: item.term,
      term_lower: item.term_lower,
      paper_id_first_seen: item.paper_id_first_seen,
      block_id_first_seen: item.block_id_first_seen,
      lookup_count: item.lookup_count,
      first_seen_at_ms: item.first_seen_at_ms,
      last_seen_at_ms: item.last_seen_at_ms,
      note: item.note,
      learned: item.learned,
    };
    await db.put("vocabulary", row);
  }
}

function vocabularyFromRow(r: VocabularyRow): VocabularyItem {
  return {
    id: r.key,
    term: r.term,
    term_lower: r.term_lower,
    paper_id_first_seen: r.paper_id_first_seen,
    block_id_first_seen: r.block_id_first_seen,
    lookup_count: r.lookup_count,
    first_seen_at_ms: r.first_seen_at_ms,
    last_seen_at_ms: r.last_seen_at_ms,
    note: r.note,
    learned: r.learned,
  };
}

export async function listVocabulary(): Promise<VocabularyItem[]> {
  const db = await getDB();
  const rows = (await db.getAll("vocabulary")) as VocabularyRow[];
  return rows
    .map(vocabularyFromRow)
    .sort((a, b) => b.last_seen_at_ms - a.last_seen_at_ms);
}

export async function updateVocabulary(item: VocabularyItem): Promise<void> {
  const db = await getDB();
  const row: VocabularyRow = {
    key: item.id,
    term_lower_key: item.term_lower,
    term: item.term,
    term_lower: item.term_lower,
    paper_id_first_seen: item.paper_id_first_seen,
    block_id_first_seen: item.block_id_first_seen,
    lookup_count: item.lookup_count,
    first_seen_at_ms: item.first_seen_at_ms,
    last_seen_at_ms: item.last_seen_at_ms,
    note: item.note,
    learned: item.learned,
  };
  await db.put("vocabulary", row);
}

export async function deleteVocabulary(id: string): Promise<void> {
  const db = await getDB();
  await db.delete("vocabulary", id);
}

// --- AI Records ---

export async function saveAiRecord(rec: AiRecord): Promise<void> {
  const db = await getDB();
  const row: AiRecordRow = {
    key: rec.id,
    paper_id: rec.paper_id,
    paper_id_key: rec.paper_id[0],
    block_id: rec.block_id,
    record_type: rec.record_type,
    input: rec.input,
    output: rec.output,
    language: rec.language,
    created_at_ms: rec.created_at_ms,
  };
  await db.put("ai_records", row);
}

// --- Settings ---

export async function saveSettings(settings: Settings): Promise<void> {
  const db = await getDB();
  const row: SettingsRow = {
    key: "default",
    explanation_language: settings.explanation_language,
    custom_tags: settings.custom_tags,
    ai_gateway_url: settings.ai_gateway_url,
    ai_gateway_token_ref: settings.ai_gateway_token_ref,
    sidebar_collapsed: settings.sidebar_collapsed,
  };
  await db.put("settings", row);
}

export async function loadSettings(): Promise<Settings | undefined> {
  const db = await getDB();
  const row = (await db.get("settings", "default")) as SettingsRow | undefined;
  if (!row) return undefined;
  return {
    explanation_language: row.explanation_language,
    custom_tags: row.custom_tags,
    ai_gateway_url: row.ai_gateway_url,
    ai_gateway_token_ref: row.ai_gateway_token_ref,
    sidebar_collapsed: row.sidebar_collapsed,
  };
}

// --- Reading Stats ---

export async function saveReadingStats(stats: ReadingStats): Promise<void> {
  const db = await getDB();
  const row: ReadingStatsRow = {
    key: stats.paper_id[0],
    paper_id: stats.paper_id,
    total_reading_ms: stats.total_reading_ms,
    session_started_ms: stats.session_started_ms,
    session_count: stats.session_count,
  };
  await db.put("reading_stats", row);
}

export async function loadReadingStats(paperId: string): Promise<ReadingStats | undefined> {
  const db = await getDB();
  const row = (await db.get("reading_stats", paperId)) as ReadingStatsRow | undefined;
  if (!row) return undefined;
  return {
    paper_id: row.paper_id,
    total_reading_ms: row.total_reading_ms,
    session_started_ms: row.session_started_ms,
    session_count: row.session_count,
  };
}