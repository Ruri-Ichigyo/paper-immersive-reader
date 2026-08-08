# MVP Acceptance Fixes Implementation Plan

**Status:** Planned
**Date:** 2026-08-08

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix three critical gaps blocking MVP browser release and address four UX polish issues, bringing the web app to full PRD §6 compliance.

**Architecture:** All changes within `crates/web-app` and minimal `crates/core` touches. No new crates, no new Rust dependencies. pdf.js is a JS-only asset.

**Tech Stack:** Rust, Leptos 0.7, wasm-bindgen, web-sys, IndexedDB, vendored pdf.js, Trunk

**Verification commands:**
```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features
```

---

## Phase 1 - Critical Fixes (block release)

### Task 1: Deploy pdf.js assets and bridge function (G1)

**Problem:** `WasmPdfBridge` calls `globalThis.pdfjs_extract_text(data: &[u8]) -> JsValue`, but `index.html` doesn't load pdf.js and no JS bridge function exists. PDF import silently fails in the browser.

**Files:**
- Modify: `crates/web-app/index.html`
- Modify: `crates/web-app/assets/pdfjs/README.md` (update instructions)
- Modify: `crates/web-app/Trunk.toml` (create if needed)

**Steps:**

1. Create `crates/web-app/Trunk.toml` with copy-file rules for pdf.js assets:

```toml
[[hooks]]
# No build hooks needed - pdf.js is pre-built

# Trunk will copy these files to the dist directory
# via data-trunk attributes in index.html
```

2. Update `crates/web-app/index.html` to load pdf.js and register the bridge:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Paper Immersive Reader</title>
    <link data-trunk rel="rust" data-cargo-no-default-features data-wasm-opt="0">
    <link data-trunk rel="css" href="styles.css">
    <link data-trunk rel="copy-file" href="assets/pdfjs/pdf.min.js">
    <link data-trunk rel="copy-file" href="assets/pdfjs/pdf.worker.min.js">
</head>
<body>
    <script src="pdf.min.js"></script>
    <script>
        // Register the pdf.js worker and expose a bridge function
        // for WasmPdfBridge to call via wasm-bindgen.
        if (window.pdfjsLib) {
            pdfjsLib.GlobalWorkerOptions.workerSrc = "pdf.worker.min.js";
        }

        /**
         * Extract text from a PDF byte array.
         * @param {Uint8Array} data - PDF file bytes
         * @returns {string | null} - Extracted text or null on failure
         */
        async function pdfjs_extract_text(data) {
            try {
                if (!window.pdfjsLib) {
                    console.error("pdf.js not loaded");
                    return null;
                }
                const doc = await pdfjsLib.getDocument({ data }).promise;
                let fullText = "";
                for (let i = 1; i <= doc.numPages; i++) {
                    const page = await doc.getPage(i);
                    const content = await page.getTextContent();
                    const pageText = content.items
                        .map(item => item.str)
                        .join(" ");
                    fullText += pageText + "\n\n";
                }
                return fullText || null;
            } catch (err) {
                console.error("PDF extraction failed:", err);
                return null;
            }
        }
        // Expose globally for wasm-bindgen extern
        globalThis.pdfjs_extract_text = pdfjs_extract_text;
    </script>
