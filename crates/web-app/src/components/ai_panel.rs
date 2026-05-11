use leptos::prelude::*;

#[component]
pub fn AiPanel(
    content: Option<String>,
    panel_type: String,
    is_loading: bool,
    error: Option<String>,
) -> impl IntoView {
    view! {
        <div class="ai-panel">
            <h3>{panel_type}</h3>
            {move || {
                if is_loading {
                    view! { <p class="ai-loading">"Loading..."</p> }.into_any()
                } else if let Some(ref err) = error {
                    view! { <p class="ai-error">{err.clone()}</p> }.into_any()
                } else if let Some(ref text) = content {
                    view! { <div class="ai-content"><p>{text.clone()}</p></div> }.into_any()
                } else {
                    view! { <p class="ai-empty">"Select text and choose an action."</p> }.into_any()
                }
            }}
        </div>
    }
}
