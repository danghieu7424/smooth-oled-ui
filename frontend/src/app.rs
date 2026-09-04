// src/app.rs
use leptos::*;
use leptos_router::*;

use crate::pages::home_page::HomePage;
use crate::store::init_global_state;

use crate::features::file_browser::page::FileBrowserPage;

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
                        <Route path="/*path" view=FileBrowserPage />
                        <Route path="/demo" view=HomePage />
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
