use leptos::prelude::*;
use paper_immersive_reader_core::domain::{Annotation, AnnotationType, BlockId};

/// Render block text with inline annotation decoration.
///
/// Splits the block text into segments based on annotation offsets,
/// wrapping annotated ranges in `<mark>`, `<u>`, or `<span>` elements.
#[component]
pub fn BlockContent(
    block_text: String,
    block_id: Option<BlockId>,
    annotations: Vec<Annotation>,
    on_mouse_up: Callback<()>,
) -> impl IntoView {
    // Filter annotations for this block and sort by start_offset
    let relevant: Vec<&Annotation> = annotations
        .iter()
        .filter(|a| block_id.as_ref() == Some(&a.block_id))
        .filter(|a| {
            a.start_offset.is_some()
                && a.end_offset.is_some()
                && a.start_offset.unwrap() < a.end_offset.unwrap()
        })
        .collect();

    let chars: Vec<char> = block_text.chars().collect();
    let total_len = chars.len() as u32;

    // Build a sorted list of (start, end, annotation_type, note)
    let mut ranges: Vec<(u32, u32, AnnotationType, Option<String>)> = relevant
        .iter()
        .map(|a| {
            let start = a.start_offset.unwrap().min(total_len);
            let end = a.end_offset.unwrap().min(total_len);
            (start, end, a.annotation_type.clone(), a.note.clone())
        })
        .filter(|(s, e, _, _)| *s < *e)
        .collect();
    ranges.sort_by_key(|(s, _, _, _)| *s);

    // Build text segments: interleaving unannotated and annotated ranges
    let mut segments: Vec<TextSegment> = Vec::new();
    let mut cursor = 0u32;

    for (start, end, ann_type, note) in &ranges {
        // Clamp to avoid overlap issues - skip if this range starts before cursor
        if *start < cursor {
            continue;
        }
        // Unannotated segment before this annotation
        if *start > cursor {
            let text: String = chars[(cursor as usize)..(*start as usize)].iter().collect();
            segments.push(TextSegment {
                text,
                annotation: None,
                note: None,
            });
        }
        // Annotated segment
        let text: String = chars[(*start as usize)..(*end as usize)].iter().collect();
        segments.push(TextSegment {
            text,
            annotation: Some((*ann_type).clone()),
            note: note.clone(),
        });
        cursor = *end;
    }

    // Trailing unannotated segment
    if cursor < total_len {
        let text: String = chars[(cursor as usize)..].iter().collect();
        segments.push(TextSegment {
            text,
            annotation: None,
            note: None,
        });
    }

    // If no segments (empty text), render empty
    if segments.is_empty() && !block_text.is_empty() {
        segments.push(TextSegment {
            text: block_text,
            annotation: None,
            note: None,
        });
    }

    view! {
        <div class="block-content" on:mouseup=move |_| on_mouse_up.run(())>
            {segments
                .into_iter()
                .map(|seg| {
                    match &seg.annotation {
                        None => view! {
                            <span>{seg.text}</span>
                        }.into_any(),
                        Some(AnnotationType::Highlight) => {
                            let title = seg.note.clone().unwrap_or_default();
                            view! {
                                <mark class="ann-inline-highlight" title=title>{seg.text}</mark>
                            }.into_any()
                        }
                        Some(AnnotationType::Underline) => {
                            let title = seg.note.clone().unwrap_or_default();
                            view! {
                                <u class="ann-inline-underline" title=title>{seg.text}</u>
                            }.into_any()
                        }
                        Some(AnnotationType::Note) => {
                            let title = seg.note.clone().unwrap_or_default();
                            view! {
                                <span class="ann-inline-note" title=title>{seg.text}</span>
                            }.into_any()
                        }
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}

struct TextSegment {
    text: String,
    annotation: Option<AnnotationType>,
    note: Option<String>,
}
