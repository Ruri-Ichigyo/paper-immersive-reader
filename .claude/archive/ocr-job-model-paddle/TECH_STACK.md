# TECH_STACK — Paper Immersive Reader

## 0. Technical Principles
This stack must satisfy the product constraints:
- **Web-first**
- **Local-first**
- offline-capable for non-AI reading workflows
- extensible toward desktop/app later
- suitable for rich text selection, annotation anchoring, PDF parsing, AI side panel, and user-managed sync

## 1. Architecture Decision Summary

### 1.1 Recommended Architecture
**Split architecture:**
- **Frontend Web App**: local-first PWA responsible for reading experience, local storage, parsing orchestration, annotations, export, and initiating sync.
- **Optional Sync/Integration Gateway**: separate deployable backend service for S3 and Zotero credentials, sync operations, AI proxying if needed, and heavy async jobs when browser-side execution is not sufficient.

### 1.2 Why this architecture
A pure browser-only solution is not enough for all requirements because:
- S3 AK/SK should not live permanently in browser storage if avoidable.
- Zotero bi-directional sync and attachment transfer are operationally easier through a backend/gateway.
- OCR and PDF processing may be too heavy or inconsistent purely in-browser for large files.
- Local-first remains intact because local app remains the source of day-to-day reading state.

## 2. Frontend Stack

### 2.1 Framework
- **Next.js 15+ (App Router)**
- **TypeScript**

Rationale:
- strong ecosystem
- route-based app structure
- future SSR/edge flexibility for landing/auth/setup pages if needed
- easy packaging as a PWA shell

### 2.2 UI Layer
- **React 19**
- **Tailwind CSS**
- **shadcn/ui** for primitives
- **Lucide** for icons

Rationale:
- fast composition for app-like UI
- readable design system control
- high-quality primitives without locking into a monolithic component library

### 2.3 State Management
Use layered state intentionally:
- **Zustand** for app/session state:
  - current paper
  - reading block index
  - panel state
  - transient selection state
- **TanStack Query** for async remote/gateway calls:
  - AI explanations
  - Zotero sync status
  - S3 sync jobs
- local derived selectors for UI responsiveness

### 2.4 Forms & Validation
- **React Hook Form**
- **Zod**

For settings pages, import flows, S3 config, export options.

## 3. Local-First Persistence

### 3.1 Local Database
- **IndexedDB** as the primary local datastore
- **Dexie.js** as the IndexedDB wrapper

Reason:
- browser-native durable storage
- supports offline use
- suitable for structured paper, annotation, vocab, and AI record data
- much better fit than localStorage for large PDFs and structured documents

### 3.2 File Storage Strategy
Store original PDF assets in one of:
- IndexedDB Blob storage for smaller-to-medium files
- or File System Access API as an optional power-user mode on supported browsers

Recommended default:
- use **IndexedDB Blobs** for portability

### 3.3 Search Index
- **MiniSearch** or **FlexSearch** in-browser for local search over:
  - paper title
  - parsed text blocks
  - annotations
  - vocabulary

## 4. PDF Parsing and OCR

### 4.1 PDF Rendering / Text Layer
- **pdf.js** for rendering PDF pages and accessing text items

### 4.2 Text Extraction Pipeline
Recommended layered strategy:
1. Try structured text extraction using **pdf.js** text content
2. Run cleanup heuristics:
   - repeated line detection (header/footer)
   - paragraph merging
   - hyphenation cleanup
3. If extracted text quality is below threshold, mark as OCR candidate

### 4.3 OCR
Recommended options:
- **Browser-first fallback:** Tesseract.js for smaller/local OCR experiments or limited pages
- **Production-recommended:** OCR via **Gateway worker** using:
  - PaddleOCR / Tesseract / cloud OCR adapter depending deployment choice

Decision:
- In docs, standardize on **Gateway OCR service** as primary production path.
- Browser OCR may exist as fallback or dev-mode option but should not be the primary path for large scanned PDFs.

### 4.4 Segmentation / Chunking
- deterministic first-pass sentence segmentation using a text splitter library or custom NLP rules
- AI-assisted adaptive chunking service to merge sentences into reader-friendly blocks

Rule:
- Store both:
  - canonical extracted text
  - derived block segmentation version

This avoids losing re-segmentation options later.

## 5. Annotation & Text Anchoring

