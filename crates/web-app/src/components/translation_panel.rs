use leptos::prelude::*;

#[component]
pub fn TranslationPanel(
    source_text: Option<String>,
    translation: Option<String>,
    language: String,
    is_loading: bool,
    error: Option<String>,
) -> impl IntoView {
    view! {
        <div class="translation-panel">
            <h3>"Translation (" {language} ")"</h3>
            {move || {
                if is_loading {
                    view! { <p class="tl-loading">"Translating..."</p> }.into_any()
                } else if let Some(ref err) = error {
                    view! { <p class="tl-error">{err.clone()}</p> }.into_any()
                } else if let (Some(ref src), Some(ref tl)) = (&source_text, &translation) {
                    view! {
                        <div class="tl-result">
                            <p class="tl-source"><em>{src.clone()}</em></p>
                            <p class="tl-target">{tl.clone()}</p>
                        </div>
                    }.into_any()
                } else {
                    view! { <p class="tl-empty">"Select text and choose Translate."</p> }.into_any()
                }
            }}
        </div>
    }
}
