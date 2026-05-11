use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <div class="route-settings">
            <h1>"Settings"</h1>
            <p>"Configure explanation language, tags, and gateway connection."</p>
        </div>
    }
}
