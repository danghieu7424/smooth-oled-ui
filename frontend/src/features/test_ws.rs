// src/features/test_ws.rs
use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent};

/****
 * Component: TestWs
 * Chức năng: Giao tiếp qua WebSocket. Gửi tin nhắn và nhận phản hồi (echo).
 ****/
#[component]
pub fn TestWs() -> impl IntoView {
    let (logs, set_logs) = create_signal(String::new());
    let (message, set_message) = create_signal(String::new());
    let (socket, set_socket) = create_signal::<Option<WebSocket>>(None);

    let connect_ws = move |_| {
        if socket.get().is_some() { return; }

        let window = web_sys::window().unwrap();
        let host = window.location().host().unwrap();
        let ws_url = if host.contains("localhost") || host.contains("127.0.0.1") {
            "ws://localhost:5000/api/v1/ws".to_string()
        } else {
            format!("wss://{}/api/v1/ws", host)
        };

        if let Ok(ws) = WebSocket::new(&ws_url) {
            set_logs.update(|l| l.push_str("Đang mở kết nối WebSocket...\n"));
            
            let onmessage = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Some(text) = e.data().as_string() {
                    set_logs.update(|l| l.push_str(&format!("<- {}\n", text)));
                }
            }) as Box<dyn FnMut(_)>);
            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();

            let onopen = wasm_bindgen::closure::Closure::wrap(Box::new(move |_| {
                set_logs.update(|l| l.push_str("Kết nối thành công!\n"));
            }) as Box<dyn FnMut(web_sys::Event)>);
            ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            onopen.forget();

            set_socket.set(Some(ws));
        }
    };

    let send_msg = move |_| {
        if let Some(ws) = socket.get() {
            let msg = message.get();
            if !msg.is_empty() {
                if ws.send_with_str(&msg).is_ok() {
                    set_logs.update(|l| l.push_str(&format!("-> {}\n", msg)));
                    set_message.set("".to_string());
                }
            }
        }
    };

    view! {
        <h2>"WebSocket Chat"</h2>
        <div class="form-group">
            <button on:click=connect_ws disabled=move || socket.get().is_some()>
                {move || if socket.get().is_some() { "Đã kết nối" } else { "Kết nối WebSocket" }}
            </button>
        </div>
        <div class="form-group" style="flex-direction: row; gap: 8px; margin-top: 8px;">
            <input type="text" placeholder="Gõ tin nhắn..." style="flex: 1"
                on:input=move |ev| set_message.set(event_target_value(&ev))
                prop:value=message
                disabled=move || socket.get().is_none()
            />
            <button on:click=send_msg disabled=move || socket.get().is_none()>"Gửi"</button>
        </div>
        <div class="response-box" style="margin-top: 16px;">
            {logs}
        </div>
    }
}
