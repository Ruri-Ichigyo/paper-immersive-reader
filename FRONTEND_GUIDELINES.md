# FRONTEND_GUIDELINES - Paper Immersive Reader (Web)

## 0. Goals
Define frontend information architecture, UI layout regions, interaction rules, and component conventions to implement the immersive English paper reading experience.

Non-negotiable constraints:
- **Web-first**
- **Local-first** default
- **Readability-first** UI (distraction-free)
- **No sharing features**

## 1. Information Architecture (IA)

### 1.1 Top-level navigation
- Library (Home)
- Import
- Annotations (cross-paper annotation browser)
- Vocabulary (Wordbook)
- Settings

Navigation bar (`app-nav` class) is present on all pages except during fullscreen reading.

Navigation rules:
- Reading page is accessed by opening a paper from Library.
- Fullscreen reading hides global navigation via `body.reading-fullscreen-active` CSS class.
- Exit fullscreen via button or `F` key.

### 1.2 Route map (actual)
- `/` - Redirect to `/library`
- `/library` - Library page
- `/import` - Import page
- `/read/:paper_id` - Immersive Reading Page
- `/annotations` - Cross-paper annotation browser
- `/vocabulary` - Vocabulary page
- `/settings` - Settings page

Routing implemented via `leptos_router` 0.7 with `<Router>` / `<Routes>` / `<Route>` components.

## 2. Core Layouts

### 2.1 Library Page
Primary regions:
- Header: "Library" title + "Import Paper" button
- Main: list of paper cards

Paper card shows:
- title (or "Untitled")
- import source badge (PDF/Paste/URL/DOI/Zotero)
- import time (relative: "2d ago")
- status badge (Created/Processing/Ready/Ready OCR/Failed)
- reading progress bar + percentage
- last read time (relative)
- reading duration ("2h 15m") + session count
- "Read →" button
- "Delete" button (with confirmation dialog)

### 2.2 Import Page
Two-column grid layout (`.import-sections`):

**Paste Text section:**
- Title input (optional)
- Content textarea
- Import button
- Progress stage indicator

**Upload PDF section:**
- Title input (optional)
- File picker
- Import button
- Progress stage indicator

Progress stages (PDF): "Extracting text…" → "Cleaning headers…" → "Segmenting blocks…" → "Done!"
Progress stages (Paste): "Segmenting text…" → "Done!"

Stages displayed with spinner animation (`.import-spinner`).

### 2.3 Immersive Reading Page (核心)

#### Layout
CSS Grid: `1fr 400px` (main content + sidebar).

- **Main content column** (`.reading-main`):
  - Toolbar: progress indicator + "Translate Block" button + "Fullscreen" button
  - Block content area (`.block-content`)
  - Block navigator (Previous / Next buttons)
  - Selection popover (appears on text selection)
  - Annotation form overlay (modal)
  - Annotations list (for current paper)
- **Sidebar** (`.reading-sidebar`):
  - Translation panel (bilingual: English left, Chinese right)
  - AI panel (term/sentence explanation)

#### Fullscreen mode
- `.reading-fullscreen` class on the layout: `position: fixed; z-index: 200`
- `body.reading-fullscreen-active` hides the nav bar
- Exit via "Exit Fullscreen" button or `F` key

#### Required UI elements
- Progress indicator:
  - "Block X of Y (Z%)" text
  - Progress bar (`.progress-fill` width = percentage)
- Block navigation:
  - "← Previous" / "Next →" buttons
  - Disabled at boundaries
- Selection popover:
  - Annotate
  - Translate
  - Explain Term
  - Explain Sentence
  - Dismiss (✕)
- Annotation form (modal overlay):
  - Type selector (Highlight / Underline / Note)
  - Note textarea (optional)
  - Tag selector (default tags + custom tags from Settings)
  - Save / Cancel buttons
- Annotation list:
  - Each annotation: quote, type styling, note (if any), tags, edit button, delete button (✕)

