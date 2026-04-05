# BACKEND_STRUCTURE — Paper Immersive Reader

## 0. Purpose
This document defines the backend/service structure and data model for a **local-first, web-first** academic paper reading tool.

Key constraints from PRD:
- Local-first storage is the default and must work offline.
- Sync is user-managed.
- S3-compatible sync: user config endpoint/bucket/AK/SK, **sync everything**, **no extra encryption before upload**.
- Zotero: **metadata + PDF + annotations/notes** bi-directional sync.
- No sharing.

This implies a backend is **optional** for core reading, but **recommended** for security/operability.

## 1. System Topology

### 1.1 Components
1. **Web App (PWA)**
   - Owns: reading UX, local DB, import orchestration, annotation UX, export, offline mode.
   - Runs: PDF text extraction in Web Workers; may run limited OCR in-browser.

2. **Sync/Integration Gateway (optional but recommended)**
   - Owns: S3 credentials handling, S3 object operations, Zotero API calls, AI API key protection (if using a vendor LLM), heavy OCR jobs.
   - Exposes: HTTPS API for the web app.
   - Must be deployable by the user (self-host) to match “user-managed data”.

3. **External Services**
   - S3-compatible storage
   - Zotero Web API
   - AI model API (if enabled)

### 1.2 Why a Gateway is recommended (to eliminate ambiguity)
Even though requirements mention user AK/SK, a pure browser implementation has practical and security issues:
- Long-lived AK/SK in browser storage is risky.
- S3 CORS + credential exposure is messy.
- Large OCR/PDF processing may be too heavy in the browser.

**Decision:** Backend is a **Gateway** focused on integrations and async jobs, not the system of record.

## 2. Responsibilities by Layer

### 2.1 Web App Responsibilities
- Create and manage **Paper** objects locally.
- Persist:
  - parsed text blocks
  - reading progress
  - annotations
  - vocabulary
  - AI records (responses)
  - settings (tags, languages)
- Import pipeline orchestration:
  - PDF.js extraction
  - header/footer removal heuristics
  - paragraph segmentation
  - AI chunking call (if enabled)
  - decide OCR fallback
- Export generation (local): original + annotations + summaries + vocabulary
- Sync client:
  - diff local entities
  - upload/download bundles via Gateway
  - conflict presentation UI

### 2.2 Gateway Responsibilities
- S3 sync operations:
  - object upload/download/list
  - version metadata management
  - optional multipart upload
- Zotero bi-directional sync:
  - version-based pull (since library version)
  - push local annotations/notes
  - attachment handling (PDF)
  - mapping between local IDs and Zotero item keys
- AI proxy (recommended):
  - hides model API keys
  - enforces rate limits
  - prompt versioning
- OCR/Heavy processing jobs (recommended):
  - accept page images or PDF
  - run OCR pipeline
  - return extracted text + confidence + word boxes (optional)

## 3. Gateway Deployment Model

### 3.1 Modes
- **Mode A (Local-only)**: no gateway, no sync, no AI.
- **Mode B (Gateway enabled)**: user provides a gateway URL; web app uses it for:
  - S3 sync
  - Zotero sync
  - AI
  - OCR

### 3.2 Authentication
Since this is a personal tool (not multi-tenant by default):
- simplest: gateway uses a single **API token** (static) configured in the client.
- optional: device-level tokens per installation.

## 4. Data Model (Canonical Entities)

This section is the “source of truth” for what must be synced (FR-39).

### 4.1 Entity IDs
- Use **UUIDv7** (sortable) for client-generated IDs.
- Include `deviceId` for sync metadata.

### 4.2 Core Entities

#### Paper
- `paperId`
- `title`
- `sourceType`: pdf | paste | url | doi | zotero
- `sourceRef`: path/url/doi/zoteroKey
- `importedAt`
- `status`: CREATED | INGESTING | READY_TEXT | READY_OCR | FAILED
- `parseVersion`
- `language`: default `en`

#### PaperAsset
- `assetId`
- `paperId`
- `type`: pdf
- `contentHash` (SHA-256)
- `byteSize`
- `storageRef`:
  - local: indexeddb blob key or file-handle token
  - remote: s3 object key

#### ExtractedPage (optional but recommended)
Store raw extraction results to support re-processing and debugging.
- `paperId`
- `pageIndex`
- `extractionMethod`: native | ocr
- `rawItems` (if native): PDF.js text items + transforms
- `text`
- `qualityScore`
- `layoutHints`: multi-column detection results

#### Block
A reading unit used in immersive reading.
- `blockId`
- `paperId`
- `orderIndex`
- `text`
- `blockHash`
- `sourceSpan`:
  - page references (optional)
  - offsets into canonical text

#### ReadingProgress
- `paperId`
- `currentBlockIndex`
- `currentBlockId`
- `progressPercent`
- `lastUpdatedAt`

#### Annotation
Anchoring must survive re-segmentation and OCR errors.
- `annotationId`
- `paperId`
- `blockId` (hint)
- `type`: highlight | underline | note
- `tags`: string[]
- `quote`: exact text
- `prefix` / `suffix` (short context)
- `startOffset` / `endOffset` (best-effort)
- `createdAt`
- `updatedAt`
- `anchoringStatus`: anchored | unanchored