### 5.1 Anchor Model
Use a hybrid anchor model for resilience:
- `paperId`
- `blockId`
- `textQuote` exact quote
- `prefix` / `suffix` context
- `startOffset` / `endOffset`
- optional PDF page reference if available

Reason:
Text extraction and re-segmentation may change offsets; quote-based anchoring improves restoration.

### 5.2 Annotation Rendering
- custom annotation layer over rendered reading blocks
- PDF raw page annotation is not the primary UX in MVP; immersive reading blocks are primary

## 6. AI Integration Stack

### 6.1 AI Call Boundary
AI requests should go through a backend/gateway in production.
Reasons:
- API key protection
- rate limiting
- auditing / prompt versioning
- easier model switching

### 6.2 AI Features Covered
- term explanation
- sentence explanation
- paragraph summary
- full paper summary
- structure extraction
- Q&A
- note generation
- keyword extraction

### 6.3 Prompting Rules
- input should include selected text + minimal context window
- output schema should be structured JSON when possible
- responses must separate:
  - direct explanation
  - optional background knowledge
  - confidence or caution note for ambiguous scientific claims

## 7. Sync and Integration Stack

### 7.1 S3-Compatible Sync
Recommended approach:
- Gateway service uses **AWS SDK v3** with S3-compatible configuration
- store serialized snapshots + incremental entity bundles
- client uploads/downloads through Gateway APIs, not directly with raw AK/SK where possible

### 7.2 Sync Model
Use **entity-based sync** rather than whole-database overwrite.
Sync entities:
- papers
- paper assets metadata
- progress
- annotations
- favorites
- vocabulary
- AI records
- settings profiles

### 7.3 Zotero Integration
Recommended approach:
- Gateway uses Zotero Web API for metadata and note sync
- attachment/PDF handling may require import/export pipeline depending deployment mode
- maintain external mapping table:
  - `paperId <-> zoteroItemKey`
  - `annotationId <-> zoteroAnnotationKey` if applicable

## 8. Backend / Gateway Stack

### 8.1 Recommended Backend Framework
- **Fastify** or **NestJS** on Node.js

Recommendation:
- **Fastify + TypeScript** for lower overhead and clear service boundaries.

### 8.2 Background Jobs
- **BullMQ** with Redis for OCR / sync / import jobs if self-hosted infra exists
- If avoiding Redis early, start with in-process queue for single-user deployment and graduate later

### 8.3 Database (Gateway Side)
Gateway does not need to be the system of record for reading data, but it needs operational storage for:
- sync jobs
- connector configs
- mapping tables
- audit logs

Recommended:
- **PostgreSQL**
- **Prisma** ORM

## 9. Packaging and Deployment

### 9.1 Web App
- deploy on Vercel / self-hosted Node environment
- enable PWA using `next-pwa` or equivalent

### 9.2 Sync Gateway
- deploy separately on Docker-compatible environment
- expose HTTPS endpoint

### 9.3 Desktop Extension Path
When expanding to desktop:
- reuse frontend app in **Tauri** shell
- gain stronger local file access and background processing

## 10. Testing Stack

### 10.1 Frontend
- **Vitest** for unit tests
- **React Testing Library** for component behavior
- **Playwright** for end-to-end flows

### 10.2 Backend
- **Vitest** or **Jest**
- **Supertest** for API tests

### 10.3 Parsing Quality Tests
Maintain fixture corpus of PDFs:
- normal text PDFs
- two-column academic PDFs
- image/scanned PDFs
- PDFs with repeated headers/footers
- malformed PDFs

## 11. Observability
- structured logs in gateway
- client error capture for import/sync failures
- per-step import telemetry:
  - extract success rate
  - OCR fallback rate
  - parse quality score

## 12. Versioning Decisions
- schema version for local DB
- segmentation version for block generation
- prompt version for AI-derived content
- sync protocol version for gateway payloads

## 13. Final Recommended Stack Summary

### Frontend
- Next.js + TypeScript
- React + Tailwind + shadcn/ui
- Zustand + TanStack Query
- Dexie / IndexedDB
- pdf.js
- React Hook Form + Zod
- MiniSearch/FlexSearch
- Vitest + RTL + Playwright

### Gateway / Backend
- Node.js + Fastify + TypeScript
- PostgreSQL + Prisma
- BullMQ + Redis (when async load grows)
- AWS SDK v3 (S3-compatible)
- Zotero Web API integration
- OCR worker pipeline

### Future Desktop
- Tauri wrapping the same frontend
