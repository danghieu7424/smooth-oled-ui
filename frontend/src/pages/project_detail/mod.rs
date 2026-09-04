use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use crate::pages::dashboard::fetch_me;

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
    
    let (active_tab, set_active_tab) = create_signal("dashboard".to_string());
    
    let project_resource = create_resource(
        move || id_str(),
        |id| async move {
            if id.is_empty() { return Err("No ID".to_string()); }
            fetch_project_detail(id).await
        }
    );

    view! {
        <div class="firebase-layout firebase-project-detail">


            <div class="detail-container">
                // Sidebar
                <aside class="fb-sidebar">
                    <div class="sidebar-group">
                        <div class=move || if active_tab.get() == "dashboard" { "sidebar-item active" } else { "sidebar-item" } on:click=move |_| set_active_tab.set("dashboard".to_string())>
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                            <span>"Dashboard"</span>
                        </div>
                    </div>
                    
                    <div class="sidebar-title">"QUẢN LÝ"</div>
                    <div class="sidebar-group">
                        <div class=move || if active_tab.get() == "versions" { "sidebar-item active" } else { "sidebar-item" } on:click=move |_| set_active_tab.set("versions".to_string())>
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
                                Some(Ok(detail)) => view! {
                                    <div class="analytics-header">
                                        <div class="title-row">
                                            <div class="main-header">
                                                {move || {
                                                    let api_link = format!("http://localhost:7424/api/firmware/{}-{}", detail.user_suid, detail.project_id);
                                                    view! {
                                                        <div>
                                                            <h1 style="color: #fff; font-size: 1.5rem; margin-bottom: 0.5rem;">{detail.name.clone()} <span class="badge">"Pro"</span></h1>
                                                            <div class="project-info" style="color: #90a4ae; font-size: 0.9rem; font-family: monospace;">
                                                                <div style="margin-bottom: 0.25rem;">"ID: " <span style="color: #82b1ff;">{detail.project_id.clone()}</span></div>
                                                                <div>"Link Update: " <span style="color: #a5d6a7;">{api_link}</span></div>
                                                            </div>
                                                        </div>
                                                    }.into_view()
                                                }}
                                            </div>
                                        </div>
                                    </div>

                                    {move || if active_tab.get() == "dashboard" {
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
                                        view! {
                                            <div class="analytics-content">
                                                <h2>"Danh sách Phiên bản (Firmwares)"</h2>
                                                <div style="background: #2a2a2c; border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 12px; padding: 1.5rem; color: #fff;">
                                                    {
                                                        if detail.firmwares.is_empty() {
                                                            view! { <div style="color: #90a4ae;">"Chưa có phiên bản nào được phát hành."</div> }.into_view()
                                                        } else {
                                                            let fw_list = detail.firmwares.clone().into_iter().map(|fw| {
                                                                view! {
                                                                    <div style="padding: 1rem 0; border-bottom: 1px solid rgba(255,255,255,0.1); display: flex; justify-content: space-between; align-items: center;">
                                                                        <div>
                                                                            <div style="font-weight: 500; font-size: 1.1rem; color: #82b1ff;">{fw.version}</div>
                                                                            <div style="font-size: 0.85rem; color: #90a4ae; margin-top: 0.25rem;">"Ghi chú: " {fw.notes.clone().unwrap_or_else(|| "Không có".to_string())}</div>
                                                                        </div>
                                                                        <div style="text-align: right;">
                                                                            <div style="font-size: 0.9rem;">{fw.created_at}</div>
                                                                            <div style="font-size: 0.8rem; color: #a5d6a7; margin-top: 0.25rem;">{fw.devices_count} " thiết bị"</div>
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
                                    }}
                                }.into_view(),
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
