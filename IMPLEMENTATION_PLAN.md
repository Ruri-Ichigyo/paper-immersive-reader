# IMPLEMENTATION_PLAN — Paper Immersive Reader

## 0. Objective
Deliver a **web-first, local-first English academic paper immersive reader** with a phased implementation sequence that reduces risk around PDF parsing, OCR, AI accuracy, and sync consistency.

This plan is execution-oriented and removes ambiguity by defining:
- implementation phases
- dependencies
- concrete deliverables
- acceptance criteria
- explicit out-of-scope boundaries per phase

## 1. Product Delivery Strategy

### 1.1 Principle
Build the product in the order of **highest user value + lowest architectural regret**:
1. local reading loop first
2. content quality second
3. AI assistance third
4. sync/integration last

### 1.2 Why this order
Without a strong local reading core, AI/sync features only amplify poor parsing and weak UX.

## 2. Phase Plan Overview

### Phase 0 — Foundation
Goal:
- establish app shell, local data model, import scaffolding, and reading page skeleton.

### Phase 1 — P0 Core Closed Loop
Goal:
- deliver the minimum usable product:
  - import PDF/text
  - parse text
  - immersive reading
  - selection translation
  - AI explanation
  - annotations
  - progress persistence
  - vocabulary auto-collection

### Phase 2 — Reading Quality Improvements
Goal:
- improve parsing reliability, chunk quality, block anchoring, error handling, export quality.

### Phase 3 — P1 Knowledge Features
Goal:
- summaries, structure extraction, Q&A, notes, keywords, annotation search/export.

### Phase 4 — OCR + Advanced PDF Resilience
Goal:
- scanned PDF support and higher annotation robustness.

### Phase 5 — S3 Sync
Goal:
- user-managed full-content sync across devices.

### Phase 6 — Zotero Bi-directional Sync
Goal:
- metadata + PDF + annotations/notes sync with Zotero.

## 3. Detailed Phase Breakdown

## 3.1 Phase 0 — Foundation

### Scope
- Next.js app scaffold
- local DB schema (IndexedDB via Dexie)
- app routes
- UI shell for Library / Import / Reading / Vocabulary / Annotations / Settings
- typed domain models

### Deliverables
- route structure exists
- local DB initialized with migrations/versioning
- core entities implemented:
  - Paper
  - Block
  - ReadingProgress
  - Annotation
  - VocabularyItem
  - AIRecord
  - TagDefinition
- import wizard shell exists
- reading page skeleton exists

### Dependencies
None.

### Acceptance Criteria
- app boots locally
- can create a Paper stub in local DB
- can navigate between all core pages
- DB schema versioning is in place

### Explicitly Not Done
- real PDF parsing
- OCR
- sync
- Zotero
- production AI

## 3.2 Phase 1 — P0 Core Closed Loop

### Scope
Implement the core reading loop from requirements.

### Deliverables
1. Import:
   - local PDF upload
   - paste text import
2. Parsing:
   - PDF.js text extraction
   - basic header/footer filtering
   - paragraph segmentation
   - initial block generation
3. Reading:
   - immersive reading page
   - previous/next block navigation
   - progress indicator
   - last position restore
4. Translation:
   - selection translation for word/sentence
   - paragraph translation
   - bilingual result UI
5. AI assistance:
   - term explanation
   - sentence explanation
   - language setting for explanations
6. Annotations:
   - highlight / underline / note / tags
7. Vocabulary:
   - auto-collect looked-up words

### Dependencies
- Phase 0 completed
- AI provider/gateway path decided

### Acceptance Criteria
- user imports a PDF and reaches reading page
- reading page renders block 1 with progress 0%
- next/previous navigation updates block and persists position
- user can select text and trigger translation
- translation result appears in bilingual layout
- user can request AI explanation for a selected term/sentence
- annotation persists after reload
- looked-up word appears in vocabulary page

### Risks
- parse quality may be weak for multi-column papers
- AI explanation may over-explain without guardrails

### Mitigation
- keep block architecture flexible
- keep AI prompts constrained and structured

## 3.3 Phase 2 — Reading Quality Improvements

### Scope
Improve robustness of text quality and reader experience.

### Deliverables
- improved repeated header/footer detection
- hyphenation cleanup
- block re-segmentation versioning
- multi-column detection heuristics
- resilient annotation anchors:
  - exact quote
  - prefix/suffix
  - offsets
