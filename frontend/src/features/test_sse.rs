// src/features/test_sse.rs
use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{EventSource, MessageEvent};

/****
 * Component: TestSse
 * Chức năng: Mở kết nối Server-Sent Events và hiển thị log thời gian thực.
 ****/
#[component]
pub fn TestSse() -> impl IntoView {
    let (logs, set_logs) = create_signal(String::new());
    let (is_connected, set_connected) = create_signal(false);

    let connect_sse = move |_| {
        if is_connected.get() { return; }

        if let Ok(es) = EventSource::new("http://localhost:5000/api/v1/events") {
            set_connected.set(true);
            set_logs.update(|l| l.push_str("Đang kết nối SSE...\n"));

            // Xử lý sự kiện chung (nếu backend gửi không có tên)
            let onmessage = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Some(text) = e.data().as_string() {
                    set_logs.update(|l| l.push_str(&format!(">> [Message] {}\n", text)));
                }
            }) as Box<dyn FnMut(_)>);
            es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();

            // Xử lý sự kiện "ws_chat" (khi có người nhắn tin trên WebSocket)
            let on_ws_chat = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Some(text) = e.data().as_string() {
                    set_logs.update(|l| l.push_str(&format!(">> [ws_chat] {}\n", text)));
                }
            }) as Box<dyn FnMut(_)>);
            let _ = es.add_event_listener_with_callback("ws_chat", on_ws_chat.as_ref().unchecked_ref());
            on_ws_chat.forget();

            // Xử lý sự kiện "alert" (khi bấm nút Broadcast)
            let on_alert = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Some(text) = e.data().as_string() {
                    set_logs.update(|l| l.push_str(&format!(">> [alert] {}\n", text)));
                }
            }) as Box<dyn FnMut(_)>);
            let _ = es.add_event_listener_with_callback("alert", on_alert.as_ref().unchecked_ref());
            on_alert.forget();
        } else {
            set_logs.set("Trình duyệt không hỗ trợ EventSource hoặc lỗi URL".to_string());
        }
    };

    let send_broadcast = move |_| {
        spawn_local(async move {
            let payload = serde_json::json!({ "topic": "alert", "message": "Test Broadcast từ SSE Card!" });
            let _ = gloo_net::http::Request::post("http://localhost:5000/api/v1/events/broadcast")
                .json(&payload)
                .unwrap()
                .send()
                .await;
        });
    };

    view! {
        <h2>"Server-Sent Events (SSE)"</h2>
        <div class="form-group" style="flex-direction: row; gap: 8px;">
            <button on:click=connect_sse disabled=is_connected>
                {move || if is_connected.get() { "Đang lắng nghe..." } else { "Bắt đầu nghe" }}
            </button>
            <button on:click=send_broadcast>"Gửi Broadcast"</button>
        </div>
        <div class="response-box" style="margin-top: 16px;">
            {logs}
        </div>
    }
}
