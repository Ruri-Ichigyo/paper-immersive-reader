use leptos::prelude::*;

use super::{SelectionAction, TextSelection};

#[component]
pub fn SelectionPopover(
    selection: Option<TextSelection>,
    on_action: Callback<(SelectionAction, TextSelection)>,
    on_dismiss: Callback<()>,
) -> impl IntoView {
    // Wrap in a div that stops mouseup propagation so clicking inside the
    // popover does not clear the underlying text selection.
    view! {
        <div
            class="selection-popover-wrapper"
            on:mouseup=|e| e.stop_propagation()
            on:mousedown=|e| e.stop_propagation()
        >
            {move || {
                if let Some(ref sel) = selection {
                    view! {
                        <div class="selection-popover">
                            <button on:click={
                                let sel = sel.clone();
                                move |_| on_action.run((SelectionAction::Annotate, sel.clone()))
                            }>
                                "📝 Note"
                            </button>
                            <button on:click={
                                let sel = sel.clone();
                                move |_| on_action.run((SelectionAction::Translate, sel.clone()))
                            }>
                                "🌐 Translate"
                            </button>
                            <button on:click={
                                let sel = sel.clone();
                                move |_| on_action.run((SelectionAction::ExplainTerm, sel.clone()))
                            }>
                                "💡 Term"
                            </button>
                            <button on:click={
                                let sel = sel.clone();
                                move |_| on_action.run((SelectionAction::ExplainSentence, sel.clone()))
                            }>
                                "📖 Sentence"
                            </button>
                            <button class="dismiss-btn" on:click=move |_| on_dismiss.run(())>
                                "✕"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}