#### Interaction rules
- `Next block` persists reading position immediately (`save_progress`).
- Selection popover appears on `mouseup` after text selection.
- AI requests are async (`spawn_local`); loading state shown in panels.
- Vocabulary auto-collected on single-word translation.
- Reading session time tracked: start on mount, accumulate on unmount (`on_cleanup`).
- Annotation inline decoration: `BlockContent` splits text by annotation offsets, wraps in `<mark>`/`<u>`/`<span>`.

### 2.4 Vocabulary Page
Table layout with search and filter:

- Search input (filters by term or note, case-insensitive)
- Filter buttons: All / Learning / Learned
- Table columns: Term | Lookups | First Seen | Last Seen | Note | Status | Actions
- Note column: inline editor (Add Note / Edit / Save / Cancel)
- Actions: "Mark Learned"/"Unlearn" toggle + "📄 Source" link (jumps to `/read/{paper_id}`)

### 2.5 Settings Page
Form layout:
- AI Explanation Language (select: English / 中文 / 日本語 / 한국어)
- AI Gateway URL (text input, default: `http://127.0.0.1:3000`)
- Gateway Token (password input, optional)
- Custom Tags (comma-separated text input)
- Save button + success message

## 3. Readability and Typography

### 3.1 English reading typography
- Block content: `font-size: 1.15rem; line-height: 1.8;`
- Body: system font stack (`-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, ...`)
- Max reading width constrained in fullscreen (`max-width: 800px`)

### 3.2 Bilingual layout rules (FR-15) ✅ Implemented
- Translation panel uses `.tl-result` with `display: flex; flex-direction: row; gap: 1rem`
- Left: `.tl-source` (English, italic, with "English" label)
- Right: `.tl-target` (Chinese, with language label)
- Responsive: `@media (max-width: 600px)` switches to `flex-direction: column`

### 3.3 Highlighting and emphasis
- Highlight: `.ann-inline-highlight` (`background: #fef3c7`)
- Underline: `.ann-inline-underline` (`text-decoration: underline; text-decoration-color: #3b82f6; text-decoration-thickness: 2px`)
- Note: `.ann-inline-note` (`border-bottom: 2px dashed #10b981; cursor: help`)

## 4. Component Conventions

### 4.1 Component directory structure (actual)
```
crates/web-app/src/
├── components/
│   ├── ai_panel.rs           (AiPanel)
│   ├── annotation_renderer.rs (AnnotationRenderer)
│   ├── block_content.rs      (BlockContent)
│   ├── block_navigator.rs    (BlockNavigator)
│   ├── progress_indicator.rs (ProgressIndicator)
│   ├── selection_popover.rs  (SelectionPopover)
│   └── translation_panel.rs  (TranslationPanel)
├── routes/
│   ├── library.rs            (LibraryPage)
│   ├── import.rs             (ImportPage)
│   ├── read.rs               (ReadPage)
│   ├── annotations.rs        (AnnotationsPage - cross-paper browser)
│   ├── vocabulary.rs         (VocabularyPage)
│   └── settings.rs           (SettingsPage)
├── selection/
│   ├── popover.rs            (SelectionPopover view)
│   └── mod.rs                (TextSelection, SelectionAction, compute_anchors)
├── storage/
│   ├── mod.rs                (AppStorage trait, StorageError)
│   ├── memory_store.rs       (InMemoryStore)
│   ├── indexed_db_store.rs   (IndexedDbStore, cfg-gated wasm32)
│   └── schema.rs             (StoredPaper, AiRecord, ReadingStats, Settings)
├── ai/
│   └── client.rs             (WebAiClient, GatewayAiClient cfg-gated wasm32)
├── import/
│   ├── mod.rs                (WebImportService, segment_text, filter_repeated_lines)
│   └── pdf_bridge.rs         (PdfTextBridge trait, StubPdfBridge, WasmPdfBridge)
├── annotations/
│   └── mod.rs                (create_annotation, save_and_reload_annotation)
├── export/
│   └── mod.rs                (ExportBundle: paper + blocks + annotations + vocabulary)
├── reading/
│   ├── navigation.rs         (NavigationAction)
│   └── session.rs            (ReadingSession)
├── state.rs                  (AppState, use_app_state)
├── util.rs                   (current_time_ms, short_id, format_relative_time)
└── app.rs                    (App component with Router)
```

