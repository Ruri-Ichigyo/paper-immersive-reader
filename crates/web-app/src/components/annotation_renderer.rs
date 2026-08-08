use leptos::prelude::*;
use paper_immersive_reader_core::domain::{Annotation, AnnotationId, AnnotationType};

#[component]
pub fn AnnotationRenderer(
    annotations: Vec<Annotation>,
    #[prop(optional)] on_delete: Option<Callback<AnnotationId>>,
    #[prop(optional)] on_update: Option<Callback<Annotation>>,
) -> impl IntoView {
    view! {
        <div class="annotation-list">
            {annotations.into_iter().map(|ann| {
                let type_class = match ann.annotation_type {
                    AnnotationType::Highlight => "ann-highlight",
                    AnnotationType::Underline => "ann-underline",
                    AnnotationType::Note => "ann-note",
                };
                let note = ann.note.clone();
                let tags = ann.tags.clone();
                let ann_for_edit = ann.clone();
                let ann_for_delete = ann.id.clone();
                let delete_cb = on_delete;
                let update_cb = on_update;

                // Inline edit state
                let editing_note = RwSignal::new(false);
                let note_draft = RwSignal::new(String::new());

                view! {
                    <div class={format!("annotation-item {}", type_class)}>
                        <blockquote class="ann-quote">"{ann.quote.clone()}"</blockquote>
                        {move || {
                            if editing_note.get() {
                                let draft = note_draft.get();
                                let ann_e = ann_for_edit.clone();
                                view! {
                                    <div class="ann-inline-edit">
                                        <textarea
                                            class="text-area"
                                            rows="2"
                                            prop:value=draft
                                            on:input=move |e| note_draft.set(event_target_value(&e))
                                        ></textarea>
                                        <div class="ann-edit-actions">
                                            <button
                                                class="btn btn-primary btn-sm"
                                                on:click={
                                                    let mut a = ann_e.clone();
                                                    move |_| {
                                                        let n = note_draft.get();
                                                        a.note = if n.trim().is_empty() { None } else { Some(n) };
                                                        if let Some(cb) = update_cb {
                                                            cb.run(a.clone());
                                                        }
                                                        editing_note.set(false);
                                                    }
                                                }
                                            >
                                                "Save"
                                            </button>
                                            <button
                                                class="btn btn-secondary btn-sm"
                                                on:click=move |_| editing_note.set(false)
                                            >
                                                "Cancel"
                                            </button>
                                        </div>
                                    </div>
                                }.into_any()
                            } else if let Some(ref n) = note {
                                view! {
                                    <p class="ann-note-text">
                                        {n.clone()}
                                        {if update_cb.is_some() {
                                            let n_clone = n.clone();
                                            view! {
                                                <button
                                                    class="btn btn-sm ann-edit-btn"
                                                    on:click=move |_| {
                                                        note_draft.set(n_clone.clone());
                                                        editing_note.set(true);
                                                    }
                                                >
                                                    "edit"
                                                </button>
                                            }.into_any()
                                        } else {
                                            ().into_any()
                                        }}
                                    </p>
                                }.into_any()
                            } else if update_cb.is_some() {
                                view! {
                                    <button
                                        class="btn btn-sm ann-edit-btn"
                                        on:click=move |_| {
                                            note_draft.set(String::new());
                                            editing_note.set(true);
                                        }
                                    >
                                        "+ Add note"
                                    </button>
                                }.into_any()
                            } else {
                                ().into_any()
                            }
                        }}
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
                        {if let Some(cb) = delete_cb {
                            let id = ann_for_delete.clone();
                            view! {
                                <button
                                    class="btn btn-delete-annotation"
                                    title="Delete annotation"
                                    on:click=move |_| cb.run(id.clone())
                                >
                                    "✕"
                                </button>
                            }.into_any()
                        } else {
                            ().into_any()
                        }}
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
