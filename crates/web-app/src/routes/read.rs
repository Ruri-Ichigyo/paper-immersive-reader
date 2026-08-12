use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use wasm_bindgen::JsCast;

use paper_immersive_reader_core::domain::{AnnotationType, PaperId, VocabularyId, VocabularyItem};

use crate::components::ai_panel::AiPanel;
use crate::components::annotation_renderer::AnnotationRenderer;
use crate::components::block_content::BlockContent;
use crate::components::block_navigator::BlockNavigator;
use crate::components::progress_indicator::ProgressIndicator;
use crate::components::translation_panel::TranslationPanel;
use crate::selection::popover::SelectionPopover;
use crate::selection::{compute_anchors, SelectionAction, TextSelection};
use crate::state::use_app_state;
use crate::storage::schema::AiRecord;
use crate::util::{current_time_ms, short_id};

#[component]
pub fn ReadPage() -> impl IntoView {
    let app = use_app_state();
    let params = use_params_map();

    let paper_id = move || params.get().get("paper_id").map(|s| PaperId(s.clone()));

    // Reading session state
    let blocks = RwSignal::new(Vec::new());
    let current_index = RwSignal::new(0u32);
    let total_blocks = RwSignal::new(0u32);

    // Selection state
    let selection = RwSignal::new(Option::<TextSelection>::None);
    // Selection captured for the annotation form (kept separate from the
    // popover selection so it survives after the popover is dismissed).
    let pending_annotation_selection = RwSignal::new(Option::<TextSelection>::None);

    // Translation state
    let translation_source = RwSignal::new(Option::<String>::None);
    let translation_result = RwSignal::new(Option::<String>::None);
    let translation_loading = RwSignal::new(false);
    let translation_error = RwSignal::new(Option::<String>::None);

    // AI panel state
    let ai_content = RwSignal::new(Option::<String>::None);
    let ai_panel_type = RwSignal::new("AI Assistant".to_string());
    let ai_loading = RwSignal::new(false);
    let ai_error = RwSignal::new(Option::<String>::None);

    // Annotation form state
    let show_annotation_form = RwSignal::new(false);
    let annotation_type = RwSignal::new(AnnotationType::Highlight);
    let annotation_note = RwSignal::new(String::new());
    let annotation_tag = RwSignal::new(String::new());

    // Fullscreen state
    let is_fullscreen = RwSignal::new(false);

    // Sidebar collapse state (persisted in Settings)
    let sidebar_collapsed = RwSignal::new(app.settings().sidebar_collapsed);

    const DEFAULT_TAGS: &[&str] = &["重点", "疑问", "术语", "结论", "复习"];

    // --- Reading session time tracking (FR-32) ---
    let session_started: RwSignal<Option<u64>> = RwSignal::new(None);
    let session_paper_id = paper_id();

    // Remove fullscreen body class and persist reading time on unmount.
    {
        on_cleanup(move || {
            toggle_body_fullscreen(false);
            if let Some(ref pid) = session_paper_id {
                if let Some(started) = session_started.get_untracked() {
                    let elapsed = current_time_ms().saturating_sub(started);
                    if elapsed >= 1_000 {
                        let existing = app.reading_stats(pid).unwrap_or_else(|| {
                            crate::storage::schema::ReadingStats::new(pid.clone())
                        });
                        let mut stats = existing;
                        stats.total_reading_ms += elapsed;
                        stats.session_count += 1;
                        stats.session_started_ms = None;
                        app.save_reading_stats(&stats);
                    }
                }
            }
        });
    }

    // Load session on mount
    {
        if let Some(pid) = paper_id() {
            let b = app.blocks(&pid);
            let progress = app.progress(&pid);
            let restored = progress.map(|p| p.current_block_index).unwrap_or(0);
            let idx = if (restored as usize) < b.len() {
                restored
            } else {
                0
            };
            blocks.set(b);
            current_index.set(idx);
            total_blocks.set(blocks.get().len() as u32);

            // Start reading session timer
            let now = current_time_ms();
            session_started.set(Some(now));
            let mut stats = app
                .reading_stats(&pid)
                .unwrap_or_else(|| crate::storage::schema::ReadingStats::new(pid.clone()));
            stats.session_started_ms = Some(now);
            app.save_reading_stats(&stats);
        }
    }

    let current_block_text = move || {
        let b = blocks.get();
        let i = current_index.get() as usize;
        b.get(i).map(|block| block.text.clone()).unwrap_or_default()
    };

    let current_block_id = move || {
        let b = blocks.get();
        let i = current_index.get() as usize;
        b.get(i).map(|block| block.id.clone())
    };

    let progress_percent = move || {
        let total = total_blocks.get();
        if total == 0 {
            0
        } else {
            ((current_index.get() + 1) as u64 * 100 / total as u64) as u32
        }
    };

    let can_go_prev = move || current_index.get() > 0;
    let can_go_next = move || current_index.get() + 1 < total_blocks.get();

    // Navigation handlers
    let on_previous = Callback::new(move |_| {
        if current_index.get() > 0 {
            let new_idx = current_index.get() - 1;
            current_index.set(new_idx);
            persist_progress(&app, paper_id(), new_idx, &blocks.get());
        }
    });

    let on_next = Callback::new(move |_| {
        if current_index.get() + 1 < total_blocks.get() {
            let new_idx = current_index.get() + 1;
            current_index.set(new_idx);
            persist_progress(&app, paper_id(), new_idx, &blocks.get());
        }
    });

    // --- Keyboard navigation (Task 5) ---
    {
        let on_prev = on_previous;
        let on_next_k = on_next;
        let fs = is_fullscreen;
        window_event_listener(leptos::ev::keydown, move |ev: leptos::ev::KeyboardEvent| {
            // Don't intercept when typing in form fields
            if let Some(target) = ev.target() {
                if let Ok(el) = target.dyn_into::<web_sys::HtmlElement>() {
                    let tag = el.tag_name().to_lowercase();
                    if tag == "input" || tag == "textarea" || tag == "select" {
                        return;
                    }
                }
            }
            match ev.key().as_str() {
                "ArrowRight" | "PageDown" | " " => {
                    ev.prevent_default();
                    on_next_k.run(());
                }
                "ArrowLeft" | "PageUp" => {
                    ev.prevent_default();
                    on_prev.run(());
                }
                "f" | "F" => {
                    let new_fs = !fs.get();
                    fs.set(new_fs);
                    toggle_body_fullscreen(new_fs);
                }
                _ => {}
            }
        });
    }

    // Text selection handling — defer to next frame so the browser finalises
    // the selection before we read it (avoids empty-selection race on mouseup).
    let on_mouse_up = Callback::new(move |_| {
        let block_id_for_sel = current_block_id();
        let block_text_for_sel = current_block_text();
        let selection_for_sel = selection;

        // Capture the selection in the next animation frame
        wasm_bindgen_futures::spawn_local(async move {
            // yield one frame
            let _ = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(
                &mut |resolve, _| {
                    let window = web_sys::window().unwrap();
                    let _ = window.request_animation_frame(&resolve);
                    drop(resolve);
                },
            ))
            .await;

            let window = match web_sys::window() {
                Some(w) => w,
                None => return,
            };
            let sel = match window.get_selection() {
                Ok(Some(s)) => s,
                _ => return,
            };
            let text_js = sel.to_string();
            let text = text_js.as_string().unwrap_or_default();
            if text.trim().is_empty() {
                selection_for_sel.set(None);
                return;
            }
            let block_id = match block_id_for_sel {
                Some(id) => id,
                None => return,
            };
            let block_text = block_text_for_sel;
            let (start, end) = find_offsets(&block_text, &text).unwrap_or((0, 0));
            selection_for_sel.set(Some(TextSelection {
                block_id,
                text,
                start_offset: start,
                end_offset: end,
            }));
        });
    });

    // Selection action handler
    let on_selection_action = Callback::new({
        move |(action, sel): (SelectionAction, TextSelection)| {
            let pid = match paper_id() {
                Some(p) => p,
                None => return,
            };
            let block_id = sel.block_id.clone();
            let selected_text = sel.text.clone();
            let block_text = current_block_text();
            let (prefix, suffix) = compute_anchors(&block_text, &sel);
            #[allow(unused_variables)]
            let context = prefix.or(suffix);

            match &action {
                SelectionAction::Translate => {
                    translation_source.set(Some(selected_text.clone()));
                    translation_result.set(None);
                    translation_error.set(None);
                    translation_loading.set(true);

                    // Auto-collect vocabulary for single-word lookups
                    if selected_text.split_whitespace().count() == 1 {
                        let now = current_time_ms();
                        let term_lower = selected_text.to_lowercase();
                        let item = VocabularyItem {
                            id: VocabularyId(short_id("vocab")),
                            term: selected_text.clone(),
                            term_lower,
                            paper_id_first_seen: pid.clone(),
                            block_id_first_seen: block_id.clone(),
                            lookup_count: 1,
                            first_seen_at_ms: now,
                            last_seen_at_ms: now,
                            note: None,
                            learned: false,
                        };
                        app.upsert_vocabulary(&item);
                    }

                    #[cfg(target_arch = "wasm32")]
                    {
                        let gateway_url = app.gateway_url.get();
                        let gateway_token = app.gateway_token.get();
                        let client =
                            crate::ai::client::GatewayAiClient::new(gateway_url, gateway_token);
                        let text_owned = selected_text.clone();
                        let pid_clone = pid.clone();
                        let block_id_clone = block_id.clone();
                        let app = app;
                        wasm_bindgen_futures::spawn_local(async move {
                            match client
                                .translate_async(&text_owned, "zh", context.as_deref())
                                .await
                            {
                                Ok(result) => {
                                    translation_result.set(Some(result.clone()));
                                    translation_loading.set(false);

                                    let record = AiRecord {
                                        id: short_id("ai"),
                                        paper_id: pid_clone,
                                        block_id: block_id_clone,
                                        record_type: "translate".to_string(),
                                        input: text_owned,
                                        output: result,
                                        language: "zh".to_string(),
                                        created_at_ms: current_time_ms(),
                                    };
                                    app.save_ai_record(&record);
                                }
                                Err(e) => {
                                    translation_error.set(Some(e));
                                    translation_loading.set(false);
                                }
                            }
                        });
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let result = format!("[zh] {}", selected_text);
                        translation_result.set(Some(result.clone()));
                        translation_loading.set(false);

                        let record = AiRecord {
                            id: short_id("ai"),
                            paper_id: pid,
                            block_id,
                            record_type: "translate".to_string(),
                            input: selected_text,
                            output: result,
                            language: "zh".to_string(),
                            created_at_ms: current_time_ms(),
                        };
                        app.save_ai_record(&record);
                    }
                }
                SelectionAction::ExplainTerm | SelectionAction::ExplainSentence => {
                    let is_term = matches!(action, SelectionAction::ExplainTerm);
                    let et = if is_term { "term" } else { "sentence" };
                    let language = app.settings().explanation_language.clone();

                    ai_panel_type.set(if is_term {
                        "Term Explanation".to_string()
                    } else {
                        "Sentence Explanation".to_string()
                    });
                    ai_loading.set(true);
                    ai_error.set(None);
                    ai_content.set(None);

                    #[cfg(target_arch = "wasm32")]
                    {
                        let gateway_url = app.gateway_url.get();
                        let gateway_token = app.gateway_token.get();
                        let client =
                            crate::ai::client::GatewayAiClient::new(gateway_url, gateway_token);
                        let text_owned = selected_text.clone();
                        let et_owned = et.to_string();
                        let lang_owned = language.clone();
                        let pid_clone = pid.clone();
                        let block_id_clone = block_id.clone();
                        let app = app;
                        wasm_bindgen_futures::spawn_local(async move {
                            match client
                                .explain_async(
                                    &text_owned,
                                    &et_owned,
                                    &lang_owned,
                                    context.as_deref(),
                                )
                                .await
                            {
                                Ok(result) => {
                                    let display = result.output;
                                    ai_content.set(Some(display.clone()));
                                    ai_loading.set(false);

                                    let record = AiRecord {
                                        id: short_id("ai"),
                                        paper_id: pid_clone,
                                        block_id: block_id_clone,
                                        record_type: format!("explain_{}", et_owned),
                                        input: text_owned,
                                        output: display,
                                        language: lang_owned,
                                        created_at_ms: current_time_ms(),
                                    };
                                    app.save_ai_record(&record);
                                }
                                Err(e) => {
                                    ai_error.set(Some(e));
                                    ai_loading.set(false);
                                }
                            }
                        });
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let output = format!("[{} {} explanation] {}", language, et, selected_text);
                        ai_content.set(Some(output.clone()));
                        ai_loading.set(false);

                        let record = AiRecord {
                            id: short_id("ai"),
                            paper_id: pid,
                            block_id,
                            record_type: format!("explain_{}", et),
                            input: selected_text,
                            output,
                            language,
                            created_at_ms: current_time_ms(),
                        };
                        app.save_ai_record(&record);
                    }
                }
                SelectionAction::Annotate => {
                    // Preserve the selection for the annotation form.
                    pending_annotation_selection.set(Some(sel.clone()));
                    show_annotation_form.set(true);
                }
            }
            // Clear the popover selection for all actions.  For Annotate,
            // the selection has already been saved in
            // pending_annotation_selection above.
            selection.set(None);
        }
    });

    let on_dismiss = Callback::new(move |_| {
        selection.set(None);
    });

    // --- Paragraph-level translation (Task 8) ---
    let on_translate_block = Callback::new(move |_| {
        let pid = match paper_id() {
            Some(p) => p,
            None => return,
        };
        let block_id = match current_block_id() {
            Some(b) => b,
            None => return,
        };
        let block_text = current_block_text();
        if block_text.trim().is_empty() {
            return;
        }

        translation_source.set(Some(block_text.clone()));
        translation_result.set(None);
        translation_error.set(None);
        translation_loading.set(true);

        #[cfg(target_arch = "wasm32")]
        {
            let gateway_url = app.gateway_url.get();
            let gateway_token = app.gateway_token.get();
            let client = crate::ai::client::GatewayAiClient::new(gateway_url, gateway_token);
            let text_owned = block_text.clone();
            let pid_clone = pid.clone();
            let block_id_clone = block_id.clone();
            let app = app;
            wasm_bindgen_futures::spawn_local(async move {
                match client.translate_async(&text_owned, "zh", None).await {
                    Ok(result) => {
                        translation_result.set(Some(result.clone()));
                        translation_loading.set(false);

                        let record = AiRecord {
                            id: short_id("ai"),
                            paper_id: pid_clone,
                            block_id: block_id_clone,
                            record_type: "translate_block".to_string(),
                            input: text_owned,
                            output: result,
                            language: "zh".to_string(),
                            created_at_ms: current_time_ms(),
                        };
                        app.save_ai_record(&record);
                    }
                    Err(e) => {
                        translation_error.set(Some(e));
                        translation_loading.set(false);
                    }
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let result = format!("[zh] {}", block_text);
            translation_result.set(Some(result.clone()));
            translation_loading.set(false);

            let record = AiRecord {
                id: short_id("ai"),
                paper_id: pid,
                block_id,
                record_type: "translate_block".to_string(),
                input: block_text,
                output: result,
                language: "zh".to_string(),
                created_at_ms: current_time_ms(),
            };
            app.save_ai_record(&record);
        }
    });

    // Annotation form submit
    let on_annotate_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let pid = match paper_id() {
            Some(p) => p,
            None => return,
        };
        let block_id = match current_block_id() {
            Some(b) => b,
            None => return,
        };
        let block_text = current_block_text();
        let sel = match pending_annotation_selection.get() {
            Some(s) => s,
            None => TextSelection {
                block_id: block_id.clone(),
                text: block_text.clone(),
                start_offset: 0,
                end_offset: block_text.chars().count() as u32,
            },
        };

        let tag = annotation_tag.get();
        let input = crate::annotations::AnnotationInput {
            annotation_type: annotation_type.get(),
            note: {
                let n = annotation_note.get();
                if n.trim().is_empty() {
                    None
                } else {
                    Some(n)
                }
            },
            tags: if tag.is_empty() {
                Vec::new()
            } else {
                vec![tag]
            },
        };

        let ann = crate::annotations::create_annotation(pid, block_id, &block_text, &sel, input);
        app.save_annotation(&ann);
        show_annotation_form.set(false);
        pending_annotation_selection.set(None);
        annotation_note.set(String::new());
        annotation_tag.set(String::new());
    };

    let on_annotate_cancel = move |_| {
        show_annotation_form.set(false);
        pending_annotation_selection.set(None);
        annotation_note.set(String::new());
        annotation_tag.set(String::new());
    };

    let pid_clone = paper_id();
    let pid_for_annotations = pid_clone.clone();
    let annotations_list = move || {
        app.data_version.get();
        if let Some(ref pid) = pid_clone {
            app.annotations(pid)
        } else {
            Vec::new()
        }
    };

    // Annotations for the current block (for inline decoration)
    let current_block_annotations = move || {
        app.data_version.get();
        let bid = current_block_id();
        if let Some(ref pid) = pid_for_annotations {
            app.annotations(pid)
                .into_iter()
                .filter(|a| bid.as_ref() == Some(&a.block_id))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    };

    view! {
        <div class="route-read">
            {move || {
                let total = total_blocks.get();
                if total == 0 {
                    view! {
                        <div class="empty-state">
                            <p>"Paper not found or has no readable content."</p>
                            <a href="/library" class="btn btn-primary">"Back to Library"</a>
                        </div>
                    }.into_any()
                } else {
                    let fs = is_fullscreen;
                    view! {
                        <div class=move || {
                            let base = if fs.get() { "reading-layout reading-fullscreen" } else { "reading-layout" };
                            if sidebar_collapsed.get() {
                                format!("{} sidebar-collapsed", base)
                            } else {
                                base.to_string()
                            }
                        }>
                            // Main reading column
                            <div class="reading-main">
                                <div class="reading-toolbar">
                                    <ProgressIndicator
                                        current_index=current_index.get()
                                        total_blocks=total_blocks.get()
                                        progress_percent=progress_percent()
                                    />
                                    <div class="reading-toolbar-actions">
                                        <button
                                            class="btn btn-secondary btn-translate-block"
                                            on:click=move |_| on_translate_block.run(())
                                            disabled=move || translation_loading.get()
                                        >
                                            "Translate Block"
                                        </button>
                                        <button
                                            class="btn btn-secondary btn-sidebar-toggle"
                                            on:click=move |_| {
                                                let collapsed = !sidebar_collapsed.get();
                                                sidebar_collapsed.set(collapsed);
                                                let mut s = app.settings();
                                                s.sidebar_collapsed = collapsed;
                                                app.save_settings(&s);
                                            }
                                        >
                                            {move || if sidebar_collapsed.get() { "Show Panel" } else { "Hide Panel" }}
                                        </button>
                                        <button
                                            class="btn btn-secondary btn-fullscreen"
                                            on:click=move |_| {
                                                let new_fs = !fs.get();
                                                fs.set(new_fs);
                                                toggle_body_fullscreen(new_fs);
                                            }
                                        >
                                            {move || if fs.get() { "Exit Fullscreen" } else { "Fullscreen" }}
                                        </button>
                                    </div>
                                </div>

                                <BlockContent
                                    block_text=current_block_text()
                                    block_id=current_block_id()
                                    annotations=current_block_annotations()
                                    on_mouse_up=on_mouse_up
                                />

                                <BlockNavigator
                                    can_go_previous=can_go_prev()
                                    can_go_next=can_go_next()
                                    on_previous=on_previous
                                    on_next=on_next
                                />

                                // Selection popover
                                <SelectionPopover
                                    selection=selection.get()
                                    on_action=on_selection_action
                                    on_dismiss=on_dismiss
                                />

                                // Annotation form
                                {move || {
                                    if show_annotation_form.get() {
                                        view! {
                                            <div class="annotation-form-overlay">
                                                <form class="annotation-form" on:submit=on_annotate_submit>
                                                    <h3>"Create Annotation"</h3>
                                                    <div class="form-field">
                                                        <label>"Type"</label>
                                                        <select
                                                            class="text-input"
                                                            on:change=move |e| {
                                                                let val = event_target_value(&e);
                                                                annotation_type.set(match val.as_str() {
                                                                    "underline" => AnnotationType::Underline,
                                                                    "note" => AnnotationType::Note,
                                                                    _ => AnnotationType::Highlight,
                                                                });
                                                            }
                                                        >
                                                            <option value="highlight">"Highlight"</option>
                                                            <option value="underline">"Underline"</option>
                                                            <option value="note">"Note"</option>
                                                        </select>
                                                    </div>
                                                    <div class="form-field">
                                                        <label>"Note (optional)"</label>
                                                        <textarea
                                                            class="text-area"
                                                            rows="3"
                                                            placeholder="Add a note…"
                                                            prop:value=annotation_note
                                                            on:input=move |e| annotation_note.set(event_target_value(&e))
                                                        ></textarea>
                                                    </div>
                                                    <div class="form-field">
                                                        <label>"Tag (optional)"</label>
                                                        <select
                                                            class="text-input"
                                                            on:change=move |e| annotation_tag.set(event_target_value(&e))
                                                        >
                                                            <option value="">"- None -"</option>
                                                            {DEFAULT_TAGS.iter().map(|t| view! {
                                                                <option value=*t>{*t}</option>
                                                            }).collect::<Vec<_>>()}
                                                            {app.settings().custom_tags.iter().map(|t| view! {
                                                                <option value=t.clone()>{t.clone()}</option>
                                                            }).collect::<Vec<_>>()}
                                                        </select>
                                                    </div>
                                                    <div class="form-actions">
                                                        <button type="submit" class="btn btn-primary">"Save"</button>
                                                        <button
                                                            type="button"
                                                            class="btn btn-secondary"
                                                            on:click=on_annotate_cancel
                                                        >
                                                            "Cancel"
                                                        </button>
                                                    </div>
                                                </form>
                                            </div>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }
                                }}

                                // Annotations for this paper
                                <div class="annotations-section">
                                    <h3>"Annotations"</h3>
                                    <AnnotationRenderer
                                        annotations=annotations_list()
                                        on_delete=Callback::new(move |id: paper_immersive_reader_core::domain::AnnotationId| {
                                            app.delete_annotation(&id);
                                        })
                                        on_update=Callback::new(move |ann: paper_immersive_reader_core::domain::Annotation| {
                                            app.update_annotation(&ann);
                                        })
                                    />
                                </div>
                            </div>

                            // Side panel: translation + AI
                            <div class="reading-sidebar">
                                <TranslationPanel
                                    source_text=translation_source.get()
                                    translation=translation_result.get()
                                    language="zh".to_string()
                                    is_loading=translation_loading.get()
                                    error=translation_error.get()
                                />
                                <AiPanel
                                    content=ai_content.get()
                                    panel_type=ai_panel_type.get()
                                    is_loading=ai_loading.get()
                                    error=ai_error.get()
                                />
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// Toggle the `reading-fullscreen-active` class on `<body>` so CSS can
/// hide the app navigation bar during fullscreen reading.
fn toggle_body_fullscreen(active: bool) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(body) = document.body() {
                if active {
                    body.class_list().add_1("reading-fullscreen-active").ok();
                } else {
                    body.class_list().remove_1("reading-fullscreen-active").ok();
                }
            }
        }
    }
}

fn persist_progress(
    app: &crate::state::AppState,
    pid: Option<PaperId>,
    idx: u32,
    blocks: &[paper_immersive_reader_core::domain::Block],
) {
    if let Some(pid) = pid {
        let block_id = blocks.get(idx as usize).map(|b| b.id.clone());
        let progress = paper_immersive_reader_core::domain::ReadingProgress {
            paper_id: pid,
            current_block_index: idx,
            current_block_id: block_id,
            updated_at_ms: current_time_ms(),
        };
        app.save_progress(&progress);
    }
}

/// Find the character offsets of `selected` within `block_text`.
///
/// Returns `None` when the text cannot be found (e.g. due to browser
/// whitespace normalisation or cross-node selections).
fn find_offsets(block_text: &str, selected: &str) -> Option<(u32, u32)> {
    // 1. Exact match
    if let Some(pos) = block_text.find(selected) {
        let start = block_text[..pos].chars().count() as u32;
        let end = start + selected.chars().count() as u32;
        return Some((start, end));
    }
    // 2. Prefix search — look for first 20 chars of selected in block_text.
    //    This handles whitespace normalisation and cross-node selections
    //    where extra spaces are inserted.
    let prefix: String = selected.chars().take(20).collect();
    if prefix.chars().count() >= 4 {
        if let Some(pos) = block_text.find(&prefix) {
            let start = block_text[..pos].chars().count() as u32;
            let end = start + selected.chars().count() as u32;
            return Some((start, end));
        }
    }
    // 3. Whitespace-normalised match — try to find the first word of the
    //    selected text in the block and use that as the start.
    if let Some(first_word) = selected.split_whitespace().next() {
        if first_word.len() >= 3 {
            if let Some(pos) = block_text.find(first_word) {
                let start = block_text[..pos].chars().count() as u32;
                let end = start + selected.chars().count() as u32;
                return Some((start, end));
            }
        }
    }
    // 4. Complete failure — return None; caller produces (0, 0) and
    //    the annotation is saved as Unanchored.
    None
}
