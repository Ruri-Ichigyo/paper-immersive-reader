# BACKEND_STRUCTURE - Paper Immersive Reader

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
1. **Web App (Leptos CSR)**
   - Owns: reading UX, local DB (IndexedDB), import orchestration, annotation UX, export, offline mode.
   - Runs: PDF text extraction via pdf.js in the browser.
   - Compiled to WASM (`wasm32-unknown-unknown`) via Trunk.

2. **AI Gateway (optional but recommended)**
   - Owns: AI API key protection, translation/explanation proxying.
   - Future: S3 credentials handling, Zotero API calls, heavy OCR jobs.
   - Built with **Axum** (Rust), exposes HTTPS API for the web app.
   - Must be deployable by the user (self-host) to match "user-managed data".

3. **External Services**
   - AI model API (OpenAI-compatible, if enabled)
   - S3-compatible storage (Phase 5+)
   - Zotero Web API (Phase 6+)

### 1.2 Why a Gateway is recommended
Even though requirements mention user AK/SK, a pure browser implementation has practical and security issues:
- Long-lived AK/SK in browser storage is risky.
- S3 CORS + credential exposure is messy.
- Large OCR/PDF processing may be too heavy in the browser.

**Decision:** Backend is a **Gateway** focused on integrations and async jobs, not the system of record.

## 2. Responsibilities by Layer

### 2.1 Web App Responsibilities
- Create and manage **Paper** objects locally.
- Persist in IndexedDB:
  - parsed text blocks
  - reading progress
  - annotations
  - vocabulary
  - AI records (responses)
  - settings (tags, languages, gateway URL/token)
  - reading stats (time tracking)
- Import pipeline orchestration:
  - pdf.js text extraction
  - header/footer removal heuristics
  - paragraph segmentation
  - (future) AI chunking call
  - (future) OCR fallback
- Export generation (Phase 2+): original + annotations + summaries + vocabulary
- (future) Sync client: diff local entities, upload/download bundles via Gateway

### 2.2 Gateway Responsibilities
- **AI proxy (implemented):**
  - hides model API keys
  - enforces optional token authentication
  - structured prompt management
- **S3 sync operations (Phase 5+ - not implemented):**
  - object upload/download/list
  - version metadata management
- **Zotero bi-directional sync (Phase 6+ - not implemented):**
  - version-based pull
  - push local annotations/notes
- **OCR/Heavy processing jobs (Phase 4+ - not implemented in web):**
  - CLI-side OCR via PaddleOCR exists
  - Web-side OCR integration deferred

## 3. Gateway Deployment Model

### 3.1 Modes
- **Mode A (Local-only)**: no gateway, no AI. All reading, annotation, and vocabulary features work offline.
- **Mode B (Gateway enabled)**: user provides a gateway URL in Settings; web app uses it for AI translation/explanation.

### 3.2 Authentication
Since this is a personal tool (not multi-tenant by default):
- Gateway uses a single **static token** (`GATEWAY_TOKEN` env var).
- Web app sends `Authorization: Bearer <token>` header when token is configured in Settings.
- If `GATEWAY_TOKEN` is not set, the gateway accepts all requests without authentication.
- Implemented as `require_token` axum middleware on all `/api/*` routes.

## 4. Data Model (Canonical Entities)

This section is the "source of truth" for what must be synced (FR-39).

### 4.1 Entity IDs
- UUID v4 short segments (e.g., `paper-a1b2c3d4`, `block-e5f6g7h8`)
- Generated via `uuid::Uuid::new_v4()` with prefix

### 4.2 Core Entities

#### Paper
- `paperId` (PaperId)
- `title`: Option<String>
- `sourceType`: Pdf | Paste | Url | Doi | Zotero
- `sourceRef`: Option<String>
- `status`: Created | Ingesting | ReadyText | ReadyOcr | Failed
- `parseVersion`: u32

#### StoredPaper (web-app layer)
- `paper`: Paper
- `created_at_ms`: u64
- `updated_at_ms`: u64

#### Block
- `blockId` (BlockId)
- `paperId` (PaperId)
- `orderIndex`: u32
- `text`: String

#### ReadingProgress
- `paperId` (PaperId)
- `currentBlockIndex`: u32
- `currentBlockId`: Option<BlockId>
- `updated_at_ms`: u64

#### Annotation
- `annotationId` (AnnotationId)
- `paperId` (PaperId)
- `blockId` (BlockId)
- `annotationType`: Highlight | Underline | Note
- `quote`: String (exact text)
- `prefix` / `suffix`: Option<String> (up to 50 chars context)
- `startOffset` / `endOffset`: Option<u32>
- `note`: Option<String>
- `tags`: Vec<String>
- `anchoringStatus`: Anchored | Unanchored

