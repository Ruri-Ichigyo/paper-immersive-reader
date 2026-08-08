use leptos::prelude::*;

use crate::state::use_app_state;
use crate::storage::schema::Settings;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let app = use_app_state();

    // Load current settings
    let language = RwSignal::new(app.settings().explanation_language.clone());
    let gateway_url = RwSignal::new(app.gateway_url.get());
    let custom_tags = RwSignal::new(app.settings().custom_tags.join(", "));
    let saved_msg = RwSignal::new(Option::<String>::None);

    let on_save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let tags: Vec<String> = custom_tags
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let gw_url = gateway_url.get();
        let settings = Settings {
            explanation_language: language.get(),
            custom_tags: tags,
            ai_gateway_url: Some(gw_url.clone()),
            ai_gateway_token_ref: None,
        };

        app.save_settings(&settings);
        app.gateway_url.set(gw_url);
        saved_msg.set(Some("Settings saved.".to_string()));
    };

    view! {
        <div class="route-settings">
            <div class="page-header">
                <h1>"Settings"</h1>
            </div>

            <form class="settings-form" on:submit=on_save>
                <div class="form-field">
                    <label>"AI Explanation Language"</label>
                    <select
                        class="text-input"
                        prop:value=move || language.get()
                        on:change=move |e| language.set(event_target_value(&e))
                    >
                        <option value="en">"English"</option>
                        <option value="zh">"中文 (Chinese)"</option>
                        <option value="ja">"日本語 (Japanese)"</option>
                        <option value="ko">"한국어 (Korean)"</option>
                    </select>
                    <p class="form-hint">"Language for AI term and sentence explanations."</p>
                </div>

                <div class="form-field">
                    <label>"AI Gateway URL"</label>
                    <input
                        type="text"
                        class="text-input"
                        placeholder="http://127.0.0.1:3000"
                        prop:value=move || gateway_url.get()
                        on:input=move |e| gateway_url.set(event_target_value(&e))
                    />
                    <p class="form-hint">"The gateway handles AI translation and explanation requests."</p>
                </div>

                <div class="form-field">
                    <label>"Custom Tags"</label>
                    <input
                        type="text"
                        class="text-input"
                        placeholder="comma-separated tags"
                        prop:value=move || custom_tags.get()
                        on:input=move |e| custom_tags.set(event_target_value(&e))
                    />
                    <p class="form-hint">
                        "Default tags: 重点 / 疑问 / 术语 / 结论 / 复习. Add your own above (comma-separated)."
                    </p>
                </div>

                {move || saved_msg.get().map(|m| view! {
                    <p class="form-success">{m}</p>
                })}

                <button type="submit" class="btn btn-primary">"Save Settings"</button>
            </form>
        </div>
    }
}
