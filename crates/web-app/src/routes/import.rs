use leptos::prelude::*;

#[component]
pub fn ImportPage() -> impl IntoView {
    view! {
        <div class="route-import">
            <h1>"Import"</h1>
            <p>"Paste text or upload a PDF to get started."</p>
        </div>
    }
}
