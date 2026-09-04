// src/features/test_jspb.rs
use leptos::*;
use gloo_net::http::Request;

/****
 * Component: TestJspb
 * Chức năng: Gọi API JSPB, bóc tách chuỗi bảo mật )]}',\n và parse JSON.
 ****/
#[component]
pub fn TestJspb() -> impl IntoView {
    let (response, set_response) = create_signal("Chưa có kết quả...".to_string());

    let fetch_jspb = move |_| {
        spawn_local(async move {
            set_response.set("Đang gọi /demo/jspb...".to_string());
            match Request::get("http://localhost:5000/demo/jspb").send().await {
                Ok(resp) => {
                    let text = resp.text().await.unwrap_or_default();
                    // Loại bỏ chuỗi chống Hijacking
                    let prefix = ")]}',\n";
                    if text.starts_with(prefix) {
                        let clean_json = &text[prefix.len()..];
                        set_response.set(format!("Raw Text:\n{}\n\nParsed JSON:\n{}", text, clean_json));
                    } else {
                        set_response.set(format!("Không nhận diện được chuẩn JSPB:\n{}", text));
                    }
                }
                Err(e) => set_response.set(format!("Lỗi mạng: {:?}", e)),
            }
        });
    };

    view! {
        <h2>"JSPB Anti-Hijacking"</h2>
        <div class="form-group">
            <button on:click=fetch_jspb>"Lấy dữ liệu JSPB"</button>
        </div>
        <div class="response-box" style="margin-top: 16px;">
            {response}
        </div>
    }
}
