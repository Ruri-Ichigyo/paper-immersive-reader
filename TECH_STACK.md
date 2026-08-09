# TECH_STACK - Paper Immersive Reader

## 0. Technical Principles
This stack must satisfy the product constraints:
- **Web-first**
- **Local-first**
- offline-capable for non-AI reading workflows
- extensible toward desktop/app later
- suitable for rich text selection, annotation anchoring, PDF parsing, AI side panel, and user-managed sync

## 1. Architecture Decision Summary

### 1.1 Actual Architecture
**Split architecture (Cargo-native):**
- **Web App**: local-first Leptos CSR application compiled to WASM, responsible for reading experience, local storage, parsing orchestration, annotations, and initiating AI requests.
- **Optional AI Gateway**: thin Rust HTTP gateway (Axum) for AI API key protection, translation/explanation proxying, and future heavy async jobs.
- **CLI**: Rust CLI for OCR job execution and local management (persistence via SQLite).

The browser is the source of truth for reading data. The gateway is optional for core reading and required only for AI-backed actions.

### 1.2 Why this architecture
A pure browser-only solution is not enough for all requirements because:
- AI API keys should not live permanently in browser storage.
- Future S3/Zotero sync and OCR operations are operationally easier through a backend/gateway.
- Local-first remains intact because local app remains the source of day-to-day reading state.

## 2. Frontend Stack

### 2.1 Framework
- **Leptos 0.7** (CSR mode)
- **Rust** compiled to `wasm32-unknown-unknown`

Rationale:
- single-language full-stack (Rust for both frontend and gateway)
- fine-grained reactivity via signals
- strong type safety across the entire stack
- no JS build toolchain beyond Trunk

### 2.2 UI Layer
- Hand-written CSS (`crates/web-app/styles.css`) with CSS custom properties (variables) for theming
- No CSS framework (Tailwind/shadcn not used)
- Responsive grid layout for reading (1fr 400px sidebar)

### 2.3 State Management
Layered state via Leptos primitives:
- **`AppState`** struct provided via Leptos context (`provide_context` / `expect_context`)
- **`RwSignal<T>`** for reactive state:
  - `store: RwSignal<InMemoryStore>` - interior mutability for all domain data
  - `data_version: RwSignal<u64>` - increment-after-mutation pattern for reactivity
  - `settings: RwSignal<Settings>` - UI preferences
  - `gateway_url: RwSignal<String>` / `gateway_token: RwSignal<Option<String>>` - runtime AI config
- **`Callback<T>`** for event handlers passed to child components

### 2.4 Routing
- **`leptos_router` 0.7** for client-side routing
- Routes: `/library`, `/import`, `/read/:paper_id`, `/vocabulary`, `/settings`

## 3. Local-First Persistence

### 3.1 Local Database
- **IndexedDB** as the primary local datastore (browser)
- Direct `web_sys::IdbDatabase` API (no Dexie wrapper)
- `IndexedDbStore` (cfg-gated for `wasm32`) provides async CRUD mirroring the `AppStorage` trait
- `InMemoryStore` used for native testing and as an in-memory cache on wasm32
- DB name: `paper-immersive-reader`, version 2

### 3.2 Object Stores
Eight IndexedDB object stores:
- `papers` (key: paper_id)
- `blocks` (key: block_id, index: paper_id)
- `progress` (key: paper_id)
- `annotations` (key: annotation_id, index: paper_id)
- `vocabulary` (key: vocab_id, index: term_lower)
- `ai_records` (key: record_id, index: paper_id)
- `settings` (key: "app_settings")
- `reading_stats` (key: paper_id)

### 3.3 Persistence Strategy
- On startup (`init_persistence`): open IndexedDB, merge all data into `InMemoryStore`
- After each mutation: `spawn_local` async write to IndexedDB
- Merge (not replace) on load to avoid losing data written during async load

## 4. PDF Parsing and Text Extraction

### 4.1 PDF Text Extraction
- **pdf.js** (vendored at `crates/web-app/assets/pdfjs/`) loaded via `<script>` in `index.html`
- `WasmPdfBridge` calls global `pdfjs_extract_text(Uint8Array) -> Promise<string|null>` via `wasm-bindgen` extern
- `StubPdfBridge` for native testing (returns canned text or error)

### 4.2 Text Cleanup Pipeline
1. Extract text via pdf.js (pages joined by `\n\n`)
2. `filter_repeated_lines`: detect header/footer lines appearing as first/last line of >50% of pages (3+ pages required)
3. `segment_text`: split on `\n\n`; fallback to single-newline split for lines >20 chars
4. Generate `Block` objects with UUID-based short IDs

### 4.3 OCR (Phase 4 - not yet in web app)
- CLI-side: `ScriptPaddleOcrRunner` + `LocalOcrExecutor` via `crates/cli`
- Web-side OCR integration deferred to Phase 4

### 4.4 Segmentation
- Deterministic first-pass paragraph segmentation (double-newline split)
- Single-newline fallback for PDFs without clear paragraph breaks
- AI adaptive chunking (FR-12) deferred to later phase

## 5. Annotation & Text Anchoring

### 5.1 Anchor Model
Hybrid anchor model for resilience:
- `paper_id` / `block_id`
- `quote`: exact selected text
- `prefix` / `suffix`: up to 50 chars of context on each side
- `start_offset` / `end_offset`: character offsets (best-effort)
- `anchoring_status`: `Anchored` | `Unanchored`

