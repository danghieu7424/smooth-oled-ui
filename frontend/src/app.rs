// src/app.rs
use leptos::*;
use leptos_router::*;

use crate::store::init_global_state;

use crate::pages::dashboard::DashboardPage;
use crate::pages::project_detail::ProjectDetailPage;
use crate::pages::project_new::ProjectNewPage;
use crate::pages::login::LoginPage;

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
                        <Route path="/" view=DashboardPage />
                        <Route path="/login" view=LoginPage />
                        <Route path="/projects/new" view=ProjectNewPage />
                        <Route path="/projects/:id" view=ProjectDetailPage />
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