#### VocabularyItem
- `vocabId` (VocabularyId)
- `term`: String (original casing preserved)
- `term_lower`: String (lowercase index key)
- `paperIdFirstSeen` (PaperId)
- `blockIdFirstSeen` (BlockId)
- `lookupCount`: u32
- `firstSeenAtMs` / `lastSeenAtMs`: u64
- `note`: Option<String>
- `learned`: bool

#### AiRecord
- `id`: String
- `paperId` (PaperId)
- `blockId` (BlockId)
- `recordType`: String ("translate", "explain_term", "explain_sentence", "translate_block")
- `input`: String
- `output`: String
- `language`: String
- `created_at_ms`: u64

#### ReadingStats
- `paperId` (PaperId)
- `total_reading_ms`: u64
- `session_started_ms`: Option<u64>
- `session_count`: u32

#### Settings
- `explanation_language`: String ("en", "zh", "ja", "ko")
- `custom_tags`: Vec<String>
- `ai_gateway_url`: Option<String>
- `ai_gateway_token_ref`: Option<String>

### 4.3 Sync Metadata Entity (Phase 5+ - not yet implemented)
Each synced entity must have:
- `entityId`
- `entityType`
- `updatedAt`
- `deletedAt` (tombstone)
- `deviceId`
- `contentHash`

## 5. S3-Compatible Sync Design (Phase 5+ - not yet implemented)

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

## 6. Zotero Bi-directional Sync (Phase 6+ - not yet implemented)

### 6.1 Canonical Mapping
Maintain a mapping table in the gateway DB:
- `paperId` ↔ `zoteroItemKey`
- `annotationId` ↔ `zoteroAnnotationKey`
- `assetId` ↔ `zoteroAttachmentKey`

### 6.2 Pull Flow
- Use Zotero library version (Last-Modified-Version).
- Fetch changed items since last version.
- Download PDFs on-demand or per user choice.

### 6.3 Push Flow
- Convert local annotations/notes to Zotero annotation item schema.
- Use optimistic concurrency via Zotero item versions.

## 7. OCR & PDF Processing (Phase 4+ - CLI implemented, web not yet)

### 7.1 CLI OCR (implemented)
- `crates/cli`: `ocr run-next` and `ocr run <job-id>`
- `ScriptPaddleOcrRunner`: calls external `scripts/run_paddle_ocr.sh`
- `LocalOcrExecutor`: resolves managed PDF assets, runs OCR, persists results
- SQLite persistence for OCR jobs and results

### 7.2 Web OCR (not yet implemented)
- Future: browser-side Tesseract.js or gateway-side PaddleOCR
- Progressive OCR for large documents

## 8. Gateway API (HTTP)

This API is intentionally small and stable.

### 8.1 Health ✅ Implemented
- `GET /health`
- Response: `200 OK` + `{"status": "ok"}`

### 8.2 AI ✅ Implemented
- `POST /api/translate`
  - Request body: `{ "text": String, "target_language": String, "context": Option<String> }`
  - Response: `{ "translation": String, "source_text": String, "target_language": String, "normalized_term": Option<String> }`
- `POST /api/explain`
  - Request body: `{ "text": String, "explanation_type": String, "language": String, "context": Option<String> }`
  - Response: `{ "explanation": String, "background": Option<String>, "caution": Option<String> }`
- Both routes protected by `require_token` middleware when `GATEWAY_TOKEN` is set.

### 8.3 Sync - S3 (Phase 5+ - not implemented)
- `POST /sync/push`
- `POST /sync/pull`

### 8.4 Assets (Phase 5+ - not implemented)
- `POST /assets/upload/init`
- `GET /assets/download`

### 8.5 Zotero (Phase 6+ - not implemented)
- `POST /zotero/connect`
- `POST /zotero/pull`
- `POST /zotero/push`

### 8.6 OCR (Phase 4+ - not implemented in web)
- `POST /ocr/document`
- `POST /ocr/page`

## 9. Gateway Internal Storage (Phase 5+ - not yet implemented)

### 9.1 Database (Gateway)
Gateway DB will store:
- connector configs (S3, Zotero)
- mapping tables
- job records
- audit logs

It should **not** be treated as the canonical store of reading data.

### 9.2 Secrets Handling (not yet implemented)
- Store S3 AK/SK and Zotero API key in gateway-side secrets store.
- Never return raw secrets to the browser.

## 10. Observability
- Structured `tracing` logs per request in gateway.
- Browser console warnings for IndexedDB unavailability.
- Import progress stages surfaced in UI.

## 11. Edge Cases to Encode in Implementation
- Multi-column PDF reading order issues: store raw pdf.js text items.
- Garbled fonts/unicode errors: trigger OCR (Phase 4+).
- Unanchored annotations: display with `Unanchored` status and support re-anchoring.
- Very large scanned PDFs: progressive OCR (page-on-demand).
- Zotero rate limits: respect Retry-After; batch requests (Phase 6+).
