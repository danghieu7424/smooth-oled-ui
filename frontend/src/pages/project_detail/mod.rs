use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use crate::pages::dashboard::fetch_me;
use crate::shared::ui::file_upload::FileUploadDropzone;

#[derive(Clone, Copy, PartialEq)]
pub enum UpdateType {
    Major,
    Minor,
    Patch,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Firmware {
    pub id: i64,
    pub version: String,
    pub file_path: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub devices_count: i64,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct ProjectDetail {
    pub id: i64,
    pub project_id: String,
    pub user_suid: String,
    pub name: String,
    pub active_devices: i64,
    pub latest_version: String,
    pub token: String,
    pub firmwares: Vec<Firmware>,
}

async fn fetch_project_detail(id: String) -> Result<ProjectDetail, String> {
    gloo_net::http::Request::get(&format!("http://localhost:7424/api/projects/{}", id))
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ProjectDetail>()
        .await
        .map_err(|e| e.to_string())
}

#[component]
pub fn ProjectDetailPage() -> impl IntoView {
    let params = use_params_map();
    let id_str = move || params.with(|p| p.get("id").cloned().unwrap_or_default());
    
    let active_tab = move || params.with(|p| p.get("tab").cloned().unwrap_or_else(|| "dashboard".to_string()));
    let navigate = use_navigate();
    let (update_type, set_update_type) = create_signal(UpdateType::Patch);
    let (show_upload_modal, set_show_upload_modal) = create_signal(false);
    
    let (fw_file, set_fw_file) = create_signal(None::<web_sys::File>);
    let (fw_notes, set_fw_notes) = create_signal(String::new());
    let (upload_status, set_upload_status) = create_signal(String::new());
    let (is_uploading, set_is_uploading) = create_signal(false);
    
    let (toast_msg, set_toast_msg) = create_signal(String::new());
    let (show_toast, set_show_toast) = create_signal(false);
    let toast_timer = store_value(None::<gloo_timers::callback::Timeout>);
    
    let trigger_toast = move |msg: &str| {
        set_toast_msg(msg.to_string());
        set_show_toast(true);
        if let Some(t) = toast_timer.get_value() {
            t.cancel();
        }
        let timer = gloo_timers::callback::Timeout::new(2500, move || {
            set_show_toast(false);
        });
        toast_timer.set_value(Some(timer));
    };
    
    let project_resource = create_resource(
        move || id_str(),
        |id| async move {
            if id.is_empty() { return Err("No ID".to_string()); }
            fetch_project_detail(id).await
        }
    );

    view! {
        <div class="firebase-layout firebase-project-detail">

            <div class={move || if show_toast() { "copy-toast show" } else { "copy-toast" }}>
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 6L9 17l-5-5"></path></svg>
                {move || toast_msg()}
            </div>

            <div class="detail-container">
                // Sidebar
                <aside class="fb-sidebar">
                    <div class="sidebar-group">
                        <div class={
                            let nav = navigate.clone();
                            let id = id_str.clone();
                            move || if active_tab() == "dashboard" { "sidebar-item active" } else { "sidebar-item" } 
                        } on:click={
                            let nav = navigate.clone();
                            let id = id_str.clone();
                            move |_| nav(&format!("/projects/{}/dashboard", id()), Default::default())
                        }>
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                            <span>"Dashboard"</span>
                        </div>
                    </div>
                    
                    <div class="sidebar-title">"QUẢN LÝ"</div>
                    <div class="sidebar-group">
                        <div class={
                            let nav = navigate.clone();
                            let id = id_str.clone();
                            move || if active_tab() == "versions" { "sidebar-item active" } else { "sidebar-item" } 
                        } on:click={
                            let nav = navigate.clone();
                            let id = id_str.clone();
                            move |_| nav(&format!("/projects/{}/versions", id()), Default::default())
                        }>
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"></path></svg>
                            <span>"Các phiên bản"</span>
                        </div>
                    </div>
                </aside>

                // Main Analytics View
                <main class="fb-analytics-main">
                    <Suspense fallback=move || view! { <div class="loading-state">"Đang tải dữ liệu..."</div> }>
                        {move || {
                            match project_resource.get() {
                                Some(Ok(detail)) => {
                                    let api_link = format!("http://localhost:7424/api/firmware/{}-{}", detail.user_suid, detail.project_id);
                                    let d_name = detail.name.clone();
                                    let d_project_id = detail.project_id.clone();
                                    let detail_clone = detail.clone();
                                    let full_id_copy = format!("{}-{}", detail.user_suid, detail.project_id);
                                    let copy_id = move |_| {
                                        if let Some(window) = web_sys::window() {
                                            let clipboard = window.navigator().clipboard();
                                            let _ = clipboard.write_text(&full_id_copy);
                                            trigger_toast("Đã copy toàn bộ ID!");
                                        }
                                    };
                                    let token_copy = detail.token.clone();
                                    let copy_token = move |_| {
                                        if let Some(window) = web_sys::window() {
                                            let clipboard = window.navigator().clipboard();
                                            let _ = clipboard.write_text(&token_copy);
                                            trigger_toast("Đã copy Token bí mật!");
                                        }
                                    };
                                    
                                    view! {
                                        <div class="analytics-header">
                                            <div class="title-row">
                                                <div class="main-header">
                                                    <div>
                                                        <h1 style="color: #fff; font-size: 1.5rem; margin-bottom: 0.5rem;">{d_name} <span class="badge">"Pro"</span></h1>
                                                        <div class="project-info" style="color: #90a4ae; font-size: 0.9rem; font-family: monospace;">
                                                            <div style="margin-bottom: 0.25rem; display: flex; align-items: center; gap: 0.5rem;">
                                                                <div>"ID: " <span style="color: #82b1ff;">{d_project_id}</span></div>
                                                                <button title="Copy Full ID" on:click=copy_id style="background: none; border: none; color: #90a4ae; cursor: pointer; padding: 0.2rem; display: flex; align-items: center; border-radius: 4px;" class="fw-delete-btn">
                                                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                                                                </button>
                                                            </div>
                                                            <div style="margin-bottom: 0.25rem; display: flex; align-items: center; gap: 0.5rem;">
                                                                <div>"Token: " <span style="color: #ffb74d;">"••••••••••••••••"</span></div>
                                                                <button title="Copy Token" on:click=copy_token style="background: none; border: none; color: #90a4ae; cursor: pointer; padding: 0.2rem; display: flex; align-items: center; border-radius: 4px;" class="fw-delete-btn">
                                                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                                                                </button>
                                                            </div>
                                                            <div>"Link Update: " <span style="color: #a5d6a7;">{api_link}</span></div>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        {move || {
                                            let active = active_tab();
                                            let detail_inner = detail_clone.clone();
                                            if active == "dashboard" {
                                        view! {
                                            <div class="analytics-content">
                                                <h2>"Thống kê dữ liệu"</h2>
                                                
                                                <div class="analytics-card">
                                                    <div class="card-header">
                                                        <div class="card-title">
                                                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="18" y="3" width="4" height="18"></rect><rect x="10" y="8" width="4" height="13"></rect><rect x="2" y="13" width="4" height="8"></rect></svg>
                                                            "Analytics"
                                                        </div>
                                                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="1"></circle><circle cx="12" cy="5" r="1"></circle><circle cx="12" cy="19" r="1"></circle></svg>
                                                    </div>

                                                    <div class="card-body">
                                                        <div class="chart-column">
                                                            <h3>"Thiết bị cập nhật / ngày"</h3>
                                                            <div class="chart-val">
                                                                <span>"- - -"</span>
                                                            </div>
                                                            <div class="chart-area empty">
                                                                <div style="text-align: center;">
                                                                    <div>"No data"</div>
                                                                    <small>"for the last 14 days"</small>
                                                                </div>
                                                            </div>
                                                        </div>
                                                        
                                                        <div class="chart-column">
                                                            <h3>"Lượt yêu cầu phiên bản"</h3>
                                                            <div class="chart-val">
                                                                <span>"- - -"</span>
                                                            </div>
                                                            <div class="chart-area empty">
                                                                <div style="text-align: center;">
                                                                    <div>"No data"</div>
                                                                    <small>"for the last 14 days"</small>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </div>
                                                    
                                                    <div class="card-footer">
                                                        <span class="legend"><span class="color-box this-week"></span> "This week"</span>
                                                        <span class="legend"><span class="color-box last-week"></span> "Last week"</span>
                                                    </div>
                                                </div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        let p_id_for_upload = detail_inner.project_id.clone();
                                        
                                        let base_name = detail_inner.name.clone();
                                        let firmwares_for_calc = detail_inner.firmwares.clone();
                                        
                                        let calculated_version = move || {
                                            let mut major = 0;
                                            let mut minor = 0;
                                            let mut patch = 0;
                                            
                                            if let Some(latest) = firmwares_for_calc.first() {
                                                if let Some(idx) = latest.version.rfind('V').or_else(|| latest.version.rfind('v')) {
                                                    let num_str = &latest.version[idx + 1..];
                                                    let parts: Vec<&str> = num_str.split('.').collect();
                                                    if parts.len() == 3 {
                                                        major = parts[0].parse().unwrap_or(0);
                                                        minor = parts[1].parse().unwrap_or(0);
                                                        patch = parts[2].parse().unwrap_or(0);
                                                    }
                                                }
                                            }
                                            
                                            match update_type.get() {
                                                UpdateType::Major => { major += 1; minor = 0; patch = 0; },
                                                UpdateType::Minor => { minor += 1; patch = 0; },
                                                UpdateType::Patch => { patch += 1; },
                                            }
                                            
                                            format!("{}_V{}.{}.{}", base_name, major, minor, patch)
                                        };

                                        view! {
                                            <div class="analytics-content">
                                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;">
                                                    <h2 style="margin: 0;">"Danh sách Phiên bản (Firmwares)"</h2>
                                                    <button 
                                                        on:click=move |_| set_show_upload_modal.set(true)
                                                        style="background: #FFCA28; color: #000; font-weight: 600; padding: 0.6rem 1.2rem; border: none; border-radius: 6px; cursor: pointer; display: flex; align-items: center; gap: 0.5rem;"
                                                    >
                                                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg>
                                                        "Tải lên phiên bản mới"
                                                    </button>
                                                </div>

                                                {{
                                                    let p_id_for_modal = p_id_for_upload.clone();
                                                    move || if show_upload_modal.get() {
                                                    let calc_for_view = calculated_version.clone();
                                                    let calc_for_upload = calculated_version.clone();
                                                    let id_for_upload = p_id_for_modal.clone();
                                                    
                                                    view! {
                                                        <div class="modal-overlay" style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000; backdrop-filter: blur(4px);">
                                                            <div class="modal-content" style="background: #2a2a2c; border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 12px; padding: 2rem; width: 500px; max-width: 90vw;">
                                                                <h3 style="margin-top: 0; margin-bottom: 1.5rem; font-size: 1.2rem; color: #ffca28; border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 1rem;">"Tải lên phiên bản mới"</h3>
                                                                
                                                                <div style="margin-bottom: 1.5rem;">
                                                                    <label style="display: block; font-size: 0.85rem; color: #b0bec5; margin-bottom: 0.5rem;">"Tên phiên bản:"</label>
                                                                    <div style="width: 100%; background: #1e1e1e; border: 1px solid #424242; border-radius: 6px; padding: 0.6rem 1rem; color: #90a4ae; font-size: 0.95rem; user-select: none;">
                                                                        {move || calc_for_view()}
                                                                    </div>
                                                                    
                                                                    <div style="display: flex; gap: 1rem; margin-top: 0.8rem; font-size: 0.85rem; color: #fff;">
                                                                        <label style="display: flex; align-items: center; gap: 0.4rem; cursor: pointer;">
                                                                            <input type="radio" name="update_type_modal" prop:checked=move || update_type.get() == UpdateType::Major on:click=move |_| set_update_type.set(UpdateType::Major) />
                                                                            "Bản chính thức"
                                                                        </label>
                                                                        <label style="display: flex; align-items: center; gap: 0.4rem; cursor: pointer;">
                                                                            <input type="radio" name="update_type_modal" prop:checked=move || update_type.get() == UpdateType::Minor on:click=move |_| set_update_type.set(UpdateType::Minor) />
                                                                            "Bản bổ sung"
                                                                        </label>
                                                                        <label style="display: flex; align-items: center; gap: 0.4rem; cursor: pointer;">
                                                                            <input type="radio" name="update_type_modal" prop:checked=move || update_type.get() == UpdateType::Patch on:click=move |_| set_update_type.set(UpdateType::Patch) />
                                                                            "Bản vá lỗi"
                                                                        </label>
                                                                    </div>
                                                                </div>
                                                                
                                                                <div style="margin-bottom: 1.5rem;">
                                                                    <label style="display: block; font-size: 0.85rem; color: #b0bec5; margin-bottom: 0.5rem;">"Ghi chú (Có thể bỏ qua):"</label>
                                                                    <textarea 
                                                                        style="width: 100%; background: #1e1e1e; border: 1px solid #424242; border-radius: 6px; padding: 0.6rem 1rem; color: #fff; font-size: 0.95rem; outline: none; font-family: inherit; resize: vertical;"
                                                                        rows="2"
                                                                        placeholder="VD: Sửa lỗi kết nối, cập nhật giao diện..."
                                                                        prop:value=move || fw_notes.get()
                                                                        on:input=move |ev| set_fw_notes.set(event_target_value(&ev))
                                                                    ></textarea>
                                                                </div>
                                                                
                                                                <div style="margin-bottom: 1.5rem;">
                                                                    <FileUploadDropzone 
                                                                        on_files_select={move |files: Vec<web_sys::File>| {
                                                                            if let Some(f) = files.into_iter().next() {
                                                                                set_fw_file.set(Some(f));
                                                                            }
                                                                        }}
                                                                        on_clear={move |_| set_fw_file.set(None)}
                                                                        title="Tải lên Firmware".to_string()
                                                                        description="Kéo thả hoặc click để chọn file .bin".to_string()
                                                                        accept=".bin".to_string()
                                                                    />
                                                                </div>
                                                                
                                                                <div style="display: flex; justify-content: space-between; align-items: center;">
                                                                    <button 
                                                                        on:click=move |_| {
                                                                            set_show_upload_modal.set(false);
                                                                            set_upload_status.set("".to_string());
                                                                            set_fw_file.set(None);
                                                                            set_fw_notes.set("".to_string());
                                                                        }
                                                                        style="background: transparent; border: 1px solid #90a4ae; color: #90a4ae; padding: 0.6rem 1.5rem; border-radius: 6px; cursor: pointer;"
                                                                    >
                                                                        "Quay lại"
                                                                    </button>
                                                                    <div style="display: flex; align-items: center; gap: 1rem;">
                                                                        <span style="font-size: 0.9rem; color: #ff8a65;">{move || upload_status.get()}</span>
                                                                        <button 
                                                                            style="background: #82b1ff; color: #000; font-weight: 600; padding: 0.6rem 1.5rem; border: none; border-radius: 6px; cursor: pointer; display: flex; align-items: center; gap: 0.5rem;"
                                                                            disabled=is_uploading
                                                                            on:click=move |_| {
                                                                                let id = id_for_upload.clone();
                                                                                let version = calc_for_upload();
                                                                                let file = fw_file.get();
                                                                                let notes = fw_notes.get();
                                                                                
                                                                                if let Some(file) = file {
                                                                                    set_is_uploading.set(true);
                                                                                    set_upload_status.set("Đang tải lên...".to_string());
                                                                                    
                                                                                    let form_data = web_sys::FormData::new().unwrap();
                                                                                    form_data.append_with_str("version", &version).unwrap();
                                                                                    form_data.append_with_blob("file", &file).unwrap();
                                                                                    form_data.append_with_str("notes", &notes).unwrap();
                                                                                    
                                                                                    spawn_local(async move {
                                                                                        let res = gloo_net::http::Request::post(&format!("http://localhost:7424/api/projects/{}/firmware", id))
                                                                                            .credentials(web_sys::RequestCredentials::Include)
                                                                                            .body(form_data).unwrap()
                                                                                            .send().await;
                                                                                            
                                                                                        set_is_uploading.set(false);
                                                                                        match res {
                                                                                            Ok(r) if r.ok() => {
                                                                                                set_upload_status.set("Tải lên thành công!".to_string());
                                                                                                set_fw_file.set(None);
                                                                                                set_fw_notes.set("".to_string());
                                                                                                set_show_upload_modal.set(false);
                                                                                                project_resource.refetch();
                                                                                            },
                                                                                            _ => {
                                                                                                set_upload_status.set("Lỗi tải lên!".to_string());
                                                                                            }
                                                                                        }
                                                                                    });
                                                                                } else {
                                                                                    set_upload_status.set("Vui lòng chọn file .bin".to_string());
                                                                                }
                                                                            }
                                                                        >
                                                                            {move || if is_uploading.get() { "Đang xử lý..." } else { "Hoàn tất" }}
                                                                        </button>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view!{}.into_view()
                                                }
                                                }}

                                                <div style="background: #2a2a2c; border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 12px; padding: 1.5rem; color: #fff;">
                                                    {
                                                        if detail_inner.firmwares.is_empty() {
                                                            view! { <div style="color: #90a4ae;">"Chưa có phiên bản nào được phát hành."</div> }.into_view()
                                                        } else {
                                                            let total_fws = detail_inner.firmwares.len();
                                                            let fw_list = detail_inner.firmwares.clone().into_iter().map(|fw| {
                                                                let fw_version_for_delete = fw.version.clone();
                                                                let p_id_for_delete = p_id_for_upload.clone();
                                                                view! {
                                                                    <div class="fw-row" style="display: flex; justify-content: space-between; align-items: center;">
                                                                        <div>
                                                                            <div style="font-weight: 500; font-size: 1.1rem; color: #82b1ff;">
                                                                                {fw.version}
                                                                            </div>
                                                                            <div style="font-size: 0.85rem; color: #90a4ae; margin-top: 0.25rem;">"Ghi chú: " {fw.notes.clone().unwrap_or_else(|| "Không có".to_string())}</div>
                                                                        </div>
                                                                        <div style="display: flex; align-items: center; gap: 1.5rem;">
                                                                            <div style="text-align: right;">
                                                                                <div style="font-size: 0.9rem;">{fw.created_at}</div>
                                                                                <div style="font-size: 0.8rem; color: #a5d6a7; margin-top: 0.25rem;">{fw.devices_count} " thiết bị"</div>
                                                                            </div>
                                                                            {
                                                                                if total_fws > 1 {
                                                                                    view! {
                                                                                        <button
                                                                                            class="fw-delete-btn"
                                                                                            title="Xóa phiên bản này"
                                                                                            on:click=move |_| {
                                                                                                if window().confirm_with_message("Bạn có chắc chắn muốn xóa phiên bản này không?").unwrap_or(false) {
                                                                                                    let pid = p_id_for_delete.clone();
                                                                                                    let ver = fw_version_for_delete.clone();
                                                                                                    spawn_local(async move {
                                                                                                        let res = gloo_net::http::Request::delete(&format!("http://localhost:7424/api/projects/{}/firmware/{}", pid, ver))
                                                                                                            .credentials(web_sys::RequestCredentials::Include)
                                                                                                            .send().await;
                                                                                                        if let Ok(r) = res {
                                                                                                            if r.ok() {
                                                                                                                project_resource.refetch();
                                                                                                            } else {
                                                                                                                window().alert_with_message("Lỗi khi xóa phiên bản.").unwrap();
                                                                                                            }
                                                                                                        }
                                                                                                    });
                                                                                                }
                                                                                            }
                                                                                        >
                                                                                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                                                                                        </button>
                                                                                    }.into_view()
                                                                                } else {
                                                                                    view! {
                                                                                        <button
                                                                                            class="fw-delete-btn"
                                                                                            title="Không thể xóa khi chỉ còn 1 phiên bản"
                                                                                            disabled=true
                                                                                        >
                                                                                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                                                                                        </button>
                                                                                    }.into_view()
                                                                                }
                                                                            }
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }).collect_view();
                                                            view! { <div>{fw_list}</div> }.into_view()
                                                        }
                                                    }
                                                </div>
                                            </div>
                                        }.into_view()
                                    }
                                    }}
                                    }.into_view()
                                },
                                Some(Err(e)) => view! { <div class="error-state">"Lỗi: " {e}</div> }.into_view(),
                                None => view! {}.into_view(),
                            }
                        }}
                    </Suspense>
                </main>
            </div>
        </div>
    }
}
