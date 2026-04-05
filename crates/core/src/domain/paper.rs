use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperStatus {
    Created,
    Ingesting,
    ReadyText,
    ReadyOcr,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperSourceType {
    Pdf,
    Paste,
    Url,
    Doi,
    Zotero,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paper {
    pub id: PaperId,
    pub title: Option<String>,
    pub source_type: PaperSourceType,
    pub source_ref: Option<String>,
    pub status: PaperStatus,
    pub parse_version: u32,
}
