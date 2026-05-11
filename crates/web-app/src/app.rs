use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <nav>
            <a href="/library">"Library"</a>
            <a href="/import">"Import"</a>
            <a href="/vocabulary">"Vocabulary"</a>
            <a href="/settings">"Settings"</a>
        </nav>
        <main>
            <crate::routes::Router />
        </main>
    }
}
