// Mirrors Rust domain types from crates/core/src/domain/

export interface PaperId {
  0: string;
}

export type PaperStatus = "Created" | "Ingesting" | "ReadyText" | "ReadyOcr" | "Failed";
export type PaperSourceType = "Pdf" | "Paste" | "Url" | "Doi" | "Zotero";

export interface Paper {
  id: PaperId;
  title: string | null;
  source_type: PaperSourceType;
  source_ref: string | null;
  status: PaperStatus;
  parse_version: number;
}

export interface StoredPaper {
  paper: Paper;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface BlockId {
  0: string;
}

export interface Block {
  id: BlockId;
  paper_id: PaperId;
  order_index: number;
  text: string;
  block_hash: string | null;
}

export type AnnotationType = "Highlight" | "Underline" | "Note";
export type AnchoringStatus = "Anchored" | "Unanchored";

export interface Annotation {
  id: string;
  paper_id: PaperId;
  block_id: BlockId;
  annotation_type: AnnotationType;
  quote: string;
  prefix: string | null;
  suffix: string | null;
  start_offset: number | null;
  end_offset: number | null;
  note: string | null;
  tags: string[];
  anchoring_status: AnchoringStatus;
}

export interface VocabularyItem {
  id: string;
  term: string;
  term_lower: string;
  paper_id_first_seen: PaperId;
  block_id_first_seen: BlockId;
  lookup_count: number;
  first_seen_at_ms: number;
  last_seen_at_ms: number;
  note: string | null;
  learned: boolean;
}

export interface ReadingProgress {
  paper_id: PaperId;
  current_block_index: number;
  current_block_id: BlockId | null;
  updated_at_ms: number;
}

export interface AiRecord {
  id: string;
  paper_id: PaperId;
  block_id: BlockId;
  record_type: string;
  input: string;
  output: string;
  language: string;
  created_at_ms: number;
}

export interface ReadingStats {
  paper_id: PaperId;
  total_reading_ms: number;
  session_started_ms: number | null;
  session_count: number;
}

export interface Settings {
  explanation_language: string;
  custom_tags: string[];
  ai_gateway_url: string | null;
  ai_gateway_token_ref: string | null;
  sidebar_collapsed: boolean;
}

export const DEFAULT_SETTINGS: Settings = {
  explanation_language: "en",
  custom_tags: [],
  ai_gateway_url: null,
  ai_gateway_token_ref: null,
  sidebar_collapsed: false,
};

export const DEFAULT_TAGS = ["重点", "疑问", "术语", "结论", "复习"];
