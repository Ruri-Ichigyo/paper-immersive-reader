use leptos::prelude::*;

#[component]
pub fn BlockNavigator(
    can_go_previous: bool,
    can_go_next: bool,
    on_previous: Callback<()>,
    on_next: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="block-navigator">
            <button
                class="nav-btn nav-prev"
                disabled=move || !can_go_previous
                on:click=move |_| on_previous.run(())
            >
                "← Previous"
            </button>
            <button
                class="nav-btn nav-next"
                disabled=move || !can_go_next
                on:click=move |_| on_next.run(())
            >
                "Next →"
            </button>
        </div>
    }
}
