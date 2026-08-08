use leptos::prelude::*;

use crate::state::use_app_state;
use crate::util::format_relative_time;

#[component]
pub fn VocabularyPage() -> impl IntoView {
    let app = use_app_state();

    // Track which item's note editor is expanded
    let editing_note_id = RwSignal::new(Option::<String>::None);
    let note_draft = RwSignal::new(String::new());

    // Search / filter
    let search_query = RwSignal::new(String::new());
    let filter_learned = RwSignal::new(None::<bool>); // None = all, Some(true) = learned, Some(false) = learning

    let filtered_items = move || {
        let mut v = app.vocabulary();
        let q = search_query.get().to_lowercase();
        let learned_filter = filter_learned.get();
        if !q.is_empty() {
            v.retain(|item| {
                item.term.to_lowercase().contains(&q)
                    || item
                        .note
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
            });
        }
        if let Some(want_learned) = learned_filter {
            v.retain(|item| item.learned == want_learned);
        }
        v.sort_by(|a, b| b.last_seen_at_ms.cmp(&a.last_seen_at_ms));
        v
    };

    view! {
        <div class="route-vocabulary">
            <div class="page-header">
                <h1>"Vocabulary"</h1>
            </div>

            <div class="vocab-filters">
                <input
                    type="text"
                    class="text-input vocab-search"
                    placeholder="Search terms or notes…"
                    prop:value=move || search_query.get()
                    on:input=move |e| search_query.set(event_target_value(&e))
                />
                <div class="vocab-filter-buttons">
                    <button
                        class=move || {
                            if filter_learned.get().is_none() { "vocab-btn vocab-btn-active" } else { "vocab-btn" }
                        }
                        on:click=move |_| filter_learned.set(None)
                    >
                        "All"
                    </button>
                    <button
                        class=move || {
                            if filter_learned.get() == Some(false) { "vocab-btn vocab-btn-active" } else { "vocab-btn" }
                        }
                        on:click=move |_| filter_learned.set(Some(false))
                    >
                        "Learning"
                    </button>
                    <button
                        class=move || {
                            if filter_learned.get() == Some(true) { "vocab-btn vocab-btn-active" } else { "vocab-btn" }
                        }
                        on:click=move |_| filter_learned.set(Some(true))
                    >
                        "Learned"
                    </button>
                </div>
            </div>

            {move || {
                let v = filtered_items();
                if v.is_empty() {
                    let total = app.vocabulary().len();
                    if total == 0 {
                        view! {
                            <div class="empty-state">
                                <p>"No vocabulary items yet. Translate words while reading to collect them here."</p>
                                <a href="/library" class="btn btn-primary">"Browse Papers"</a>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="empty-state">
                                <p>"No vocabulary items match your search."</p>
                                <button
                                    class="btn btn-secondary"
                                    on:click=move |_| {
                                        search_query.set(String::new());
                                        filter_learned.set(None);
                                    }
                                >
                                    "Clear Filters"
                                </button>
                            </div>
                        }.into_any()
                    }
                } else {
                    view! {
                        <table class="vocabulary-table">
                            <thead>
                                <tr>
                                    <th>"Term"</th>
                                    <th>"Lookups"</th>
                                    <th>"First Seen"</th>
                                    <th>"Last Seen"</th>
                                    <th>"Note"</th>
                                    <th>"Status"</th>
                                    <th>"Actions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {v.into_iter().map(|item| {
                                    let term = item.term.clone();
                                    let lookups = item.lookup_count;
                                    let first_seen = format_relative_time(item.first_seen_at_ms);
                                    let last_seen = format_relative_time(item.last_seen_at_ms);
                                    let note_text = item.note.clone();
                                    let learned = item.learned;
                                    let item_id = item.id.0.clone();
                                    let item_id_for_edit = item_id.clone();
                                    let item_for_learn = item.clone();
                                    let item_for_save = item.clone();
                                    let source_paper_id = item.paper_id_first_seen.0.clone();
                                    let status_label = if learned { "Learned" } else { "Learning" };
                                    let status_class = if learned { "vocab-learned" } else { "vocab-learning" };

                                    view! {
                                        <tr>
                                            <td class="vocab-term">{term}</td>
                                            <td>{lookups}</td>
                                            <td>{first_seen}</td>
                                            <td>{last_seen}</td>
                                            <td>
                                                {move || {
                                                    let is_editing = editing_note_id.get().as_deref() == Some(&item_id_for_edit);
                                                    if is_editing {
                                                        let item_s = item_for_save.clone();
                                                        let draft = note_draft.get();
                                                        let app = app;
                                                        view! {
                                                            <div class="vocab-note-editor">
                                                                <textarea
                                                                    rows="2"
                                                                    prop:value=draft
                                                                    on:input=move |e| note_draft.set(event_target_value(&e))
                                                                ></textarea>
                                                                <div class="vocab-actions">
                                                                    <button
                                                                        class="vocab-btn"
                                                                        on:click=move |_| {
                                                                            let mut updated = item_s.clone();
                                                                            let n = note_draft.get();
                                                                            updated.note = if n.trim().is_empty() { None } else { Some(n) };
                                                                            app.update_vocabulary(&updated);
                                                                            editing_note_id.set(None);
                                                                        }
                                                                    >
                                                                        "Save"
                                                                    </button>
                                                                    <button
                                                                        class="vocab-btn"
                                                                        on:click=move |_| {
                                                                            editing_note_id.set(None);
                                                                        }
                                                                    >
                                                                        "Cancel"
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    } else if let Some(ref n) = note_text {
                                                        let id_edit = item_id_for_edit.clone();
                                                        let n_clone = n.clone();
                                                        view! {
                                                            <span class="vocab-note-text">{n.clone()}</span>
                                                            <button
                                                                class="vocab-btn"
                                                                on:click=move |_| {
                                                                    note_draft.set(n_clone.clone());
                                                                    editing_note_id.set(Some(id_edit.clone()));
                                                                }
                                                            >
                                                                "Edit"
                                                            </button>
                                                        }.into_any()
                                                    } else {
                                                        let id_edit = item_id_for_edit.clone();
                                                        view! {
                                                            <button
                                                                class="vocab-btn"
                                                                on:click=move |_| {
                                                                    note_draft.set(String::new());
                                                                    editing_note_id.set(Some(id_edit.clone()));
                                                                }
                                                            >
                                                                "Add Note"
                                                            </button>
                                                        }.into_any()
                                                    }
                                                }}
                                            </td>
                                            <td class=status_class>{status_label}</td>
                                            <td>
                                                <button
                                                    class=move || {
                                                        if learned { "vocab-btn vocab-btn-active" } else { "vocab-btn" }
                                                    }
                                                    on:click=move |_| {
                                                        let mut updated = item_for_learn.clone();
                                                        updated.learned = !updated.learned;
                                                        app.update_vocabulary(&updated);
                                                    }
                                                >
                                                    {move || if learned { "Unlearn" } else { "Mark Learned" }}
                                                </button>
                                                <a
                                                    class="vocab-btn vocab-source-link"
                                                    href=format!("/read/{}", source_paper_id)
                                                    title="Jump to source paper"
                                                >
                                                    "📄 Source"
                                                </a>
                                            </td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    }.into_any()
                }
            }}
        </div>
    }
}
