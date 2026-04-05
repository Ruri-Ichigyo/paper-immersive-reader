# APP_FLOW — Paper Immersive Reader

## 0. Purpose
Define end-to-end user flows and key state transitions for the web-first, local-first English paper immersive reading tool.

## 1. Primary Screens (Web)

### 1.1 Home / Library
- Purpose: list all imported papers (reading objects).
- Core actions:
  - Import paper (PDF / paste text / URL / DOI / Zotero)
  - Open reading page
  - Search/filter library (P1)
  - Open paper details (metadata, progress, export)

### 1.2 Import Wizard
- Purpose: ingest paper content and create a persistent reading object.
- Import modes:
  1) Local PDF
  2) Paste text
  3) Web URL
  4) DOI / paper link
  5) Zotero

### 1.3 Reading Page (Immersive)
- Purpose: distraction-free block-by-block reading.
- Main regions:
  - Content area (current block)
  - Navigation (Next block, Previous block)
  - Progress indicator
  - Annotation controls
  - AI side panel (term/sentence explanation, summaries, Q&A)

### 1.4 Vocabulary (Wordbook)
- Purpose: manage looked-up words.
- Core actions:
  - List, search, tag, mark as learned
  - Jump back to source context

### 1.5 Annotations & Notes
- Purpose: browse/search/export annotations.
- Core actions:
  - Filter by tag/type/paper
  - Export (original text + annotations + summaries + vocabulary)

### 1.6 Settings
- AI explanation language
- Tag management (default + custom)
- Sync settings (S3 endpoint/bucket/AK/SK)
- Zotero connection

## 2. Data Objects and Reading States

### 2.1 Paper Lifecycle State Machine
States:
1. `CREATED` — object exists with minimal metadata
2. `INGESTING` — import pipeline running
3. `READY_TEXT` — text extracted and segmented; reading possible
4. `READY_OCR` — OCR completed (if needed) and segmented
5. `FAILED` — ingest failed; error captured

Transitions:
- `CREATED` → `INGESTING` on import start
- `INGESTING` → `READY_TEXT` when PDF/text extraction and segmentation succeed
- `INGESTING` → `READY_OCR` when OCR path completes successfully
- `INGESTING` → `FAILED` when any required pipeline step fails
- `FAILED` → `INGESTING` on user retry

### 2.2 Reading Position State
Represent reading as an index into canonical `blocks[]`.
- `currentBlockId`
- `currentBlockIndex`
- `progressPercent` computed as `index / blocks.length`

Rule:
- `Next block` increments index by 1.
- If block list changes (re-segmentation), map old anchors to new anchors by `blockHash` / `textQuote` fallback.

### 2.3 Selection Interaction State
- `selection` (start offset, end offset, selected text)
- `selectionContext` (blockId, paperId)

Selection outcomes:
- Translate selection
- Explain selection (term or sentence)
- Annotate selection

## 3. Core Flows

## 3.1 Flow A — Import a Local PDF (P0)

### Entry
User at Home/Library → clicks Import → chooses Local PDF.

### Steps
1. User selects PDF file.
2. System creates Paper object:
   - metadata: `importMethod=pdf`, `importedAt`, placeholder title (until extracted)
   - state: `CREATED`
3. System transitions to `INGESTING`.
4. Pre-processing pipeline runs:
   - Extract text (FR-03)
   - Detect and remove header/footer noise (FR-05)
   - Auto paragraph segmentation (FR-04)
   - AI adaptive chunking into blocks (FR-12)
5. If text extraction yields insufficient text (threshold):
   - Run OCR pipeline (FR-06)
   - Re-run segmentation + chunking
6. On success: state `READY_TEXT` or `READY_OCR`.
7. User is routed to Reading Page at block index 0.

### Exit / Success
- Reading page renders first block.
- Progress shows 0%.

### Failure Handling
- If pipeline fails:
  - Paper enters `FAILED` state with error detail.
  - UI shows retry + diagnostics hints (e.g., “scanned PDF detected, try OCR”).

## 3.2 Flow B — Import Paste Text (P0)
1. User chooses Paste Text.
2. Paste content.
3. System creates Paper with `importMethod=paste`.
4. System runs:
   - normalization
   - paragraph segmentation
   - AI chunking
5. Enter `READY_TEXT` and route to Reading.

