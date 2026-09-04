use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

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
    
    let project_resource = create_resource(
        move || id_str(),
        |id| async move {
            if id.is_empty() { return Err("No ID".to_string()); }
            fetch_project_detail(id).await
        }
    );

    view! {
        <div class="firebase-layout firebase-project-detail">
            <header class="fb-header">
                <div class="fb-header-left">
                    <div class="logo">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="#FFA000"><path d="M11.64 5.93h.01L15.8 13.5l1.83-3.19a.53.53 0 0 1 .9 0l4.31 7.5a.51.51 0 0 1-.44.78H1.61a.51.51 0 0 1-.45-.77l6.83-11.96a.53.53 0 0 1 .9 0l1.43 2.5 1.32-2.43a.52.52 0 0 1 .9 0z"/></svg>
                        <span>"OTA Hub"</span>
                    </div>
                    <span class="header-divider"></span>
                    <span class="project-name-header">
                        {move || match project_resource.get() {
                            Some(Ok(p)) => format!("{} ▼", p.name),
                            _ => "Đang tải...".to_string()
                        }}
                    </span>
                </div>
                <div class="fb-header-right">
                    <div class="nav-icon">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
                    </div>
                    <A href="/" class="nav-icon back-home">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg>
                    </A>
                </div>
            </header>

            <div class="detail-container">
                // Sidebar
                <aside class="fb-sidebar">
                    <div class="sidebar-group">
                        <A href="/" class="sidebar-item">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path></svg>
                            <span>"Dashboard"</span>
                        </A>
                        <div class="sidebar-item active">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                            <span>"Chi tiết dự án"</span>
                        </div>
                    </div>
                    
                    <div class="sidebar-title">"QUẢN LÝ"</div>
                    <div class="sidebar-group">
                        <div class="sidebar-item">
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
                                                    let api_link = format!("http://localhost:7424/api/firmware/{}/{}", detail.user_suid, detail.project_id);
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
                                                    <h3>"Thiết bị hoạt động (Active)"</h3>
                                                    <div class="chart-val">
                                                        <span>{detail.active_devices}</span>
                                                        <span class="trend">"--"</span>
                                                    </div>
                                                    <div class="chart-area empty">
                                                        "Chưa có dữ liệu"
                                                        <small>"trong 14 ngày qua"</small>
                                                    </div>
                                                </div>
                                                
                                                <div class="chart-column">
                                                    <h3>"Số bản phát hành (Firmwares)"</h3>
                                                    <div class="chart-val">
                                                        <span>{detail.firmwares.len()}</span>
                                                        <span class="trend">"--"</span>
                                                    </div>
                                                    <div class="chart-area line-chart">
                                                        <svg viewBox="0 0 100 40" preserveAspectRatio="none">
                                                            // Mô phỏng biểu đồ đường dốc lên SVG
                                                            <path d="M0,35 L20,30 L40,32 L60,15 L80,20 L100,5" fill="none" stroke="#69db7c" stroke-width="2"></path>
                                                            <path d="M0,35 L20,30 L40,32 L60,15 L80,20 L100,5 L100,40 L0,40 Z" fill="rgba(105, 219, 124, 0.1)" stroke="none"></path>
                                                        </svg>
                                                    </div>
                                                </div>

                                                <div class="chart-promo">
                                                    <div class="promo-img">
                                                        <svg width="60" height="60" viewBox="0 0 24 24" fill="none" stroke="#82b1ff" stroke-width="1.5"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>
                                                    </div>
                                                    <div class="promo-text">
                                                        <p>"Tích hợp nền tảng!"</p>
                                                        <a href="#">"Tài liệu API"</a>
                                                        <a href="#">"Tài liệu ESP-IDF"</a>
                                                    </div>
                                                </div>
                                            </div>
                                            
                                            <div class="card-footer">
                                                <span class="legend"><span class="color-box this-week"></span> "Tuần này"</span>
                                                <span class="legend"><span class="color-box last-week"></span> "Tuần trước"</span>
                                            </div>
                                        </div>
                                    </div>
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
