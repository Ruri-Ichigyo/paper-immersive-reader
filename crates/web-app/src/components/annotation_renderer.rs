use leptos::prelude::*;
use paper_immersive_reader_core::domain::{Annotation, AnnotationType};

#[component]
pub fn AnnotationRenderer(annotations: Vec<Annotation>) -> impl IntoView {
    view! {
        <div class="annotation-list">
            <h3>"Annotations"</h3>
            {annotations.into_iter().map(|ann| {
                let type_class = match ann.annotation_type {
                    AnnotationType::Highlight => "ann-highlight",
                    AnnotationType::Underline => "ann-underline",
                    AnnotationType::Note => "ann-note",
                };
                let note = ann.note.clone();
                let tags = ann.tags.clone();
                view! {
                    <div class={format!("annotation-item {}", type_class)}>
                        <blockquote class="ann-quote">"{ann.quote.clone()}"</blockquote>
                        {note.map(|n| view! { <p class="ann-note-text">{n}</p> })}
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
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