### 5.2 Offset Resolution (`find_offsets`)
4-level fallback:
1. Exact substring match
2. First 20 chars of selection as prefix search
3. First word (≥3 chars) of selection
4. None -> `(0, 0)` + `Unanchored` status

### 5.3 Inline Decoration
`BlockContent` component splits block text into annotated and unannotated segments, wrapping ranges in `<mark>`, `<u>`, or `<span>` elements based on annotation type.

## 6. AI Integration Stack

### 6.1 AI Call Boundary
AI requests go through the gateway in production:
- API key protection
- prompt versioning
- structured JSON response parsing
- fallback to `StubAiClient` for development (`GATEWAY_AI_USE_STUB=true`)

### 6.2 Gateway AI Endpoints
- `POST /api/translate` (body: text, target_language, context?)
- `POST /api/explain` (body: text, explanation_type: "term"|"sentence", language, context?)

### 6.3 Browser-Side AI Client
- `GatewayAiClient` (cfg-gated `wasm32`): async HTTP via `gloo-net::http::Request`
- Optional `Authorization: Bearer <token>` header when gateway token is configured
- 401 responses produce user-facing error: "unauthorized - check gateway token in Settings"

### 6.4 Prompting Rules
- Translation: system prompt instructs "return only the translation"
- Explanation: system prompt requests structured JSON with `explanation`, optional `background`, optional `caution`
- `parse_explain_json` strips markdown code fences and falls back to raw text on parse failure

## 7. Sync and Integration Stack (Phase 5+ - not yet implemented)

### 7.1 S3-Compatible Sync (planned)
- Gateway service with S3-compatible configuration
- entity-based sync (papers, progress, annotations, vocabulary, AI records)
- manifest-based incremental sync

### 7.2 Zotero Integration (planned)
- Gateway uses Zotero Web API
- mapping table: paperId ↔ zoteroItemKey

## 8. Backend / Gateway Stack

### 8.1 Framework
- **Axum 0.8** + **tokio** (async runtime)
- **reqwest** for upstream AI API calls

### 8.2 Configuration
`GatewayConfig::from_env()` reads:
- `GATEWAY_BIND_ADDRESS` (default: `127.0.0.1:3000`)
- `GATEWAY_AI_API_KEY` / `AI_API_KEY` (required unless stub)
- `GATEWAY_AI_BASE_URL` (default: `https://api.openai.com/v1`)
- `GATEWAY_AI_MODEL` (default: `gpt-3.5-turbo`)
- `GATEWAY_AI_USE_STUB` (default: `false`)
- `GATEWAY_TOKEN` (optional shared secret)

### 8.3 Authentication
- `require_token` axum middleware: if `GATEWAY_TOKEN` env var is set, validates `Authorization: Bearer <token>` header on all `/api/*` routes
- If not set, all requests pass through

### 8.4 Logging
- `tracing` + `tracing-subscriber` with JSON formatted logs to `logs/YYYY-MM-DD_HHMMSS.log`

## 9. Packaging and Deployment

### 9.1 Web App
- **Trunk** as the build tool: `trunk serve crates/web-app/index.html --open`
- WASM build: `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features`
- `index.html` loads pdf.js assets and registers `pdfjs_extract_text` bridge function

### 9.2 Gateway
- `cargo run -p paper-immersive-reader-gateway`
- Scripts: `scripts/run_gateway.sh` (auto-sources `.env`), `scripts/run_web_app.sh`

### 9.3 Future Desktop Extension
- Tauri wrapping the same Leptos frontend (planned)

## 10. Testing Stack

### 10.1 Compilation Gates
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- WASM build: `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features`

### 10.2 Test Organization
- Integration tests in `tests/` directory (73 test files)
- Native tests use `InMemoryStore` + `StubPdfBridge` + `WebAiClient`
- WASM-specific code cfg-gated, tested via compilation

### 10.3 PDF Fixture Corpus
- Real PDF fixtures under `tests/` for embedded-text and OCR-boundary scenarios

## 11. Observability
- Structured `tracing` logs in gateway
- Browser console warnings for IndexedDB unavailability
- Per-step import progress UI ("Extracting text…" -> "Cleaning headers…" -> "Segmenting blocks…")

## 12. Versioning Decisions
- IndexedDB schema version: 2 (with `onupgradeneeded` migration)
- `parse_version: 1` on Paper (reserved for future re-segmentation)
- Block IDs: UUID v4 short segments (`block-xxxxxxxx`)

## 13. Final Actual Stack Summary

### Frontend (Web App)
- Rust + Leptos 0.7 (CSR) + wasm-bindgen + web_sys
- leptos_router 0.7 for client-side routing
- IndexedDB via web_sys (8 object stores, DB v2)
- pdf.js via wasm-bindgen extern
- gloo-net for gateway HTTP calls
- Hand-written CSS with CSS variables
- Trunk for dev server and build

### Gateway
- Rust + Axum 0.8 + tokio + reqwest
- OpenAI-compatible chat completions API
- StubAiClient for development
- Token middleware (optional Bearer auth)
- tracing for structured logging

### CLI
- Rust CLI for OCR job execution
- SQLite persistence (`database/app-state.db`)
- JSON-to-SQLite auto-migration
- ScriptPaddleOcrRunner for OCR execution

### Shared Crates
- `crates/core`: domain types (Paper, Block, Annotation, VocabularyItem, ReadingProgress, OcrJob)
- `crates/application`: import orchestration, OCR executor, persistence contracts
- `crates/importing`: PDF extraction, text normalization, segmentation
