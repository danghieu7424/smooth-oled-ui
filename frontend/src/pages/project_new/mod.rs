use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct CreateProjectReq {
    project_id: String,
    name: String,
}

#[component]
pub fn ProjectNewPage() -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (project_id, set_project_id) = create_signal(String::new());
    let (error_msg, set_error_msg) = create_signal(None::<String>);
    let (is_loading, set_is_loading) = create_signal(false);
    
    let navigate = use_navigate();

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        if name.get().is_empty() || project_id.get().is_empty() {
            set_error_msg.set(Some("Vui lòng điền đầy đủ thông tin.".to_string()));
            return;
        }

        set_is_loading.set(true);
        set_error_msg.set(None);

        let nav = navigate.clone();

        spawn_local(async move {
            let req_body = CreateProjectReq {
                name: name.get(),
                project_id: project_id.get(),
            };

            let res = gloo_net::http::Request::post("http://localhost:7424/api/projects")
                .json(&req_body)
                .expect("Failed to serialize")
                .send()
                .await;

            set_is_loading.set(false);

            match res {
                Ok(resp) => {
                    if resp.ok() {
                        nav("/", Default::default());
                    } else {
                        set_error_msg.set(Some("Lỗi khi tạo dự án (Có thể ID đã tồn tại).".to_string()));
                    }
                }
                Err(_) => {
                    set_error_msg.set(Some("Lỗi kết nối máy chủ.".to_string()));
                }
            }
        });
    };

    view! {
        <div class="firebase-layout firebase-new-project">
            <header class="fb-header">
                <div class="fb-header-left">
                    <A href="/" class="logo">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="#FFA000"><path d="M11.64 5.93h.01L15.8 13.5l1.83-3.19a.53.53 0 0 1 .9 0l4.31 7.5a.51.51 0 0 1-.44.78H1.61a.51.51 0 0 1-.45-.77l6.83-11.96a.53.53 0 0 1 .9 0l1.43 2.5 1.32-2.43a.52.52 0 0 1 .9 0z"/></svg>
                        <span>"OTA Hub"</span>
                    </A>
                </div>
            </header>

            <main class="fb-main fb-centered">
                <div class="create-project-card">
                    <h1>"Tạo một dự án"</h1>
                    <p class="subtitle">"Dự án cho phép bạn quản lý thiết bị và cập nhật Firmware."</p>

                    <form on:submit=on_submit>
                        <div class="form-group">
                            <label for="p_name">"Tên dự án của bạn là gì?"</label>
                            <input 
                                type="text" 
                                id="p_name"
                                placeholder="VD: ESP32 Smart Home"
                                prop:value=name
                                on:input=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_name.set(val.clone());
                                    let generated_id = val.to_lowercase().replace(" ", "-");
                                    set_project_id.set(generated_id);
                                }
                            />
                        </div>

                        <div class="form-group">
                            <label for="p_id">"ID Dự án (định danh duy nhất)"</label>
                            <input 
                                type="text" 
                                id="p_id"
                                placeholder="VD: esp32-smart-home"
                                prop:value=project_id
                                on:input=move |ev| set_project_id.set(event_target_value(&ev))
                            />
                            <small class="hint">"ID dự án không thể thay đổi sau khi tạo."</small>
                        </div>

                        {move || {
                            error_msg.get().map(|msg| view! {
                                <div class="error-alert">
                                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
                                    {msg}
                                </div>
                            })
                        }}

                        <div class="form-actions">
                            <A href="/" class="btn-cancel">"Hủy"</A>
                            <button type="submit" class="btn-submit" disabled=is_loading>
                                {move || if is_loading.get() { "Đang tạo..." } else { "Tạo dự án" }}
                            </button>
                        </div>
                    </form>
                </div>
            </main>
        </div>
    }
}
