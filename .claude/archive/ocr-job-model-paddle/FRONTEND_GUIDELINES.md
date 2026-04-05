# FRONTEND_GUIDELINES — Paper Immersive Reader (Web)

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
- Reading (dedicated per paper)
- Vocabulary (Wordbook)
- Annotations/Notes
- Settings

Navigation rules:
- Reading page must be accessible by opening a paper; it is not a global tab unless user is currently reading.
- Fullscreen reading hides global navigation; provide a single exit control.

### 1.2 Route map (suggested)
- `/` — Library
- `/import` — Import Wizard
- `/paper/:paperId` — Paper Details (optional, can be merged into reading)
- `/read/:paperId` — Immersive Reading Page
- `/vocabulary` — Wordbook
- `/annotations` — Annotations browser
- `/settings` — Settings

## 2. Core Layouts

### 2.1 Library Page
Primary regions:
- Header: app title, import button, search
- Main list: papers
- Right-side quick info panel (optional): selected paper metadata, progress, last read

Paper list item must show:
- title (or placeholder "Untitled" until metadata extraction)
- import source (pdf/paste/url/doi/zotero)
- import time
- reading progress
- last opened time
- status badge (`READY_TEXT`, `READY_OCR`, `FAILED`)

### 2.2 Import Wizard
Step-based UI:
1) Choose import method
2) Provide input (file upload / text box / URL / DOI / Zotero connect)
3) Import processing progress (staged)
4) Success route to reading

Progress stages must be explicit:
- Extracting text
- Cleaning header/footer
- Segmenting paragraphs
- AI chunking
- OCR (if triggered)

### 2.3 Immersive Reading Page (核心)

#### Layout
- Main content column: current block
- Right side panel: AI assistant + translation results + context
- Bottom navigation bar: Previous/Next block + progress indicator

Fullscreen mode:
- hide global nav and library UI
- keep:
  - minimal top bar (paper title, exit, fullscreen toggle)
  - bottom nav
  - AI panel (toggle)

#### Required UI elements
- Progress indicator:
  - percentage
  - block index (e.g., 12/240)
  - optional time spent
- Block navigation:
  - `Previous block`
  - `Next block`
  - keyboard shortcuts
- Selection popover:
  - Translate
  - Explain (term/sentence)
  - Annotate
- Annotation toolbar:
  - highlight
  - underline
  - note
  - tag selector

#### Interaction rules
- `Next block` persists reading position immediately.
- Selection popover must appear within 150ms after mouse-up.
- When AI request is running:
  - show streaming output if supported
  - allow cancel
- AI panel can be collapsed; collapsed state persists per user.

### 2.4 Vocabulary Page
List + detail pattern:
- Left: searchable list
- Right: details for selected word (translations history, notes, occurrences)

Vocabulary item fields to display:
- word/phrase
- lookup count
- first seen paper
- last seen time
- learned status

### 2.5 Annotations Page
Filter + list + export:
- Filters: paper, tag, type, date range
- List: annotation preview + jump to location
- Export action (P1)

## 3. Readability and Typography

### 3.1 English reading typography
- Use a readable serif for body blocks (optional) and high-contrast layout.
- Maintain line length ~60–80 characters.
- Use generous line-height (1.6+).

### 3.2 Bilingual layout rules (FR-15)
- Translation display is **English left, Chinese right**.
- For small screens, stack vertically with explicit labels.

### 3.3 Highlighting and emphasis
- Highlight color must remain readable on white and dark mode.
- Underline should not collide with descenders; use CSS `text-decoration-thickness` and `text-underline-offset`.

## 4. Component Conventions

### 4.1 Component directory structure (suggested)
- `components/ui/*` — primitive UI components (buttons, dialogs, popovers)
- `components/reading/*` — reading-specific components
- `components/import/*` — import wizard components
- `components/annotations/*`
- `components/vocabulary/*`

### 4.2 Naming
- React components: `PascalCase`
- Hooks: `useXxx`
- State stores: `useXxxStore`

### 4.3 Reusable abstractions
Must standardize the following cross-cutting components:
- `SelectionPopover`
- `AIPanel`
- `ProgressIndicator`
- `BlockNavigator`
- `AnnotationRenderer`
- `ImportProgress`

## 5. State Management Guidelines

### 5.1 Store boundaries
- Reading session store (paperId, currentBlockIndex, ui panel states)
- Import store (current step, job status)
- Settings store (language, tags, sync config)

### 5.2 Persistence
- Persist only stable UI preferences (panel collapsed, fullscreen default).
- Persist domain state (progress, annotations, vocabulary) in IndexedDB via a data layer.

## 6. Error Handling UX

### 6.1 Import failures
- If parse fails: show error code + recommended action
- Provide retry
- If likely scanned: recommend OCR

### 6.2 Sync failures
- Show last sync time
- Show retry with backoff
- Show conflicts list if any

### 6.3 AI failures
- If offline: show offline banner and disable AI actions
- If rate-limited: show wait time

## 7. Accessibility & Keyboard Support
- Reading:
  - `J`/`K` or arrow keys for next/previous block
  - `F` toggle fullscreen
  - `Esc` exit fullscreen
- All popovers/dialogs must be keyboard navigable.
- Provide sufficient contrast for highlights.

## 8. Performance Guidelines
- Lazy-load PDF rendering and AI panel components.
- Use web workers for extraction/OCR.
- Prefetch next N blocks in reading view to avoid UI stalls.

## 9. Telemetry (local-first)
Local counters (no remote analytics by default):
- reading time per paper
- OCR fallback rate
- import failure reasons
- sync last success time

Export telemetry only if user opts in (future).
