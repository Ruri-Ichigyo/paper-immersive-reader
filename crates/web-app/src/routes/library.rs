use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use paper_immersive_reader_core::domain::{PaperId, PaperSourceType, PaperStatus};
use wasm_bindgen::{JsCast, JsValue};

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
                            let paper_id_str = sp.paper.id.0.clone();
                            let paper_id = PaperId(sp.paper.id.0.clone());
                            let title = sp
                                .paper
                                .title
                                .clone()
                                .unwrap_or_else(|| "Untitled".to_string());
                            let source_label = source_type_label(&sp.paper.source_type);
                            let status_label = status_label(&sp.paper.status);
                            let created = format_relative_time(sp.created_at_ms);
                            let reading_time = format_reading_duration(
                                app.reading_stats(&paper_id)
                                    .map(|s| s.total_reading_ms)
                                    .unwrap_or(0),
                            );
                            let sessions = app
                                .reading_stats(&paper_id)
                                .map(|s| s.session_count)
                                .unwrap_or(0);
                            let total_blocks = app.blocks(&paper_id).len() as u32;
                            let progress = app.progress(&paper_id);
                            let progress_pct = progress.as_ref().map(|p| {
                                if total_blocks == 0 { 0u32 }
                                else { ((p.current_block_index + 1) * 100 / total_blocks) as u32 }
                            });
                            let last_read = progress.as_ref().map(|p| format_relative_time(p.updated_at_ms));
                            let nav = nav.clone();
                            let title_for_delete = title.clone();
                            view! {
                                <div class="paper-card">
                                    <div class="paper-card-header">
                                        <h2 class="paper-title">{title.clone()}</h2>
                                        <span class="paper-source">{source_label}</span>
                                    </div>
                                    <div class="paper-card-meta">
                                        <span class="paper-status">{status_label}</span>
                                        <span class="paper-date">"Imported " {created}</span>
                                    </div>
                                    <div class="paper-card-meta">
                                        <span class="paper-reading-time">
                                            "🕑 " {reading_time}
                                            {if sessions > 0 {
                                                view! { <span class="paper-sessions">" · " {sessions} " session" {if sessions == 1 { "" } else { "s" }}</span> }.into_any()
                                            } else {
                                                ().into_any()
                                            }}
                                        </span>
                                    </div>
                                    <div class="paper-card-progress">
                                        {if let Some(pct) = progress_pct {
                                            view! {
                                                <div class="progress-bar-container">
                                                    <div class="progress-bar-fill" style={format!("width: {}%", pct.min(100))}></div>
                                                </div>
                                                <div class="progress-meta">
                                                    <span class="progress-pct">{pct.min(100)}"%"</span>
                                                    {last_read.map(|lr| view! { <span class="last-read">"Last read " {lr}</span> })}
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <span class="progress-not-started">"Not started"</span> }.into_any()
                                        }}
                                    </div>
                                    <div class="paper-card-actions">
                                        <button
                                            class="btn btn-read"
                                            on:click=move |_| {
                                                nav(&format!("/read/{}", paper_id_str), Default::default());
                                            }
                                        >
                                            "Read ->"
                                        </button>
                                        <button
                                            class="btn btn-secondary btn-export"
                                            title="Export paper data"
                                            on:click={
                                                let pid_for_export = paper_id.clone();
                                                let app = app;
                                                move |_| {
                                                    if let Some(bundle) = app.export_paper(&pid_for_export) {
                                                        trigger_download(&bundle.filename(), &bundle.to_json().unwrap_or_default());
                                                    }
                                                }
                                            }
                                        >
                                            "Export"
                                        </button>
                                        <button
                                            class="btn btn-danger btn-delete-paper"
                                            title="Delete paper"
                                            on:click={
                                                let pid_for_delete = paper_id.clone();
                                                move |_| {
                                                    let msg = format!(
                                                        "Are you sure you want to delete \"{}\"? This cannot be undone.",
                                                        title_for_delete
                                                    );
                                                    if web_sys::window()
                                                        .and_then(|w| w.confirm_with_message(&msg).ok())
                                                        .unwrap_or(false)
                                                    {
                                                        app.delete_paper(&pid_for_delete);
                                                    }
                                                }
                                            }
                                        >
                                            "Delete"
                                        </button>
                                    </div>
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

fn format_reading_duration(ms: u64) -> String {
    if ms < 60_000 {
        "< 1 min".to_string()
    } else {
        let total_secs = ms / 1000;
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m", mins)
        }
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

/// Create a JSON file download in the browser via Blob + object URL.
fn trigger_download(filename: &str, content: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let parts = js_sys::Array::of1(&JsValue::from_str(content));
    let blob = match web_sys::Blob::new_with_str_sequence(&parts) {
        Ok(b) => b,
        Err(_) => return,
    };
    let url = match web_sys::Url::create_object_url_with_blob(&blob) {
        Ok(u) => u,
        Err(_) => return,
    };
    let anchor: web_sys::Element = match document.create_element("a") {
        Ok(el) => el,
        Err(_) => return,
    };
    let anchor: web_sys::HtmlAnchorElement = match anchor.dyn_into() {
        Ok(a) => a,
        Err(_) => return,
    };
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();
    web_sys::Url::revoke_object_url(&url).ok();
}