## 3.3 Flow C — Import by URL / DOI (P0/P1 boundary)
Because publisher paywalls and formats vary:
- **P0**: accept URL/DOI as metadata and attempt fetch only when content is directly retrievable.
- If fetch fails: store link and allow user to upload PDF manually.

Flow:
1. User enters URL/DOI.
2. System creates Paper object with source link.
3. System attempts to retrieve full text or PDF.
4. If success, proceed as PDF pipeline; else Paper is `CREATED` with “needs PDF upload” status.

## 3.4 Flow D — Immersive Reading Navigation (P0)

### Entry
Paper state `READY_*`.

### Steps
1. Read current block.
2. Click `Next block`.
3. System:
   - updates `currentBlockIndex`
   - persists reading position (FR-31)
   - updates progress (FR-09)
4. (Optional) Auto-trigger AI: if user enables “auto explain terms” (future), system fetches suggestions.

### Exit
User leaves reading page; system persists:
- last block index
- last updated timestamp

## 3.5 Flow E — Selection Translate (P0)

### Entry
User selects a word/sentence in current block.

### Steps
1. System captures selection and context.
2. User triggers Translate.
3. System returns translation result:
   - for word/sentence (FR-13)
   - or paragraph (FR-14)
4. UI shows bilingual layout (FR-15).
5. System records vocabulary item if selection is a word/phrase (FR-29).

### Exit
Selection panel closes; vocabulary updated.

## 3.6 Flow F — AI Term Explanation / Sentence Explanation (P0)

### Entry
User selects term/sentence OR clicks explain on block.

### Steps
1. System sends prompt including:
   - selected text
   - minimal surrounding context
   - user setting: explanation language (FR-17)
2. AI response returns:
   - explanation (term or sentence)
   - optional short background section (guardrail)
3. UI renders in right AI panel (FR-16/18).

### Persistence
AI responses are persisted under “AI records” and are part of sync scope (FR-39).

## 3.7 Flow G — Annotate (P0)

### Entry
User selects text range.

### Steps
1. User chooses annotation type: highlight / underline / note / tag.
2. System stores annotation with anchor:
   - paperId
   - blockId + text offsets
   - quote text (for resilience)
   - createdAt
   - type
   - tags
3. UI decorates text in place.

### Default tags
- 重点, 疑问, 术语, 结论, 复习

### Exit
Annotation persists and is included in export and sync.

## 3.8 Flow H — Vocabulary Management (P0/P1)

### Steps
1. User opens Vocabulary.
2. System lists words with:
   - lookup count
   - first seen paperId and location
   - last seen
3. User can mark as learned / add note / tag.

## 3.9 Flow I — Export (P1)

### Entry
User chooses Export for a paper or across papers.

### Output content (default include)
- original text
- annotations
- summaries
- vocabulary

### Output content (default exclude)
- translations
- term explanations
- AI Q&A logs

## 3.10 Flow J — S3 Sync (P2)

### Preconditions
- User has configured S3 endpoint, bucket, AK/SK (FR-38)
- User acknowledges “no extra encryption” (FR-40)

### Flow
1. User opens Settings → Sync.
2. Configure S3.
3. Click Sync now.
4. System uploads all scope data (FR-39):
   - papers, progress, annotations, favorites, vocabulary, AI logs
5. On next device, user configures same S3, downloads and merges.

### Conflict Handling (must be defined)
- Use deterministic merge:
  - progress: max(lastUpdatedAt)
  - annotations: union by annotationId
  - vocabulary: union by vocabId
  - AI logs: union by recordId

## 3.11 Flow K — Zotero Bi-directional Sync (P2)

### Preconditions
User connects Zotero.

### Flow
1. Import from Zotero:
   - metadata + PDFs
   - notes/annotations
2. Export back to Zotero:
   - push annotations/notes created in our app
3. Keep in sync:
   - periodic or manual sync

### Conflict Handling
- treat Zotero as an external authoritative source for bibliographic metadata.
- treat our app as authoritative for reading progress and internal annotations unless explicitly exported.

## 4. Cross-cutting Behaviors

### 4.1 Offline Behavior
- All reading, annotation, vocabulary must function offline.
- AI features degrade gracefully:
  - if offline: show “AI unavailable offline” and keep reading enabled.

### 4.2 Performance Expectations
- Import should show progress stages (extracting, cleaning, chunking, OCR if needed).
- Reading page should prefetch next N blocks and avoid reflow.

### 4.3 Accessibility
- Keyboard navigation for next/previous block.
- Selection interactions must not trap focus.

