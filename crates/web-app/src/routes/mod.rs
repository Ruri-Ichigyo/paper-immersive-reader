pub mod import;
pub mod library;
pub mod read;
pub mod settings;
pub mod vocabulary;

use import::ImportPage;
use leptos::prelude::*;
use library::LibraryPage;
use read::ReadPage;
use settings::SettingsPage;
use vocabulary::VocabularyPage;

#[component]
pub fn Router() -> impl IntoView {
    view! {
        <div class="router-placeholder">
            <LibraryPage />
            <ImportPage />
            <ReadPage />
            <VocabularyPage />
            <SettingsPage />
        </div>
    }
}