- import diagnostics UI
- export v1:
  - original text
  - annotations
  - summaries
  - vocabulary

### Dependencies
- Phase 1 complete

### Acceptance Criteria
- repeated headers/footers are removed for common academic layouts
- annotations survive re-open and minor re-segmentation
- export file excludes translations / term explanations / AI Q&A by default

## 3.4 Phase 3 — P1 Knowledge Features

### Scope
Enhance comprehension and review workflows.

### Deliverables
- paragraph summary
- full paper summary
- structure extraction:
  - objective
  - methods
  - results
  - conclusion
- paper Q&A
- note generation
- keyword extraction
- key sentence marking
- annotation search/filter
- export annotations by category

### Dependencies
- Phase 1 stable
- prompt schemas finalized

### Acceptance Criteria
- summaries are stored in AIRecord and accessible later
- user can ask a question bound to current paper context
- annotation search returns matching note/tag/text results

## 3.5 Phase 4 — OCR + Advanced PDF Resilience

### Scope
Support scanned/image PDFs and improve annotation stability.

### Deliverables
- scanned PDF detection
- OCR fallback pipeline
- OCR confidence thresholding
- progressive OCR for large documents
- unanchored annotation handling UI

### Dependencies
- Phase 2 complete
- OCR path selected (browser or gateway primary)

### Acceptance Criteria
- scanned PDFs can produce readable text blocks
- OCR confidence is stored and surfaced when low
- annotation failures are shown as unanchored rather than silently lost

## 3.6 Phase 5 — S3 Sync

### Scope
Implement user-managed full-content sync.

### Deliverables
- settings UI for endpoint / bucket / AK / SK (or gateway connection)
- entity manifest model
- push/pull sync flows
- sync status UI
- deterministic conflict handling
- full scope sync:
  - papers
  - progress
  - annotations
  - favorites
  - vocabulary
  - AI records

### Dependencies
- local entity models stable
- gateway API available (recommended)

### Acceptance Criteria
- two devices can converge on same dataset through S3
- conflicts are not silent
- uploads use no extra client-side encryption
- sync can resume after interruption

### Risk
- sync conflicts across devices

### Mitigation
- manifest + per-entity metadata + conflict copies

## 3.7 Phase 6 — Zotero Bi-directional Sync

### Scope
Integrate Zotero as a scholarly library companion.

### Deliverables
- connect Zotero account
- import metadata + PDF + notes/annotations
- push notes/annotations back
- maintain local↔Zotero mapping tables
- conflict behavior defined for metadata vs annotations vs local progress

### Dependencies
- Phase 5 preferred, though limited read-only import can start earlier
- gateway connector implemented

### Acceptance Criteria
- imported Zotero items create Paper objects correctly
- Zotero PDF attachment mapping works
- annotations/notes sync without duplicating endlessly

## 4. Engineering Workstreams

### 4.1 Workstream A — Data Layer
- domain entities
- Dexie schema
- migrations
- repositories/services

### 4.2 Workstream B — Reader Experience
- reading layout
- block navigator
- selection popover
- annotation renderer
- AI panel

### 4.3 Workstream C — Document Pipeline
- PDF extraction
- cleanup heuristics
- segmentation
- OCR fallback

### 4.4 Workstream D — AI Layer
- prompt schemas
- structured response parsing
- caching/persistence
- guardrails

### 4.5 Workstream E — Sync & Integrations
- S3 sync
- gateway API
- Zotero connector

## 5. Milestone Map

### Milestone M1 — Usable Local Reader
Includes:
- Phase 0 + critical subset of Phase 1

Success means:
- import PDF/text
- read by block
- save progress
- annotate locally

### Milestone M2 — Comprehension Assistant
Includes:
- rest of Phase 1 + selected Phase 3 features

Success means:
- translate
- explain terms/sentences
- vocabulary auto-collection

### Milestone M3 — Reliable Academic PDF Handling
Includes:
- Phase 2 + Phase 4

Success means:
- better parsing
- OCR fallback
- resilient anchors

### Milestone M4 — Cross-device Knowledge Base
Includes:
- Phase 5 + Phase 6

Success means:
- S3 sync
- Zotero sync

## 6. Acceptance Checklist by Requirement Cluster

