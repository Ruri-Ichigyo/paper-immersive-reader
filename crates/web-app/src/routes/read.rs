use leptos::prelude::*;

#[component]
pub fn ReadPage() -> impl IntoView {
    view! {
        <div class="route-read">
            <h1>"Read"</h1>
            <p>"Select a paper from the library to start reading."</p>
        </div>
    }
}