</body>
</html>
```

3. Update `crates/web-app/assets/pdfjs/README.md` to clarify that the files must be downloaded before running the app, with the exact download URL and expected filenames.

4. Update `crates/web-app/src/import/pdf_bridge.rs`: The current `WasmPdfBridge` calls `call_pdfjs_extract(data: &[u8]) -> JsValue` synchronously, but `pdfjs_extract_text` is now async. Change the extern declaration to return a `js_sys::Promise` and await it. Update `WasmPdfBridge::extract_text` to be async.

**Note:** The `PdfTextBridge::extract_text` trait method is currently sync. Two options:
- **Option A (preferred):** Keep the sync trait but have the wasm bridge call a sync wrapper that uses `js_sys::Function::call0` and converts the promise to a result synchronously. This won't work for async pdf.js.
- **Option B:** Change `PdfTextBridge::extract_text` to return `Result<String, PdfBridgeError>` but have the wasm implementation use `wasm_bindgen_futures::spawn_local` with a callback, or make the import flow async.
- **Option C (simplest):** Keep the JS bridge synchronous by using pdf.js's `getDocument` with `disableWorker: true` and awaiting in a spin-loop. Not ideal but workable for MVP.
- **Option D (recommended):** Make the JS bridge function return a string synchronously by using a blocking approach: load pdf.js with `disableWorker` and use a synchronous XHR-based approach. This is fragile.
- **Option E (best for MVP):** Change the bridge to use `JsFuture` and make `import_pdf` async on wasm32. The import route handler already runs in a sync context in the Leptos component, so wrap the async call in `wasm_bindgen_futures::spawn_local`.

**Decision:** Use Option E. Make `WasmPdfBridge::extract_text` async, update `WebImportService::import_pdf` to be async on wasm32, and update the Import page to use `spawn_local`.

**Verification:**
- `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features` passes
- `cargo test --workspace` passes (native tests use `StubPdfBridge`, unaffected)
- Manual: with pdf.js assets placed, uploading a text-based PDF in the browser produces readable blocks

---

### Task 2: Fix Paper ID collision (G2)

**Problem:** `WebImportService::next_paper_id()` returns `PaperId(format!("paper-{}", self.next_paper_counter))` starting from 1. After browser reload, IndexedDB has `paper-1`; a new import generates `paper-1` again, overwriting existing data.

**Files:**
- Modify: `crates/web-app/src/import/mod.rs`

**Steps:**

1. Remove the `next_paper_counter` field from `WebImportService`.

2. Replace `next_paper_id()` with:
```rust
fn next_paper_id(&self) -> PaperId {
    PaperId(short_id("paper"))
}
```

3. Remove the counter initialization from `new()`.

4. Similarly, fix Block ID generation in `segment_text` to use `short_id("block")` instead of `format!("block-{}", index)`. Keep `order_index` for ordering - it already exists on `Block`.

**Verification:**
- `cargo test --workspace` passes (existing tests check paper creation but not specific ID format)
- `cargo test --test web_import_paste_flow -q` passes
- `cargo test --test web_import_pdf_flow -q` passes
- Manual: import a paper, reload, import another - both papers appear in library

---

### Task 3: Fix gateway_url restoration from IndexedDB (G3)

**Problem:** `init_persistence()` in `state.rs` loads `Settings` from IndexedDB and sets `app.settings`, but does not update `app.gateway_url`. The `gateway_url` signal stays at its default `http://127.0.0.1:3000` even if the user saved a different URL.

**Files:**
- Modify: `crates/web-app/src/state.rs`

**Steps:**

1. In `init_persistence()` (wasm32 branch), after loading settings from IndexedDB, sync the gateway URL:

```rust
if let Ok(settings) = db.load_settings().await {
    app.store.update(|s| {
        s.save_settings(&settings).ok();
    });
    app.settings.set(settings.clone());
    // Sync gateway_url from persisted settings
    if let Some(ref url) = settings.ai_gateway_url {
        if !url.is_empty() {
            app.gateway_url.set(url.clone());
        }
    }
}
```

2. Also sync in the non-wasm32 `init_persistence` (no-op, but add a comment for clarity).

**Verification:**
- `cargo test --workspace` passes
- `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features` passes
- Manual: change gateway URL in Settings, reload page, open Settings - URL field shows saved value; AI requests use the saved URL

---

## Phase 2 - MVP Polish

### Task 4: Inline annotation decoration in reading view (G4)

**Problem:** APP_FLOW §3.7 requires "UI decorates text in place". Annotations are currently only listed below the block.

**Files:**
- Create: `crates/web-app/src/components/block_content.rs`
- Modify: `crates/web-app/src/components/mod.rs`
- Modify: `crates/web-app/src/routes/read.rs`
- Modify: `crates/web-app/styles.css`

**Steps:**

