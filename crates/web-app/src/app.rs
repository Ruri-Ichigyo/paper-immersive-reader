use leptos::prelude::*;
use leptos_router::components::{Redirect, Route, Router, Routes};
use leptos_router::path;

use crate::routes::{
    import::ImportPage, library::LibraryPage, read::ReadPage, settings::SettingsPage,
    vocabulary::VocabularyPage,
};
use crate::state::AppState;

#[component]
pub fn App() -> impl IntoView {
    // Provide global state via context
    let state = AppState::new();
    // Load persisted data from IndexedDB (wasm32 only, no-op on native)
    state.init_persistence();
    provide_context(state);

    view! {
        <nav class="app-nav">
            <a href="/library" class="nav-link">"📚 Library"</a>
            <a href="/import" class="nav-link">"📥 Import"</a>
            <a href="/vocabulary" class="nav-link">"📖 Vocabulary"</a>
            <a href="/settings" class="nav-link">"⚙ Settings"</a>
        </nav>
        <main class="app-main">
            <Router>
                <Routes fallback=|| view! { <p>"Loading..."</p> }>
                    <Route path=path!("/") view=move || view! { <Redirect path="/library" /> } />
                    <Route path=path!("/library") view=LibraryPage />
                    <Route path=path!("/import") view=ImportPage />
                    <Route path=path!("/read/:paper_id") view=ReadPage />
                    <Route path=path!("/vocabulary") view=VocabularyPage />
                    <Route path=path!("/settings") view=SettingsPage />
                </Routes>
            </Router>
        </main>
    }
}
