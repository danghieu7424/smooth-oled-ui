use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub error: Option<String>,
}

pub async fn fetch_me() -> Result<UserProfile, String> {
    gloo_net::http::Request::get("http://localhost:7424/api/auth/me")
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<UserProfile>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_projects: i64,
    pub total_devices: i64,
    pub total_updates: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub user_id: i64,
    pub project_id: String,
    pub name: String,
    pub created_at: String,
    pub version: Option<String>,
    pub is_starred: bool,
}

async fn fetch_stats() -> Result<Stats, String> {
    gloo_net::http::Request::get("http://localhost:7424/api/projects/stats")
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Stats>()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_projects() -> Result<Vec<Project>, String> {
    gloo_net::http::Request::get("http://localhost:7424/api/projects")
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<Project>>()
        .await
        .map_err(|e| e.to_string())
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let (filter_mode, set_filter_mode) = create_signal("all".to_string());
    let (show_dropdown, set_show_dropdown) = create_signal(false);
    let projects_resource = create_resource(|| (), |_| async move { fetch_projects().await });
    let me_resource = create_resource(|| (), |_| async move { fetch_me().await });

    view! {
        <div class="firebase-layout">
            // Top Navigation Bar
            <header class="fb-header">
                <div class="fb-header-left">
                    <A href="/" class="logo">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="#FFA000"><path d="M11.64 5.93h.01L15.8 13.5l1.83-3.19a.53.53 0 0 1 .9 0l4.31 7.5a.51.51 0 0 1-.44.78H1.61a.51.51 0 0 1-.45-.77l6.83-11.96a.53.53 0 0 1 .9 0l1.43 2.5 1.32-2.43a.52.52 0 0 1 .9 0z"/></svg>
                        <span>"OTA Hub"</span>
                    </A>
                </div>
                <div class="fb-header-right">
                    <div class="nav-icon">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path><path d="M13.73 21a2 2 0 0 1-3.46 0"></path></svg>
                    </div>
                    <Suspense fallback=move || view! { <div></div> }>
                        {move || {
                            let me = me_resource.get().unwrap_or(Ok(UserProfile { id: None, name: None, picture: None, error: Some("Not loaded".to_string()) })).unwrap_or(UserProfile { id: None, name: None, picture: None, error: Some("Not loaded".to_string()) });
                            if me.error.is_none() && me.name.is_some() {
                                view! {
                                    <div class="user-menu" style="display: flex; align-items: center; gap: 0.5rem; color: #fff;">
                                        <img src=me.picture.unwrap_or_default() alt="Avatar" class="avatar" style="width: 28px; height: 28px; border-radius: 50%;" />
                                        <span>{me.name.unwrap_or_default()}</span>
                                        <a href="http://localhost:7424/api/auth/logout" style="color: #ff8a65; font-size: 0.8rem; margin-left: 0.5rem; text-decoration: none;">"Đăng xuất"</a>
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <A href="/login" class="login-btn">
                                        <img src="https://ui-avatars.com/api/?name=Guest&background=333&color=fff" alt="Avatar" class="avatar" />
                                        <span>"Đăng nhập"</span>
                                    </A>
                                }.into_view()
                            }
                        }}
                    </Suspense>
                </div>
            </header>

            <main class="fb-main">
                <div class="greeting-section">
                    <Suspense fallback=move || view! { <h1>"Xin chào"</h1> }>
                        {move || {
                            let me = me_resource.get().unwrap_or(Ok(UserProfile { id: None, name: None, picture: None, error: Some("Not loaded".to_string()) })).unwrap_or(UserProfile { id: None, name: None, picture: None, error: Some("Not loaded".to_string()) });
                            if me.error.is_none() && me.name.is_some() {
                                view! { <h1>"Xin chào " {me.name.unwrap_or_default()}</h1> }.into_view()
                            } else {
                                view! { <h1>"Xin chào"</h1> }.into_view()
                            }
                        }}
                    </Suspense>
                    <p>"Chào mừng đến với OTA Hub"</p>
                </div>

                <div class="content-grid">
                    // Cột trái: Actions
                    <div class="actions-column">
                        <div class="section">
                            <h3 class="section-title">"Bắt đầu"</h3>
                            <A href="/projects/new" class="action-card primary-action">
                                <div class="action-icon">
                                    <svg width="24" height="24" viewBox="0 0 24 24" fill="#FFCA28"><path d="M12 2L9 9l-7 1 5 5-2 7 7-4 7 4-2-7 5-5-7-1-3-7z"/></svg>
                                </div>
                                <div class="action-text">
                                    <h4>"Tạo một dự án mới"</h4>
                                    <p>"Tích hợp OTA Firmware để nâng cấp thiết bị của bạn từ xa"</p>
                                </div>
                            </A>
                        </div>

                        <div class="section">
                            <h3 class="section-title">"Khám phá thêm"</h3>
                            <div class="action-card">
                                <div class="action-icon">
                                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#FF8A65" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/></svg>
                                </div>
                                <div class="action-text">
                                    <h4>"Xem tài liệu tích hợp ESP32"</h4>
                                    <p>"Hướng dẫn cài đặt thư viện HTTPUpdate trên ESP32/ESP8266"</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Cột phải: Projects List
                    <div class="projects-column">
                        <div class="search-bar">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
                            <input type="text" placeholder="Tìm kiếm trong tất cả dự án..." />
                        </div>

                        <div class="project-list-container">
                            <div class="project-list-header" style="position: relative; padding: 0.75rem 1.25rem; border-bottom: 1px solid rgba(255, 255, 255, 0.08);">
                                <div 
                                    class="dropdown"
                                    style="display: inline-flex; align-items: center; gap: 0.5rem; cursor: pointer; color: #b0bec5; font-size: 0.95rem; font-weight: 500;"
                                    on:click=move |_| set_show_dropdown.update(|s| *s = !*s)
                                >
                                    <span>{move || if filter_mode.get() == "all" { "Dự án" } else { "Dự án đánh dấu" }}</span>
                                    <span style="font-size: 0.7rem; transition: transform 0.2s;" style:transform=move || if show_dropdown.get() { "rotate(180deg)" } else { "rotate(0deg)" }>"▼"</span>
                                </div>
                                
                                {move || if show_dropdown.get() {
                                    view! {
                                        <div style="position: absolute; top: calc(100% + 4px); left: 1.25rem; background: #2a2a2c; border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 8px; padding: 0.5rem 0; min-width: 220px; z-index: 10; box-shadow: 0 8px 24px rgba(0,0,0,0.5);">
                                            <div 
                                                style=move || format!("padding: 0.6rem 1.25rem; cursor: pointer; font-size: 0.9rem; transition: background 0.2s; color: {}; background: {};", 
                                                    if filter_mode.get() == "all" { "#82b1ff" } else { "#b0bec5" },
                                                    if filter_mode.get() == "all" { "rgba(130, 177, 255, 0.1)" } else { "transparent" }
                                                )
                                                on:click=move |_| { set_filter_mode.set("all".to_string()); set_show_dropdown.set(false); }
                                            >
                                                "Dự án"
                                            </div>
                                            <div 
                                                style=move || format!("padding: 0.6rem 1.25rem; cursor: pointer; font-size: 0.9rem; transition: background 0.2s; display: flex; align-items: center; justify-content: space-between; color: {}; background: {};", 
                                                    if filter_mode.get() == "starred" { "#82b1ff" } else { "#b0bec5" },
                                                    if filter_mode.get() == "starred" { "rgba(130, 177, 255, 0.1)" } else { "transparent" }
                                                )
                                                on:click=move |_| { set_filter_mode.set("starred".to_string()); set_show_dropdown.set(false); }
                                            >
                                                <span>"Dự án đánh dấu"</span>
                                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon></svg>
                                            </div>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {}.into_view()
                                }}
                            </div>
                            
                            <Suspense fallback=move || view! { <div class="loading-state">"Đang tải danh sách..."</div> }>
                                {move || {
                                    match projects_resource.get() {
                                        Some(Ok(projects)) => {
                                            if projects.is_empty() {
                                                view! { <div class="empty-state">"Bạn chưa có dự án nào."</div> }.into_view()
                                            } else {
                                                let filtered_projects: Vec<_> = projects.into_iter().filter(|p| {
                                                    if filter_mode.get() == "starred" { p.is_starred } else { true }
                                                }).collect();
                                                
                                                if filtered_projects.is_empty() {
                                                    return view! { <div class="empty-state">"Không tìm thấy dự án nào."</div> }.into_view();
                                                }

                                                let count = filtered_projects.len();
                                                let list_view = filtered_projects.into_iter().map(|p| {
                                                    let p_id_clone_star = p.project_id.clone();
                                                    let p_id_clone_del = p.project_id.clone();
                                                    let p_id = p.id;
                                                    let is_starred = p.is_starred;
                                                    let p_id_str = p.project_id.clone();
                                                    view! {
                                                        <div class="project-list-item" style="cursor: pointer;" on:click=move |_| {
                                                            let navigate = use_navigate();
                                                            navigate(&format!("/projects/{}", p_id_str), Default::default());
                                                        }>
                                                            <div class="p-icon">
                                                                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 2.69l5.66 4.2c.22.16.34.42.34.69v8.42c0 .27-.12.53-.34.69L12 20.89l-5.66-4.2a1.14 1.14 0 0 1-.34-.69V7.58c0-.27.12-.53.34-.69L12 2.69z"/></svg>
                                                            </div>
                                                            <div class="p-info">
                                                                <div class="p-name">{p.name.clone()}</div>
                                                            </div>
                                                            <div class="p-actions" style="display: flex; gap: 0.75rem; align-items: center;">
                                                                <div class="p-star" on:click=move |ev| {
                                                                    ev.prevent_default();
                                                                    ev.stop_propagation();
                                                                    let id = p_id_clone_star.clone();
                                                                    spawn_local(async move {
                                                                        let _ = gloo_net::http::Request::patch(&format!("http://localhost:7424/api/projects/{}/star", id))
                                                                            .credentials(web_sys::RequestCredentials::Include)
                                                                            .send().await;
                                                                        projects_resource.refetch();
                                                                    });
                                                                }>
                                                                    <svg width="18" height="18" viewBox="0 0 24 24" fill=if is_starred { "#ffca28" } else { "none" } stroke=if is_starred { "#ffca28" } else { "currentColor" } stroke-width="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon></svg>
                                                                </div>
                                                                <div class="p-del" style="color: #ef5350; cursor: pointer; transition: color 0.2s;" on:click=move |ev| {
                                                                    ev.prevent_default();
                                                                    ev.stop_propagation();
                                                                    let id = p_id_clone_del.clone();
                                                                    if web_sys::window().unwrap().confirm_with_message("Bạn có chắc chắn muốn xóa dự án này? Thao tác không thể hoàn tác!").unwrap_or(false) {
                                                                        spawn_local(async move {
                                                                            let _ = gloo_net::http::Request::delete(&format!("http://localhost:7424/api/projects/{}", id))
                                                                                .credentials(web_sys::RequestCredentials::Include)
                                                                                .send().await;
                                                                            projects_resource.refetch();
                                                                        });
                                                                    }
                                                                }>
                                                                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view();
                                                
                                                view! {
                                                    <div class="list-wrapper">
                                                        {list_view}
                                                    </div>
                                                    <div class="list-footer">
                                                        <span>{format!("1 – {} trên {}", count, count)}</span>
                                                        <div class="pagination">
                                                            <button class="icon-btn">"<"</button>
                                                            <button class="icon-btn">">"</button>
                                                        </div>
                                                    </div>
                                                }.into_view()
                                            }
                                        },
                                        Some(Err(_)) => view! { <div class="error-state">"Lỗi tải danh sách dự án."</div> }.into_view(),
                                        None => view! {}.into_view(),
                                    }
                                }}
                            </Suspense>
                        </div>
                    </div>
                </div>
            </main>
        </div>
    }
}
