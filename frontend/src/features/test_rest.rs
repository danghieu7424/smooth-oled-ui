// src/features/test_rest.rs
use leptos::*;
use gloo_net::http::Request;

/****
 * Component: TestRest
 * Chức năng: Test gọi API REST với /demo/set (Lưu Cache/Storage) và /demo/get/:key
 ****/
#[component]
pub fn TestRest() -> impl IntoView {
    let (key, set_key) = create_signal(String::new());
    let (value, set_value) = create_signal(String::new());
    let (response, set_response) = create_signal("Chưa có kết quả...".to_string());

    let handle_set = move |_| {
        let k = key.get();
        let v = value.get();
        if k.is_empty() || v.is_empty() {
            set_response.set("Vui lòng nhập Key và Value".to_string());
            return;
        }

        spawn_local(async move {
            set_response.set("Đang lưu...".to_string());
            let payload = serde_json::json!({ "key": k, "value": v });
            
            match Request::post("http://localhost:5000/demo/set")
                .json(&payload)
                .unwrap()
                .send()
                .await {
                Ok(resp) => {
                    let text = resp.text().await.unwrap_or_else(|_| "Lỗi parse text".into());
                    set_response.set(format!("Status: {}\n{}", resp.status(), text));
                }
                Err(e) => set_response.set(format!("Lỗi mạng: {:?}", e)),
            }
        });
    };

    let handle_get = move |_| {
        let k = key.get();
        if k.is_empty() {
            set_response.set("Vui lòng nhập Key".to_string());
            return;
        }

        spawn_local(async move {
            set_response.set("Đang tải...".to_string());
            match Request::get(&format!("http://localhost:5000/demo/get/{}", k)).send().await {
                Ok(resp) => {
                    let text = resp.text().await.unwrap_or_else(|_| "Lỗi parse text".into());
                    set_response.set(format!("Status: {}\n{}", resp.status(), text));
                }
                Err(e) => set_response.set(format!("Lỗi mạng: {:?}", e)),
            }
        });
    };

    view! {
        <h2>"REST API (Cache & Storage)"</h2>
        <div class="form-group">
            <label>"Key"</label>
            <input type="text" placeholder="Nhập key..."
                on:input=move |ev| set_key.set(event_target_value(&ev))
                prop:value=key
            />
        </div>
        <div class="form-group">
            <label>"Value"</label>
            <input type="text" placeholder="Nhập value..."
                on:input=move |ev| set_value.set(event_target_value(&ev))
                prop:value=value
            />
        </div>
        <div class="form-group" style="flex-direction: row; gap: 8px; margin-top: 8px;">
            <button on:click=handle_set>"Set (Lưu)"</button>
            <button on:click=handle_get>"Get (Đọc)"</button>
        </div>
        <div class="response-box">
            {response}
        </div>
    }
}
