use leptos::prelude::*;

use super::{SelectionAction, TextSelection};

#[component]
pub fn SelectionPopover(
    selection: Option<TextSelection>,
    on_action: Callback<(SelectionAction, TextSelection)>,
    on_dismiss: Callback<()>,
) -> impl IntoView {
    view! {
        {move || {
            if let Some(ref sel) = selection {
                view! {
                    <div class="selection-popover">
                        <button on:click={
                            let sel = sel.clone();
move |_| on_action.run((SelectionAction::Annotate, sel.clone()))
                        }>
                            "Annotate"
                        </button>
                        <button on:click={
                            let sel = sel.clone();
move |_| on_action.run((SelectionAction::Translate, sel.clone()))
                        }>
                            "Translate"
                        </button>
                        <button on:click={
                            let sel = sel.clone();
move |_| on_action.run((SelectionAction::ExplainTerm, sel.clone()))
                        }>
                            "Explain Term"
                        </button>
                        <button on:click={
                            let sel = sel.clone();
move |_| on_action.run((SelectionAction::ExplainSentence, sel.clone()))
                        }>
                            "Explain Sentence"
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
    }
}