1. Create `BlockContent` component that:
   - Takes `block_text: String`, `annotations: Vec<Annotation>`, and `on_mouse_up: Callback<()>`
   - Filters annotations for the current block (matching `block_id`)
   - Sorts them by `start_offset`
   - Splits the block text into segments: unannotated ranges and annotated ranges
   - Renders annotated ranges with CSS classes based on `AnnotationType`:
     - `Highlight` -> `<mark class="ann-inline-highlight">`
     - `Underline` -> `<u class="ann-inline-underline">`
     - `Note` -> `<span class="ann-inline-note">`
   - Renders unannotated ranges as plain text
   - Attaches `on:mouseup` handler

2. Add `pub mod block_content;` to `components/mod.rs`.

3. Replace the `<div class="block-content" on:mouseup=on_mouse_up>{move || current_block_text()}</div>` in `read.rs` with `<BlockContent block_text=current_block_text() annotations=annotations_for_block() on_mouse_up=on_mouse_up />`.

4. Add a computed signal `annotations_for_block` that filters annotations by current block ID.

5. Add CSS styles for inline annotation decoration:
   - `.ann-inline-highlight` - yellow background
   - `.ann-inline-underline` - blue underline
   - `.ann-inline-note` - green dashed underline + title tooltip with note text

**Verification:**
- `cargo test --workspace` passes
- `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features` passes
- Manual: create a highlight annotation, see text highlighted in the block; create underline, see underlined text

---

### Task 5: Keyboard navigation for block navigation (G5)

**Problem:** APP_FLOW §4.3 requires keyboard navigation for next/previous block.

**Files:**
- Modify: `crates/web-app/src/routes/read.rs`
- Modify: `crates/web-app/styles.css` (optional - focus indicator)

**Steps:**

1. Add a `window` `keydown` event listener in the `ReadPage` component using `leptos::ev::keydown` or `window_event_listener`:

```rust
// In ReadPage, after on_next/on_previous definitions:
let key_handler = move |ev: leptos::ev::KeyboardEvent| {
    // Don't intercept when typing in form fields
    let target = ev.target();
    if let Some(el) = target {
        let tag = wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(&el);
        if let Ok(el) = tag {
            let tag_name = el.tag_name().to_lowercase();
            if tag_name == "input" || tag_name == "textarea" || tag_name == "select" {
                return;
            }
        }
    }
    match ev.key().as_str() {
        "ArrowRight" | "PageDown" | " " => {
            ev.prevent_default();
            on_next.run(());
        }
        "ArrowLeft" | "PageUp" => {
            ev.prevent_default();
            on_previous.run(());
        }
        "f" | "F" => {
            let new_fs = !is_fullscreen.get();
            is_fullscreen.set(new_fs);
            toggle_body_fullscreen(new_fs);
        }
        _ => {}
    }
};
// Register via window event listener
window_event_listener(leptos::ev::keydown, key_handler);
```

2. Use `leptos::prelude::window_event_listener` (available in Leptos 0.7) which auto-cleans up on component unmount. Add `use leptos::prelude::window_event_listener;` if not already imported.

3. Ensure the `on_cleanup` already present in the component also covers the keyboard listener (Leptos `window_event_listener` handles this automatically).

**Verification:**
- `cargo test --workspace` passes
- `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features` passes
- Manual: press Right Arrow / PageDown / Space to go to next block; Left Arrow / PageUp for previous; F to toggle fullscreen; typing in annotation form fields is not affected

---

### Task 6: Vocabulary management UI completeness (G6)

**Problem:** FR-30 requires vocabulary management. Current UI only lists items. Cannot mark as learned, add note, or tag.

**Files:**
- Modify: `crates/web-app/src/storage/mod.rs` (add `update_vocabulary` to trait)
- Modify: `crates/web-app/src/storage/memory_store.rs` (implement)
- Modify: `crates/web-app/src/storage/indexed_db_store.rs` (implement)
- Modify: `crates/web-app/src/state.rs` (add `update_vocabulary` helper)
- Modify: `crates/web-app/src/routes/vocabulary.rs` (add UI controls)
- Modify: `crates/web-app/styles.css` (action buttons, note editor)

