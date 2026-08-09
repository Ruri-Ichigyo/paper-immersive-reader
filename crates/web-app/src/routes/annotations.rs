use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use paper_immersive_reader_core::domain::{Annotation, AnnotationType};

use crate::state::use_app_state;

#[component]
pub fn AnnotationsPage() -> impl IntoView {
    let app = use_app_state();

    // Filters
    let search_query = RwSignal::new(String::new());
    let filter_type = RwSignal::new(Option::<AnnotationType>::None);
    let filter_tag = RwSignal::new(String::new());

    let all_annotations = move || {
        app.data_version.get();
        let papers = app.papers();
        let mut all: Vec<(Annotation, String)> = Vec::new();
        for sp in &papers {
            let anns = app.annotations(&sp.paper.id);
            let title = sp
                .paper
                .title
                .clone()
                .unwrap_or_else(|| "Untitled".to_string());
            for ann in anns {
                all.push((ann, title.clone()));
            }
        }
        all
    };

    let filtered = move || {
        let mut items = all_annotations();
        let q = search_query.get().to_lowercase();
        let type_filter = filter_type.get();
        let tag_filter = filter_tag.get();

        if !q.is_empty() {
            items.retain(|(ann, _)| {
                ann.quote.to_lowercase().contains(&q)
                    || ann
                        .note
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
            });
        }
        if let Some(ref t) = type_filter {
            items.retain(|(ann, _)| &ann.annotation_type == t);
        }
        if !tag_filter.is_empty() {
            items.retain(|(ann, _)| ann.tags.iter().any(|t| t == &tag_filter));
        }
        items
    };

    // Collect all unique tags for the filter dropdown
    let available_tags = move || {
        let items = all_annotations();
        let mut tags: Vec<String> = items
            .iter()
            .flat_map(|(ann, _)| ann.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    };

    view! {
        <div class="route-annotations">
            <div class="page-header">
                <h1>"Annotations"</h1>
            </div>

            <div class="vocab-filters">
                <input
                    type="text"
                    class="text-input vocab-search"
                    placeholder="Search annotations…"
                    prop:value=move || search_query.get()
                    on:input=move |e| search_query.set(event_target_value(&e))
                />
                <select
                    class="text-input"
                    on:change=move |e| {
                        let val = event_target_value(&e);
                        filter_type.set(match val.as_str() {
                            "highlight" => Some(AnnotationType::Highlight),
                            "underline" => Some(AnnotationType::Underline),
                            "note" => Some(AnnotationType::Note),
                            _ => None,
                        });
                    }
                >
                    <option value="">"All Types"</option>
                    <option value="highlight">"Highlight"</option>
                    <option value="underline">"Underline"</option>
                    <option value="note">"Note"</option>
                </select>
                <select
                    class="text-input"
                    on:change=move |e| filter_tag.set(event_target_value(&e))
                >
                    <option value="">"All Tags"</option>
                    {available_tags().into_iter().map(|t| {
                        let t_clone = t.clone();
                        view! {
                            <option value=t>{t_clone}</option>
                        }
                    }).collect::<Vec<_>>()}
                </select>
            </div>

            {move || {
                let items = filtered();
                if items.is_empty() {
                    let total = all_annotations().len();
                    if total == 0 {
                        view! {
                            <div class="empty-state">
                                <p>"No annotations yet. Select text while reading to create annotations."</p>
                                <a href="/library" class="btn btn-primary">"Browse Papers"</a>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="empty-state">
                                <p>"No annotations match your filters."</p>
                                <button
                                    class="btn btn-secondary"
                                    on:click=move |_| {
                                        search_query.set(String::new());
                                        filter_type.set(None);
                                        filter_tag.set(String::new());
                                    }
                                >
                                    "Clear Filters"
                                </button>
                            </div>
                        }.into_any()
                    }
                } else {
                    view! {
                        <div class="annotation-list">
                            {items.into_iter().map(|(ann, paper_title)| {
                                let type_class = match ann.annotation_type {
                                    AnnotationType::Highlight => "ann-highlight",
                                    AnnotationType::Underline => "ann-underline",
                                    AnnotationType::Note => "ann-note",
                                };
                                let type_label = match ann.annotation_type {
                                    AnnotationType::Highlight => "Highlight",
                                    AnnotationType::Underline => "Underline",
                                    AnnotationType::Note => "Note",
                                };
                                let quote = ann.quote.clone();
                                let note = ann.note.clone();
                                let tags = ann.tags.clone();
                                let paper_title_clone = paper_title.clone();
                                let paper_id = ann.paper_id.0.clone();
                                let navigate = use_navigate();

                                view! {
                                    <div class={format!("annotation-item {}", type_class)}>
                                        <div class="ann-paper-title">{paper_title}</div>
                                        <blockquote class="ann-quote">{quote}</blockquote>
                                        {note.as_ref().map(|n| view! {
                                            <p class="ann-note-text">{n.clone()}</p>
                                        })}
                                        <div class="ann-card-meta">
                                            <span class="ann-type-badge">{type_label}</span>
                                            {if !tags.is_empty() {
                                                view! {
                                                    <div class="ann-tags">
                                                        {tags.iter().map(|tag| {
                                                            view! { <span class="ann-tag">{tag.clone()}</span> }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }.into_any()
                                            } else {
                                                ().into_any()
                                            }}
                                        </div>
                                        <button
                                            class="btn btn-read btn-sm"
                                            on:click=move |_| {
                                                navigate(&format!("/read/{}", paper_id), Default::default());
                                            }
                                        >
                                            "Read " {paper_title_clone} " ->"
                                        </button>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
