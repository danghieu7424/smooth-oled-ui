use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use crate::shared::ui::file_upload::FileUploadDropzone;

#[derive(Clone, Serialize, Deserialize)]
struct CreateProjectReq {
    project_id: String,
    name: String,
    description: Option<String>,
}

#[component]
pub fn ProjectNewPage() -> impl IntoView {
    let (step, set_step) = create_signal(1);
    
    // Step 1
    let (name, set_name) = create_signal(String::new());
    let (project_id, set_project_id) = create_signal(String::new());
    
    // Step 2
    let (description, set_description) = create_signal(String::new());
    
    // Step 3
    let (file, set_file) = create_signal(None::<web_sys::File>);
    let (update_type, set_update_type) = create_signal("patch".to_string());

    // Fetch user profile to get ID
    let user_profile = create_resource(
        || (),
        |_| async move {
            crate::pages::dashboard::fetch_me().await.unwrap_or(crate::pages::dashboard::UserProfile {
                id: Some("0".to_string()),
                name: None,
                picture: None,
                error: None,
            })
        }
    );

    let (error_msg, set_error_msg) = create_signal(None::<String>);
    let (is_loading, set_is_loading) = create_signal(false);
    
    let navigate = use_navigate();

    let (custom_version, set_custom_version) = create_signal(String::new());

    // Derived version name based on update type and project name
    let derived_version = move || {
        let cv = custom_version.get();
        if !cv.is_empty() {
            return cv;
        }
        let n = name.get();
        let base_name = if n.is_empty() { "firmware".to_string() } else { n.replace(" ", "_") };
        let (major, minor, patch) = match update_type.get().as_str() {
            "major" => (1, 0, 0),
            "minor" => (0, 1, 0),
            _ => (0, 0, 1),
        };
        format!("{}_V{}.{}.{}", base_name, major, minor, patch)
    };

    let go_next = move |_| {
        if step.get() == 1 {
            if name.get().is_empty() || project_id.get().is_empty() {
                set_error_msg.set(Some("Vui lòng điền Tên dự án.".to_string()));
                return;
            }
            set_error_msg.set(None);
            set_step.set(2);
        } else if step.get() == 2 {
            set_step.set(3);
        }
    };

    let go_back = move |_| {
        if step.get() > 1 {
            set_step.set(step.get() - 1);
            set_error_msg.set(None);
        }
    };

    let on_submit_all = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_is_loading.set(true);
        set_error_msg.set(None);

        let nav = navigate.clone();

        spawn_local(async move {
            // Create project first
            let req_body = CreateProjectReq {
                name: name.get(),
                project_id: project_id.get(),
                description: if description.get().is_empty() { None } else { Some(description.get()) },
            };

            let res = gloo_net::http::Request::post("http://localhost:7424/api/projects")
                .credentials(web_sys::RequestCredentials::Include)
                .json(&req_body)
                .expect("Failed to serialize")
                .send()
                .await;

            if let Ok(resp) = res {
                if resp.ok() {
                    // Extract full project_id from response
                    let created_data: serde_json::Value = resp.json().await.unwrap_or_default();
                    let full_project_id = created_data["project_id"].as_str().unwrap_or(&project_id.get()).to_string();

                    // If file is selected, upload it
                    if let Some(upload_file) = file.get() {
                        let form_data = web_sys::FormData::new().unwrap();
                        let filename = upload_file.name();
                        form_data.append_with_str("version", &derived_version()).unwrap();
                        form_data.append_with_blob_and_filename("file", &upload_file.into(), &filename).unwrap();

                        let upload_res = gloo_net::http::Request::post(&format!("http://localhost:7424/api/projects/{}/firmware", full_project_id))
                            .credentials(web_sys::RequestCredentials::Include)
                            .body(form_data).unwrap()
                            .send()
                            .await;

                        if upload_res.is_err() || !upload_res.as_ref().unwrap().ok() {
                            set_error_msg.set(Some("Dự án đã tạo nhưng Upload Firmware thất bại.".to_string()));
                            set_is_loading.set(false);
                            return;
                        }
                    }

                    nav("/", Default::default());
                    return;
                }
            }
            set_error_msg.set(Some("Lỗi khi tạo dự án (Có thể ID đã tồn tại).".to_string()));
            set_is_loading.set(false);
        });
    };

    let on_file_change = move |ev: ev::Event| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        if let Some(files) = input.files() {
            if let Some(f) = files.get(0) {
                if f.name().ends_with(".bin") {
                    set_file.set(Some(f));
                    set_error_msg.set(None);
                } else {
                    set_file.set(None);
                    set_error_msg.set(Some("Chỉ chấp nhận file định dạng .bin".to_string()));
                }
            }
        }
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
                <div class="create-project-card wizard-mode-vertical">
                    <h1>"Tạo một dự án"</h1>
                    <p class="subtitle">"Dự án cho phép bạn quản lý thiết bị và cập nhật Firmware."</p>

                    <form on:submit=on_submit_all>
                        
                        // --- STEP 1 ---
                        <div class=move || format!("wizard-step {}", if step.get() >= 1 { "expanded" } else { "collapsed" })>
                            <div class="step-header">
                                <div class=move || format!("step-number {}", if step.get() > 1 { "completed" } else { "active" })>
                                    {move || if step.get() > 1 { "✓".to_string() } else { "1".to_string() }}
                                </div>
                                <h2>"Thông tin cơ bản"</h2>
                            </div>
                            <div class="step-content" style=move || if step.get() == 1 { "display: block;" } else { "display: none;" }>
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
                                    <label for="p_id">"ID Dự án (Tự động tạo)"</label>
                                    <div class="id-preview">
                                        <span>{move || project_id.get()}</span>
                                    </div>
                                    <small class="hint">"ID dự án sẽ được dùng trong đường link cập nhật Firmware (Ví dụ: /api/firmware/esp32-tool)."</small>
                                </div>
                                <div class="form-actions wizard-actions">
                                    <A href="/" class="btn-cancel">"Hủy"</A>
                                    <button type="button" class="btn-submit" on:click=go_next>"Tiếp tục"</button> 
                                </div>
                            </div>
                        </div>

                        // --- STEP 2 ---
                        <div class=move || format!("wizard-step {}", if step.get() >= 2 { "expanded" } else { "collapsed" })>
                            <div class="step-header">
                                <div class=move || format!("step-number {}", if step.get() > 2 { "completed" } else if step.get() == 2 { "active" } else { "" })>
                                    {move || if step.get() > 2 { "✓".to_string() } else { "2".to_string() }}
                                </div>
                                <h2>"Mô tả dự án"</h2>
                            </div>
                            <div class="step-content" style=move || if step.get() == 2 { "display: block;" } else { "display: none;" }>
                                <div class="form-group">
                                    <label for="p_desc">"Mô tả dự án (Có thể bỏ qua)"</label>
                                    <textarea 
                                        id="p_desc"
                                        placeholder="VD: Dự án điều khiển đèn LED qua Wifi..."
                                        rows="4"
                                        prop:value=description
                                        on:input=move |ev| set_description.set(event_target_value(&ev))
                                    ></textarea>
                                </div>
                                <div class="form-actions wizard-actions">
                                    <button type="button" class="btn-cancel" on:click=go_back>"Quay lại"</button>
                                    <button type="button" class="btn-submit" on:click=go_next>"Tiếp tục / Bỏ qua"</button> 
                                </div>
                            </div>
                        </div>

                        // --- STEP 3 ---
                        <div class=move || format!("wizard-step {}", if step.get() >= 3 { "expanded" } else { "collapsed" })>
                            <div class="step-header">
                                <div class=move || format!("step-number {}", if step.get() == 3 { "active" } else { "" })>"3"</div>
                                <h2>"Khởi tạo Firmware"</h2>
                            </div>
                            <div class="step-content" style=move || if step.get() == 3 { "display: block;" } else { "display: none;" }>
                                <div class="form-group mb-4">
                                    <FileUploadDropzone 
                                        on_files_select={move |files: Vec<web_sys::File>| {
                                            if let Some(f) = files.into_iter().next() {
                                                set_file.set(Some(f));
                                            }
                                        }}
                                        on_clear={move |_| set_file.set(None)}
                                        title="Tải lên Firmware khởi tạo (Có thể bỏ qua)".to_string()
                                        description="Kéo thả hoặc click để chọn file .bin".to_string()
                                        accept=".bin".to_string()
                                    />
                                </div>
                                
                                {move || if file.get().is_some() {
                                    view! {
                                        <div class="version-generator">
                                            <label>"Tên phiên bản:"</label>
                                            <div class="version-preview" style="padding: 0; background: transparent; border: none; margin-bottom: 1rem;">
                                                <input 
                                                    type="text" 
                                                    style="width: 100%; background: #1e1e20; border: 1px solid rgba(255, 255, 255, 0.15); color: #fff; padding: 0.75rem 1rem; border-radius: 6px; font-size: 1rem; font-family: monospace;"
                                                    placeholder=move || {
                                                        let n = name.get();
                                                        let base_name = if n.is_empty() { "firmware".to_string() } else { n.replace(" ", "_") };
                                                        let (major, minor, patch) = match update_type.get().as_str() {
                                                            "major" => (1, 0, 0),
                                                            "minor" => (0, 1, 0),
                                                            _ => (0, 0, 1),
                                                        };
                                                        format!("{}_V{}.{}.{}", base_name, major, minor, patch)
                                                    }
                                                    prop:value=custom_version
                                                    on:input=move |ev| set_custom_version.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <div class="version-checkboxes" style="display: flex; gap: 1.5rem;">
                                                <label class="cb-label">
                                                    <input type="radio" name="v_type" prop:checked=move || update_type.get() == "major" on:change=move |_| { set_update_type.set("major".to_string()); set_custom_version.set(String::new()); } />
                                                    <span>"Bản chính thức"</span>
                                                </label>
                                                <label class="cb-label">
                                                    <input type="radio" name="v_type" prop:checked=move || update_type.get() == "minor" on:change=move |_| { set_update_type.set("minor".to_string()); set_custom_version.set(String::new()); } />
                                                    <span>"Bản bổ sung"</span>
                                                </label>
                                                <label class="cb-label">
                                                    <input type="radio" name="v_type" prop:checked=move || update_type.get() == "patch" on:change=move |_| { set_update_type.set("patch".to_string()); set_custom_version.set(String::new()); } />
                                                    <span>"Bản vá lỗi"</span>
                                                </label>
                                            </div>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {}.into_view()
                                }}

                                {move || {
                                    error_msg.get().map(|msg| view! {
                                        <div class="error-alert mt-3">
                                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
                                            {msg}
                                        </div>
                                    })
                                }}

                                <div class="form-actions wizard-actions">
                                    <button type="button" class="btn-cancel" on:click=go_back>"Quay lại"</button>
                                    <button type="submit" class="btn-submit" disabled=is_loading>
                                        {if is_loading.get() { "Đang tạo..." } else { "Hoàn tất tạo dự án" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
            </main>
        </div>
    }
}
