use leptos::prelude::*;

#[component]
pub fn ProgressIndicator(
    current_index: u32,
    total_blocks: u32,
    progress_percent: u32,
) -> impl IntoView {
    view! {
        <div class="progress-indicator">
            <span class="progress-text">
                "Block " {current_index + 1} " of " {total_blocks}
                " (" {progress_percent} "%)"
            </span>
            <div class="progress-bar">
                <div
                    class="progress-fill"
                    style:width=format!("{}%", progress_percent)
                ></div>
            </div>
        </div>
    }
}
