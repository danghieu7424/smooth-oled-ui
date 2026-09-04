use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn LoginPage() -> impl IntoView {
    let navigate = use_navigate();
    let nav = navigate.clone();

    // Lắng nghe MessageEvent từ cửa sổ popup
    let cb = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
        if let Some(data) = event.data().as_string() {
            if data == "login_success" {
                nav("/", Default::default());
            }
        }
    }) as Box<dyn FnMut(_)>);

    let window = web_sys::window().unwrap();
    let cb_ref = cb.as_ref().unchecked_ref();
    window.add_event_listener_with_callback("message", cb_ref).unwrap();
    
    on_cleanup({
        let window = window.clone();
        move || {
            let _ = window.remove_event_listener_with_callback("message", cb.as_ref().unchecked_ref());
        }
    });

    let on_login = move |_| {
        let window = web_sys::window().unwrap();
        let _ = window.open_with_url_and_target_and_features(
            "http://localhost:7424/api/auth/google",
            "google_login",
            "width=500,height=600,left=400,top=200"
        );
    };

    view! {
        <div class="login-page">
            <div class="login-card glass-panel">
                <div class="login-header">
                    <h2>"Hệ thống Quản lý Firmware OTA"</h2>
                    <p>"Đăng nhập để tiếp tục"</p>
                </div>
                
                <div class="login-actions">
                    <button on:click=on_login class="btn btn-primary google-btn">
                        <span class="icon">"G"</span>
                        " Đăng nhập với Google"
                    </button>
                </div>
            </div>
        </div>
    }
}
