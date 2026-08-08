use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use paper_immersive_reader_core::domain::{PaperSourceType, PaperStatus};

use crate::state::use_app_state;
use crate::util::format_relative_time;

#[component]
pub fn LibraryPage() -> impl IntoView {
    let app = use_app_state();
    let navigate = use_navigate();
    let nav_for_button = navigate.clone();

    view! {
        <div class="route-library">
            <div class="page-header">
                <h1>"Library"</h1>
                <button
                    class="btn btn-primary"
                    on:click=move |_| nav_for_button("/import", Default::default())
                >
                    "+ Import Paper"
                </button>
            </div>

            {move || {
                let papers = app.papers();
                if papers.is_empty() {
                    let nav = navigate.clone();
                    view! {
                        <div class="empty-state">
                            <p>"No papers yet. Import a paper to get started."</p>
                            <button
                                class="btn btn-primary"
                                on:click=move |_| nav("/import", Default::default())
                            >
                                "Import a Paper"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    let nav = navigate.clone();
                    papers
                        .into_iter()
                        .map(move |sp| {
                            let paper_id = sp.paper.id.0.clone();
                            let title = sp
                                .paper
                                .title
                                .clone()
                                .unwrap_or_else(|| "Untitled".to_string());
                            let source_label = source_type_label(&sp.paper.source_type);
                            let status_label = status_label(&sp.paper.status);
                            let created = format_relative_time(sp.created_at_ms);
                            let nav = nav.clone();
                            view! {
                                <div class="paper-card">
                                    <div class="paper-card-header">
                                        <h2 class="paper-title">{title}</h2>
                                        <span class="paper-source">{source_label}</span>
                                    </div>
                                    <div class="paper-card-meta">
                                        <span class="paper-status">{status_label}</span>
                                        <span class="paper-date">"Imported " {created}</span>
                                    </div>
                                    <button
                                        class="btn btn-read"
                                        on:click=move |_| {
                                            nav(&format!("/read/{}", paper_id), Default::default());
                                        }
                                    >
                                        "Read ->"
                                    </button>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                        .into_any()
                }
            }}
        </div>
    }
}

fn source_type_label(st: &PaperSourceType) -> &'static str {
    match st {
        PaperSourceType::Pdf => "PDF",
        PaperSourceType::Paste => "Paste",
        PaperSourceType::Url => "URL",
        PaperSourceType::Doi => "DOI",
        PaperSourceType::Zotero => "Zotero",
    }
}

fn status_label(st: &PaperStatus) -> &'static str {
    match st {
        PaperStatus::Created => "Created",
        PaperStatus::Ingesting => "Processing…",
        PaperStatus::ReadyText => "Ready",
        PaperStatus::ReadyOcr => "Ready (OCR)",
        PaperStatus::Failed => "Failed",
    }
}