### 4.2 Naming conventions
- Leptos components: `#[component] pub fn XxxPage() -> impl IntoView` — PascalCase
- Signals: `let foo: RwSignal<T>` — snake_case
- Callbacks: `let on_foo: Callback<T>` — `on_` prefix
- Event handlers: `move |ev: leptos::ev::EventType| { ... }`
- Domain types: `PaperId`, `BlockId`, `AnnotationId` — newtype wrappers

### 4.3 Reusable abstractions (all implemented)
- `SelectionPopover` — text selection action menu
- `AiPanel` — AI explanation display
- `ProgressIndicator` — block progress bar
- `BlockNavigator` — prev/next buttons
- `AnnotationRenderer` — annotation list with edit/delete
- `BlockContent` — block text with inline annotation decoration
- `TranslationPanel` — bilingual translation display

## 5. State Management Guidelines

### 5.1 Store boundaries
All state flows through `AppState` (provided via Leptos context):

```rust
pub struct AppState {
    pub store: RwSignal<InMemoryStore>,      // all domain data
    pub data_version: RwSignal<u64>,          // bumped after each mutation for reactivity
    pub current_paper_id: RwSignal<Option<PaperId>>,
    pub settings: RwSignal<Settings>,
    pub gateway_url: RwSignal<String>,
    pub gateway_token: RwSignal<Option<String>>,
}
```

- **Reactivity pattern**: readers call `self.data_version.get()` first (creates dependency), then read from store. Mutations call `self.notify()` (increments `data_version`) after writing.
- **Persistence**: mutations write to `InMemoryStore` synchronously, then `spawn_local` an async IndexedDB write.

### 5.2 Persistence
- IndexedDB is the durable store; `InMemoryStore` is the runtime cache.
- On startup, `init_persistence()` opens IndexedDB and merges all data into `InMemoryStore`.
- After each mutation, the corresponding `persist_*` method writes to IndexedDB asynchronously.
- `replace_store()` (used after import) extracts all data from the new store and persists it to IndexedDB.

## 6. Error Handling UX

### 6.1 Import failures
- Empty input: "The text is empty. Please paste some content."
- PDF extraction failure: "Failed to extract text from the PDF. It may be a scanned document."
- Storage failure: "Failed to save the paper. Please try again."
- Errors displayed inline (`.form-error` class).

### 6.2 AI failures
- Gateway returns 401: "unauthorized - check gateway token in Settings"
- Network error: error string from `gloo-net` displayed in AI panel (`.ai-error` class)
- Loading state: "Loading..." in AI panel, "Translating..." in translation panel

### 6.3 Offline behavior
- All reading, annotation, and vocabulary features work offline (InMemoryStore + IndexedDB).
- AI features fail with network error when offline (no explicit offline banner yet).

## 7. Accessibility & Keyboard Support ✅ Implemented

### Reading keyboard shortcuts
- `ArrowRight` / `PageDown` / `Space` → Next block
- `ArrowLeft` / `PageUp` → Previous block
- `F` → Toggle fullscreen
- Shortcuts are suppressed when typing in form fields (`input`, `textarea`, `select`)

### Other accessibility
- All buttons have text labels
- Annotation form is a modal overlay (`.annotation-form-overlay`)
- Delete actions require confirmation (browser `confirm()` dialog)

## 8. Performance Guidelines
- Block content renders only the current block (not all blocks)
- IndexedDB writes are async (`spawn_local`) to avoid blocking the UI
- Translation/AI requests are async with loading indicators
- CSS transitions on progress bar and hover states

## 9. CSS Theme
CSS variables defined in `:root`:
```css
--bg-primary: #fafafa;
--bg-secondary: #ffffff;
--bg-sidebar: #f5f5f5;
--text-primary: #2c2c2c;
--text-secondary: #666;
--accent: #3b82f6;
--accent-hover: #2563eb;
--border: #e5e5e5;
--error: #dc2626;
--success: #16a34a;
--shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
--radius: 8px;
```

Responsive breakpoints:
- `@media (max-width: 900px)`: reading layout and import sections switch to single column
- `@media (max-width: 600px)`: translation panel stacks vertically
