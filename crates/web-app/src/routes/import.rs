use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[cfg(not(target_arch = "wasm32"))]
use crate::import::pdf_bridge::StubPdfBridge;
use crate::import::WebImportError;
use crate::state::use_app_state;

#[component]
pub fn ImportPage() -> impl IntoView {
    let app = use_app_state();
    let navigate = use_navigate();
    let nav_for_paste = navigate.clone();
    let nav_for_pdf = navigate.clone();

    // Paste-text form state
    let paste_title = RwSignal::new(String::new());
    let paste_body = RwSignal::new(String::new());
    let paste_error = RwSignal::new(Option::<String>::None);
    let paste_loading = RwSignal::new(false);
    let paste_stage = RwSignal::new(String::new());

    // PDF upload form state
    let pdf_title = RwSignal::new(String::new());
    let pdf_filename = RwSignal::new(Option::<String>::None);
    let pdf_error = RwSignal::new(Option::<String>::None);
    let pdf_loading = RwSignal::new(false);
    let pdf_bytes = RwSignal::new(Vec::<u8>::new());
    let pdf_stage = RwSignal::new(String::new());

    let on_paste_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        paste_error.set(None);
        paste_loading.set(true);
        paste_stage.set("Segmenting text…".to_string());

        let title = paste_title.get();
        let body = paste_body.get();

        let store = app.clone_store();
        let mut service = crate::import::WebImportService::new(store);

        match service.import_paste(
            body,
            if title.trim().is_empty() {
                None
            } else {
                Some(title)
            },
        ) {
            Ok(paper) => {
                let new_store = service.into_store();
                app.replace_store(new_store);
                paste_stage.set("Done!".to_string());
                paste_loading.set(false);
                let nav = nav_for_paste.clone();
                nav(&format!("/read/{}", paper.id.0), Default::default());
            }
            Err(e) => {
                paste_loading.set(false);
                paste_stage.set(String::new());
                paste_error.set(Some(import_error_message(&e)));
            }
        }
    };

    let on_pdf_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        pdf_error.set(None);
        pdf_loading.set(true);

        let bytes = pdf_bytes.get();
        if bytes.is_empty() {
            pdf_loading.set(false);
            pdf_error.set(Some("Please select a PDF file first.".to_string()));
            return;
        }

        let title = pdf_title.get();
        let store = app.clone_store();

        #[cfg(target_arch = "wasm32")]
        {
            let bridge = crate::import::pdf_bridge::WasmPdfBridge::new();
            let nav = nav_for_pdf.clone();
            let app = app;
            pdf_stage.set("Extracting text…".to_string());
            wasm_bindgen_futures::spawn_local(async move {
                // Show staged progress while the async import runs.
                let stage = pdf_stage;
                let advance = move |msg: &'static str, delay_ms: u32| {
                    let stage = stage;
                    wasm_bindgen_futures::spawn_local(async move {
                        wait_ms(delay_ms).await;
                        stage.set(msg.to_string());
                    });
                };
                advance("Cleaning headers…", 200);
                advance("Segmenting blocks…", 600);

                let mut service = crate::import::WebImportService::new(store);
                match service
                    .import_pdf_async(
                        bytes,
                        if title.trim().is_empty() {
                            None
                        } else {
                            Some(title)
                        },
                        &bridge,
                    )
                    .await
                {
                    Ok(paper) => {
                        let new_store = service.into_store();
                        app.replace_store(new_store);
                        pdf_stage.set("Done!".to_string());
                        pdf_loading.set(false);
                        nav(&format!("/read/{}", paper.id.0), Default::default());
                    }
                    Err(e) => {
                        pdf_loading.set(false);
                        pdf_stage.set(String::new());
                        pdf_error.set(Some(import_error_message(&e)));
                    }
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let bridge = StubPdfBridge::failing();
            let mut service = crate::import::WebImportService::new(store);
            pdf_stage.set("Processing…".to_string());
            match service.import_pdf(
                bytes,
                if title.trim().is_empty() {
                    None
                } else {
                    Some(title)
                },
                &bridge,
            ) {
                Ok(paper) => {
                    let new_store = service.into_store();
                    app.replace_store(new_store);
                    pdf_stage.set("Done!".to_string());
                    pdf_loading.set(false);
                    let nav = nav_for_pdf.clone();
                    nav(&format!("/read/{}", paper.id.0), Default::default());
                }
                Err(e) => {
                    pdf_loading.set(false);
                    pdf_stage.set(String::new());
                    pdf_error.set(Some(import_error_message(&e)));
                }
            }
        }
    };

    let on_file_change = move |ev: leptos::ev::Event| {
        let input: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(file) = input.files().and_then(|f| f.get(0)) {
            pdf_filename.set(Some(file.name()));

            let reader = match web_sys::FileReader::new() {
                Ok(r) => r,
                Err(_) => {
                    pdf_error.set(Some("Failed to create file reader.".to_string()));
                    return;
                }
            };
            let reader_clone = reader.clone();
            let onload = Closure::wrap(Box::new(move |_| {
                if let Ok(array_buffer) = reader_clone.result() {
                    let array = js_sys::Uint8Array::new(&array_buffer);
                    let bytes = array.to_vec();
                    pdf_bytes.set(bytes);
                }
            }) as Box<dyn FnMut(wasm_bindgen::JsValue)>);
            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            if reader.read_as_array_buffer(&file).is_err() {
                pdf_error.set(Some("Failed to read file.".to_string()));
            }
        }
    };

    view! {
        <div class="route-import">
            <h1>"Import Paper"</h1>

            <div class="import-sections">
                // --- Paste Text ---
                <div class="import-section">
                    <h2>"Paste Text"</h2>
                    <form on:submit=on_paste_submit>
                        <div class="form-field">
                            <label>"Title (optional)"</label>
                            <input
                                type="text"
                                class="text-input"
                                placeholder="Paper title"
                                prop:value=paste_title
                                on:input=move |e| paste_title.set(event_target_value(&e))
                            />
                        </div>
                        <div class="form-field">
                            <label>"Content"</label>
                            <textarea
                                class="text-area"
                                rows="10"
                                placeholder="Paste the paper text here. Separate paragraphs with a blank line."
                                prop:value=paste_body
                                on:input=move |e| paste_body.set(event_target_value(&e))
                            ></textarea>
                        </div>
                        {move || paste_error.get().map(|e| view! {
                            <p class="form-error">{e}</p>
                        })}
                        {move || {
                            let stage = paste_stage.get();
                            if !stage.is_empty() {
                                view! { <p class="import-stage"><span class="import-spinner"></span>{stage}</p> }.into_any()
                            } else {
                                ().into_any()
                            }
                        }}
                        <button
                            type="submit"
                            class="btn btn-primary"
                            disabled=move || paste_loading.get()
                        >
                            {move || if paste_loading.get() { "Importing…" } else { "Import Text" }}
                        </button>
                    </form>
                </div>

                // --- PDF Upload ---
                <div class="import-section">
                    <h2>"Upload PDF"</h2>
                    <form on:submit=on_pdf_submit>
                        <div class="form-field">
                            <label>"Title (optional)"</label>
                            <input
                                type="text"
                                class="text-input"
                                placeholder="Paper title"
                                prop:value=pdf_title
                                on:input=move |e| pdf_title.set(event_target_value(&e))
                            />
                        </div>
                        <div class="form-field">
                            <label>"PDF File"</label>
                            <input
                                type="file"
                                accept="application/pdf"
                                on:change=on_file_change
                            />
                            {move || pdf_filename.get().map(|n| view! {
                                <span class="file-name">"Selected: " {n}</span>
                            })}
                        </div>
                        {move || pdf_error.get().map(|e| view! {
                            <p class="form-error">{e}</p>
                        })}
                        {move || {
                            let stage = pdf_stage.get();
                            if !stage.is_empty() {
                                view! { <p class="import-stage"><span class="import-spinner"></span>{stage}</p> }.into_any()
                            } else {
                                ().into_any()
                            }
                        }}
                        <button
                            type="submit"
                            class="btn btn-primary"
                            disabled=move || pdf_loading.get()
                        >
                            {move || if pdf_loading.get() { "Importing…" } else { "Import PDF" }}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}

fn import_error_message(e: &WebImportError) -> String {
    match e {
        WebImportError::EmptyInput => "The text is empty. Please paste some content.".to_string(),
        WebImportError::ExtractionFailed => {
            "Failed to extract text from the PDF. It may be a scanned document.".to_string()
        }
        WebImportError::StorageFailed => "Failed to save the paper. Please try again.".to_string(),
    }
}

#[cfg(target_arch = "wasm32")]
async fn wait_ms(ms: u32) {
    use wasm_bindgen_futures::JsFuture;
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let window = web_sys::window().unwrap();
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32);
    });
    let _ = JsFuture::from(promise).await;
}