### 6.1 Import & Parsing
- [x] local PDF import works
- [x] paste text import works
- [ ] URL/DOI import fallback path defined (P1)
- [x] main text extraction exists (pdf.js via WasmPdfBridge)
- [x] header/footer cleanup exists (`filter_repeated_lines`)
- [x] segmentation exists (`segment_text` with double-newline + single-newline fallback)
- [ ] OCR exists in web app (Phase 4; CLI-side OCR implemented via PaddleOCR)

### 6.2 Reading
- [x] dedicated reading page (`/read/:paper_id`)
- [x] fullscreen mode (CSS `reading-fullscreen` class + body class toggle)
- [x] progress display (block index + percentage + progress bar)
- [x] next block (button + keyboard: ArrowRight/PageDown/Space)
- [ ] adaptive chunking (FR-12; current: rule-based segmentation, AI chunking deferred)

### 6.3 Translation & AI
- [x] selection translation (word/sentence via `GatewayAiClient::translate_async`)
- [x] paragraph translation ("Translate Block" button)
- [x] bilingual display (flex row: English left, Chinese right; responsive stack on mobile)
- [x] term explanation (`GatewayAiClient::explain_async` with explanation_type="term")
- [x] explanation language switching (Settings: en/zh/ja/ko)
- [x] sentence explanation (explanation_type="sentence")
- [ ] summary / structure / Q&A / notes / analysis (Phase 3, P1)

### 6.4 Annotation & Vocabulary
- [x] highlight / underline / note / tag
- [x] default tags (重点 / 疑问 / 术语 / 结论 / 复习)
- [x] custom tags (Settings → comma-separated)
- [x] auto vocabulary collection (single-word lookup → `upsert_vocabulary`)
- [x] vocabulary management (search, filter learned/learning, mark learned, add/edit notes, source link)
- [x] restore last location (`load_progress` → `current_block_index` on mount)
- [x] reading time statistics (`ReadingStats`: total_reading_ms, session_count, per-session tracking)
- [x] annotation delete (delete button + IndexedDB sync)
- [x] annotation edit (inline note editor in AnnotationRenderer)
- [x] annotation search/filter (cross-paper browser at `/annotations`: search by text/note, filter by type/tag)
- [x] export (JSON download from Library: paper + blocks + annotations + vocabulary, excludes AI records)

### 6.5 Export / Sync / Zotero
- [x] export defaults match PRD (Phase 2+; implemented: ExportBundle with paper/blocks/annotations/vocabulary/reading_stats, excludes ai_records)
- [ ] S3 sync all content (Phase 5)
- [ ] no extra encryption before upload (Phase 5)
- [ ] Zotero metadata + PDF + annotations/notes bi-directional sync (Phase 6)

### 6.6 Additional Implemented Features (beyond original checklist)
- [x] paper deletion with cascade (library → confirm dialog → delete paper + blocks + progress + annotations + ai_records + reading_stats)
- [x] vocabulary source link (jump back to paper where word was first seen)
- [x] import progress stages ("Extracting text…" → "Cleaning headers…" → "Segmenting blocks…" → "Done")
- [x] keyboard navigation (ArrowRight/PageDown/Space=next, ArrowLeft/PageUp=prev, F=fullscreen)
- [x] gateway token authentication (optional Bearer token middleware)
- [x] library paper card: reading progress bar + percentage + last read time + reading duration + session count
- [x] IndexedDB Closure leak fix (`Closure::once_into_js` instead of `forget()`)

## 7. Delivery Risks and Responses

### Risk 1 — Poor PDF body extraction
Response:
- preserve raw page items
- add fixture corpus early
- implement OCR fallback rather than overfitting heuristics

### Risk 2 — Annotation drift
Response:
- hybrid anchor strategy: quote + prefix/suffix + offsets
- expose unanchored state rather than silently failing

### Risk 3 — AI hallucination
Response:
- narrow prompts
- structured JSON outputs
- label background knowledge separately

### Risk 4 — Sync inconsistency
Response:
- per-entity version metadata
- deterministic merge rules
- conflict UI

## 8. Suggested Build Order Inside the Team
1. Data layer + routes
2. Reader shell
3. PDF import + text extraction
4. Annotation + progress persistence
5. Translation + AI explanation
6. Vocabulary
7. Export
8. OCR
9. Sync
10. Zotero

## 9. Definition of Done
A phase is done only when:
- scoped deliverables are implemented
- acceptance criteria are verifiably satisfied
- no contradictions with PRD / APP_FLOW / TECH_STACK / FRONTEND_GUIDELINES / BACKEND_STRUCTURE remain
- regressions for earlier completed phases are not introduced