**Steps:**

1. Add `update_vocabulary` method to `AppStorage` trait:
```rust
/// Update a vocabulary item's mutable fields (learned, note).
fn update_vocabulary(&mut self, item: &VocabularyItem) -> Result<(), StorageError>;
```

2. Implement in `InMemoryStore`:
```rust
fn update_vocabulary(&mut self, item: &VocabularyItem) -> Result<(), StorageError> {
    let key = item.term.to_lowercase();
    self.vocabulary.insert(key, item.clone());
    Ok(())
}
```

3. Implement in `IndexedDbStore` - add an async `update_vocabulary` method that puts the full item (same key as upsert uses):
```rust
pub async fn update_vocabulary(&self, item: &VocabularyItem) -> Result<(), StorageError> {
    // Similar to upsert but replaces the item entirely
    let txn = ...; // readwrite transaction on "vocabulary"
    let store = ...;
    let jv = to_json(item)?;
    let req = store.put_with_key(&jv, &JsValue::from_str(&item.id.0))?;
    await_val(&req).await?;
    Ok(())
}
```

4. Add `update_vocabulary` helper to `AppState` (similar to `upsert_vocabulary` but calls `store.update_vocabulary` and persists to IndexedDB).

5. Update `VocabularyPage` to add per-row actions:
   - "Mark Learned" / "Unmark" toggle button
   - Inline note editor (click to expand, text area + save)
   - Show note text if present
   - Show learned status with toggle

6. Add CSS for vocabulary action buttons and note editor.

**Verification:**
- `cargo test --workspace` passes
- Add a test: `tests/web_vocabulary_update.rs` - verify `update_vocabulary` on `InMemoryStore` preserves learned flag and note
- `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features` passes
- Manual: mark a word as learned, reload - status persists; add a note, reload - note persists

---

### Task 7: Header/footer noise removal in PDF import (G7)

**Problem:** FR-05 requires removing repeated header/footer noise. Current `segment_text` does a raw `split("\n\n")` with no filtering.

**Files:**
- Modify: `crates/web-app/src/import/mod.rs`

**Steps:**

1. Add a `filter_repeated_lines` function that:
   - Splits the extracted text by form-feed character `\x0c` (pdf.js page separator) if present, otherwise by `\n\n` chunks
   - For each "page", extracts the first and last non-empty lines
   - Counts occurrence of each candidate line across pages
   - Lines that appear on > 50% of pages are treated as headers/footers and removed
   - Returns cleaned text

```rust
fn filter_repeated_lines(text: &str) -> String {
    let pages: Vec<&str> = text.split('\x0c').collect();
    if pages.len() < 3 {
        // Not enough pages to detect repetition
        return text.to_string();
    }

    let mut line_counts: HashMap<String, usize> = HashMap::new();
    let threshold = pages.len() / 2;

    for page in &pages {
        let lines: Vec<&str> = page.lines().filter(|l| !l.trim().is_empty()).collect();
        if let Some(first) = lines.first() {
            *line_counts.entry(first.trim().to_string()).or_insert(0) += 1;
        }
        if let Some(last) = lines.last() {
            *line_counts.entry(last.trim().to_string()).or_insert(0) += 1;
        }
    }

    let noise_lines: std::collections::HashSet<&str> = line_counts
        .iter()
        .filter(|(_, &count)| count > threshold)
        .map(|(line, _)| line.as_str())
        .collect();

    pages
        .iter()
        .map(|page| {
            page.lines()
                .filter(|line| !noise_lines.contains(line.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\x0c\n")
}
```

2. Call `filter_repeated_lines` at the start of `segment_text` before splitting into blocks.

3. Also improve segmentation: after filtering, split on `\n\n` first; if only 1 segment results, fall back to splitting on single `\n` for lines > 20 chars (catches PDFs with single-newline paragraph breaks).