#### VocabularyItem
- `vocabId`
- `term`
- `paperIdFirstSeen`
- `blockIdFirstSeen`
- `lookupCount`
- `firstSeenAt`
- `lastSeenAt`
- `note` (optional)
- `learned`: boolean

#### AIRecord
- `aiRecordId`
- `paperId`
- `blockId` (optional)
- `type`: term_explain | sentence_explain | paragraph_summary | full_summary | structure | qa | notes | keywords
- `input` (selected text + context)
- `output` (structured content)
- `model` (string)
- `promptVersion`
- `createdAt`

#### TagDefinition
- `tagId`
- `name`
- `color` (optional)
- `isDefault` boolean

#### Settings
- `aiExplanationLanguage`
- `syncConfigProfileId`
- UI prefs (panel collapsed, shortcuts)

### 4.3 Sync Metadata Entity
Each synced entity must have:
- `entityId`
- `entityType`
- `updatedAt`
- `deletedAt` (tombstone)
- `deviceId`
- `contentHash`

## 5. S3-Compatible Sync Design

### 5.1 Storage Layout in Bucket
Object keys (suggested):
- `v1/manifests/<userId>/manifest.json`
- `v1/entities/<entityType>/<entityId>.json`
- `v1/assets/papers/<paperId>/<assetId>.pdf`

### 5.2 Manifest
A small index to allow incremental sync.
- lists entities and their hashes/updatedAt
- includes protocol version

### 5.3 Sync Algorithm (Client ↔ Gateway ↔ S3)
1. Client computes local change set since last sync.
2. Client uploads entity JSON blobs to Gateway.
3. Gateway writes to S3 and updates manifest.
4. Client pulls manifest, downloads missing/updated entities.
5. Client merges entities into local DB.

### 5.4 Conflict Rules (deterministic baseline)
- Progress: keep max(`updatedAt`).
- Annotation/Vocab/AIRecord: set union by ID.
- Settings: last-write-wins by `updatedAt`.

If same entity ID changed on two devices:
- keep both versions by generating a conflict copy (preferred) OR
- prompt the user.

## 6. Zotero Bi-directional Sync

### 6.1 Canonical Mapping
Maintain a mapping table in the gateway DB:
- `paperId` ↔ `zoteroItemKey`
- `annotationId` ↔ `zoteroAnnotationKey` (if using Zotero PDF annotation items)
- `assetId` ↔ `zoteroAttachmentKey`

### 6.2 Pull Flow
- Use Zotero library version (Last-Modified-Version).
- Fetch changed items since last version.
- Download PDFs on-demand or per user choice.

### 6.3 Push Flow
- Convert local annotations/notes to Zotero annotation item schema.
- Use optimistic concurrency via Zotero item versions.

### 6.4 Conflicts
- Metadata: Zotero is authoritative.
- Our reading progress: local is authoritative.
- Annotations/notes: merge union; flag duplicates.

## 7. OCR & PDF Processing Backend Jobs

### 7.1 Job Types
- `pdf_extract_text`
- `ocr_page`
- `ocr_document`
- `chunk_blocks_ai`
- `sync_s3_upload`
- `sync_s3_download`
- `sync_zotero_pull`
- `sync_zotero_push`

### 7.2 Job State
- `PENDING` → `RUNNING` → (`SUCCEEDED` | `FAILED`)
- retry policy with exponential backoff

### 7.3 OCR Output Schema
Return:
- extracted text
- confidence
- optional word boxes

Store OCR engine version in outputs for reproducibility.

## 8. Gateway API (HTTP)

This API is intentionally small and stable.

### 8.1 Health
- `GET /health`

### 8.2 Sync (S3)
- `POST /sync/push`
  - input: batch of entities + assets metadata
  - output: applied versions + errors
- `POST /sync/pull`
  - input: lastSyncCursor
  - output: changed entities list + signed download URLs

### 8.3 Assets
- `POST /assets/upload/init` (optional multipart)
- `POST /assets/upload/complete`
- `GET /assets/download?assetId=...`

### 8.4 Zotero
- `POST /zotero/connect`
- `POST /zotero/pull`
- `POST /zotero/push`

### 8.5 AI
- `POST /ai/explain-term`
- `POST /ai/explain-sentence`
- `POST /ai/summary`
- `POST /ai/structure`
- `POST /ai/qa`

### 8.6 OCR
- `POST /ocr/document`
- `POST /ocr/page`

## 9. Gateway Internal Storage

### 9.1 Database (Gateway)
Gateway DB stores:
- connector configs (S3, Zotero)
- mapping tables
- job records
- audit logs

It should **not** be treated as the canonical store of reading data.

### 9.2 Secrets Handling
- Store S3 AK/SK and Zotero API key in gateway-side secrets store.
- Never return raw secrets to the browser.

## 10. Observability
- Structured logs per request + per job.
- Import/OCR metrics:
  - OCR fallback rate
  - average OCR time per page
- Sync metrics:
  - entities pushed/pulled
  - conflict count

## 11. Edge Cases to Encode in Implementation
- Multi-column PDF reading order issues: store raw PDF.js text items.
- Garbled fonts/unicode errors: trigger OCR.
- Unanchored annotations: display separately and support re-anchoring.
- Very large scanned PDFs: progressive OCR (page-on-demand).
- Zotero rate limits: respect Retry-After; batch requests.
