// src/features/test_webrtc.rs
use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent};

/****
 * Component: TestWebrtc
 * Chức năng: Giả lập quá trình Signaling (Trao đổi SDP/ICE) cho WebRTC qua WebSocket.
 ****/
#[component]
pub fn TestWebrtc() -> impl IntoView {
    let (logs, set_logs) = create_signal(String::new());
    let (socket, set_socket) = create_signal::<Option<WebSocket>>(None);

    let connect_signaling = move |_| {
        if socket.get().is_some() { return; }

        let window = web_sys::window().unwrap();
        let host = window.location().host().unwrap();
        let ws_url = if host.contains("localhost") || host.contains("127.0.0.1") {
            "ws://localhost:5000/api/v1/webrtc/dummy_user_123".to_string()
        } else {
            format!("wss://{}/api/v1/webrtc/dummy_user_123", host)
        };

        if let Ok(ws) = WebSocket::new(&ws_url) {
            set_logs.update(|l| l.push_str("Đang mở kết nối WebRTC Signaling...\n"));
            
            let onmessage = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Some(text) = e.data().as_string() {
                    set_logs.update(|l| l.push_str(&format!("<- {}\n", text)));
                }
            }) as Box<dyn FnMut(_)>);
            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();

            let onopen = wasm_bindgen::closure::Closure::wrap(Box::new(move |_| {
                set_logs.update(|l| l.push_str("Signaling Server đã sẵn sàng!\n"));
            }) as Box<dyn FnMut(web_sys::Event)>);
            ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            onopen.forget();

            set_socket.set(Some(ws));
        }
    };

    let send_offer = move |_| {
        if let Some(ws) = socket.get() {
            let msg = r#"{"type": "sdp", "data": "dummy_offer_sdp"}"#;
            if ws.send_with_str(msg).is_ok() {
                set_logs.update(|l| l.push_str("-> Đã gửi SDP Offer giả lập\n"));
            }
        }
    };

    let send_ice = move |_| {
        if let Some(ws) = socket.get() {
            let msg = r#"{"type": "ice", "data": "dummy_ice_candidate"}"#;
            if ws.send_with_str(msg).is_ok() {
                set_logs.update(|l| l.push_str("-> Đã gửi ICE Candidate giả lập\n"));
            }
        }
    };

    view! {
        <h2>"WebRTC Signaling"</h2>
        <div class="form-group">
            <button on:click=connect_signaling disabled=move || socket.get().is_some()>
                {move || if socket.get().is_some() { "Đã kết nối Server" } else { "Kết nối Signaling" }}
            </button>
        </div>
        <div class="form-group" style="flex-direction: row; gap: 8px; margin-top: 8px;">
            <button on:click=send_offer disabled=move || socket.get().is_none()>"Gửi Offer"</button>
            <button on:click=send_ice disabled=move || socket.get().is_none()>"Gửi ICE"</button>
        </div>
        <div class="response-box" style="margin-top: 16px;">
            {logs}
        </div>
    }
}