```rust
fn segment_text(paper: &Paper, text: &str) -> Result<Vec<Block>, WebImportError> {
    let cleaned = filter_repeated_lines(text);

    let mut segments: Vec<String> = cleaned
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    // Fallback: if single-newline-separated paragraphs exist
    if segments.len() <= 1 {
        segments = cleaned
            .split('\n')
            .map(str::trim)
            .filter(|s| s.len() > 20)
            .map(ToOwned::to_owned)
            .collect();
    }

    if segments.is_empty() {
        return Err(WebImportError::EmptyInput);
    }

    segments
        .into_iter()
        .enumerate()
        .map(|(index, segment)| {
            Block::try_new(
                BlockId(short_id("block")),
                paper.id.clone(),
                index as u32,
                segment,
                None,
            )
            .map_err(|_| WebImportError::ExtractionFailed)
        })
        .collect()
}
```

4. Need to add `use std::collections::{HashMap, HashSet};` at the top of `import/mod.rs`.

**Verification:**
- `cargo test --workspace` passes (existing segmentation tests may need updating if they expect `block-{index}` IDs - update to match new ID format from Task 2)
- Add a test: `tests/web_import_header_footer_filter.rs` - verify that repeated header/footer lines are removed from the block output
- `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features` passes

---

## Phase 3 - Feature Completeness

### Task 8: Paragraph-level translation (G8 / FR-14)

**Problem:** FR-14 requires paragraph translation. Only selection-level translation is available.

**Files:**
- Modify: `crates/web-app/src/routes/read.rs`
- Modify: `crates/web-app/src/components/block_navigator.rs` (add translate button)
- Modify: `crates/web-app/styles.css`

**Steps:**

1. Add a "Translate Block" button next to the block navigator. When clicked, it sends the entire current block text to the translation flow (same as selection translate, but with the full block text).

2. Add a `translate_block` callback in `ReadPage` that:
   - Gets the current block text
   - Sets `translation_source`, clears `translation_result`, sets `translation_loading`
   - Calls `GatewayAiClient.translate_async` with the full block text
   - On success, sets `translation_result` and saves an AI record
   - Does NOT auto-collect vocabulary (only single-word lookups do that)

3. Add the button to the reading toolbar or block navigator area:
```rust
<button
    class="btn btn-secondary btn-translate-block"
    on:click=move |_| on_translate_block.run(())
    disabled=move || translation_loading.get()
>
    "Translate Block"
</button>
```

4. Add CSS for the translate-block button.

**Verification:**
- `cargo test --workspace` passes
- `cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features` passes
- Manual: click "Translate Block" - full block text appears in translation panel with translation

---

### Task 9: Improved segmentation fallback (G9 / partial FR-12)

**Problem:** Current `split("\n\n")` misses paragraphs separated by single newlines (common in PDF extraction). This is already partially addressed in Task 7's segmentation improvement.

**Files:**
- Already handled in Task 7's `segment_text` changes.

**Note:** This task is a sub-item of Task 7. If Task 7 is implemented, G9 is resolved. List it separately for tracking but implement as part of Task 7.

**Verification:**
- Same as Task 7.

---

## Task Dependency Graph

```
Task 1 (pdf.js)  ──────────── independent
Task 2 (Paper ID) ─────────── independent (but Task 7 changes same file)
Task 3 (gateway URL) ──────── independent
Task 4 (inline annotations) ── independent
Task 5 (keyboard nav) ─────── independent
Task 6 (vocabulary UI) ────── independent
Task 7 (header/footer + segmentation) ── depends on Task 2 (same file, ID format)
Task 8 (paragraph translate) ── independent (but benefits from Task 4's BlockContent)
Task 9 ── merged into Task 7
```

**Recommended execution order:** Tasks 1, 2, 3 (parallel, Phase 1) → Task 7 (after 2) → Tasks 4, 5, 6 (parallel, Phase 2) → Task 8 (Phase 3)

## Final Verification (after all tasks)

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build -p paper-immersive-reader-web-app --target wasm32-unknown-unknown --no-default-features
```

All four gates must pass. Additionally:
- Update `progress.txt` with milestone completion
- Update `docs/plans/INDEX.md` to mark topic as Done
- Archive plan files to `docs/plans/archive/`
