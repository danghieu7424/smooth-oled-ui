// src/app.rs
use leptos::*;
use leptos_router::*;

use crate::store::init_global_state;

use crate::pages::dashboard::{DashboardPage, UserProfile, fetch_me};
use crate::pages::project_detail::ProjectDetailPage;
use crate::pages::project_new::ProjectNewPage;
use crate::pages::login::LoginPage;

#[component]
pub fn MainLayout() -> impl IntoView {
    let me_resource = create_resource(|| (), |_| async move { fetch_me().await });
    
    view! {
        <div class="firebase-layout" style="min-height: 100vh; display: flex; flex-direction: column;">
            <header class="fb-header">
                <div class="fb-header-left">
                    <A href="/" class="logo">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="#FFA000"><path d="M11.64 5.93h.01L15.8 13.5l1.83-3.19a.53.53 0 0 1 .9 0l4.31 7.5a.51.51 0 0 1-.44.78H1.61a.51.51 0 0 1-.45-.77l6.83-11.96a.53.53 0 0 1 .9 0l1.43 2.5 1.32-2.43a.52.52 0 0 1 .9 0z"/></svg>
                        <span>"OTA Hub"</span>
                    </A>
                </div>
                <div class="fb-header-right">

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
            <Outlet/>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Khởi tạo Global State
    let state = init_global_state();
    provide_context(state);

    view! {
        <Router>
            <div class="super-app-layout">
                <main class="main-content">
                    <Routes>
                        <Route path="/" view=MainLayout>
                            <Route path="" view=DashboardPage />
                            <Route path="projects/new" view=ProjectNewPage />
                            <Route path="projects/:id" view=ProjectDetailPage />
                            <Route path="projects/:id/:tab" view=ProjectDetailPage />
                        </Route>
                        <Route path="/login" view=LoginPage />
                        <Route
                            path="/*any"
                            view=|| {
                                view! {
                                    <div class="not-found">
                                        <h1>"404 - Not Found"</h1>
                                        <p>"Đường dẫn không tồn tại."</p>
                                    </div>
                                }
                            }
                        />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
