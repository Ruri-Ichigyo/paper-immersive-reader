use leptos::prelude::*;
use paper_immersive_reader_core::domain::{BlockId};

/// Render block text as properly separated paragraphs.
///
/// The block text is split on `\n\n` boundaries.
/// Each paragraph is rendered as a `<p>` element for readable flowing layout.
/// Annotation decorations are rendered in the separate Annotations section
/// below the reading area rather than inline.
#[component]
pub fn BlockContent(
    block_text: String,
    block_id: Option<BlockId>,
    annotations: Vec<paper_immersive_reader_core::domain::Annotation>,
    on_mouse_up: Callback<()>,
) -> impl IntoView {
    let _ = (&block_id, &annotations);
    // Split block text into paragraphs on double-newline boundaries
    let paragraphs: Vec<String> = block_text
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    // If no paragraphs could be extracted, render the raw text as one paragraph
    let paragraphs = if paragraphs.is_empty() {
        vec![block_text.clone()]
    } else {
        paragraphs
    };

    view! {
        <div class="block-content" on:mouseup=move |_| on_mouse_up.run(())>
            {paragraphs
                .into_iter()
                .map(|para| {
                    view! { <p class="reading-paragraph">{para}</p> }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}
