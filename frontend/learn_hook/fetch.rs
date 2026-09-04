use leptos::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::{Request, RequestCredentials}; // Thư viện fetch chuẩn cho Wasm

// --- 1. MODEL DỮ LIỆU ---
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: i32,
    name: String,
    email: String, // Thêm email cho đầy đủ
}

// --- 2. HÀM FETCH API (Có gửi Cookie) ---
async fn fetch_user_api(user_id: i32) -> Result<User, String> {
    // Dùng gloo_net thay vì reqwest để config credentials dễ dàng
    let resp = Request::get(&format!("https://jsonplaceholder.typicode.com/users/{}", user_id))
        // QUAN TRỌNG: Gửi Cookie đi kèm (kể cả cross-domain nếu server cho phép)
        .credentials(RequestCredentials::Include) 
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?; // Chuyển lỗi mạng sang String

    // Kiểm tra status code (200 OK)
    if resp.ok() {
        resp.json::<User>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Lỗi API: {} - Không tìm thấy User", resp.status()))
    }
}

// --- 3. COMPONENT ---
#[component]
pub fn UserProfile() -> impl IntoView {
    // Signal lưu ID đang nhập
    let (user_id, set_user_id) = create_signal(1);

    // RESOURCE: Tự động chạy lại hàm fetch khi user_id thay đổi
    let user_resource = create_resource(
        user_id,                // Dependency
        |id| fetch_user_api(id) // Hàm fetch
    );

    view! {
        <div class="user-profile-container" style="padding: 20px; border: 1px solid #ccc;">
            <h3>"Demo Fetch API + Cookie"</h3>
            
            <div style="margin-bottom: 15px;">
                <label>"Nhập ID User: "</label>
                <input 
                    type="number" 
                    min="1" max="10"
                    // Ràng buộc giá trị input với signal
                    prop:value=user_id
                    on:input=move |ev| {
                        // Parse giá trị nhập vào và update signal
                        if let Ok(id) = event_target_value(&ev).parse::<i32>() {
                            set_user_id.set(id);
                        }
                    }
                />
            </div>

            <hr/>

            // SUSPENSE: Xử lý trạng thái Loading
            <Suspense fallback=move || view! { <p class="loading">"⏳ Đang tải dữ liệu..."</p> }>
                {move || {
                    // Xử lý 3 trạng thái của Resource
                    match user_resource.get() {
                        None => view! { <p>"Đang khởi tạo..."</p> }.into_view(),
                        
                        Some(Ok(user)) => view! {
                            <div class="user-card" style="background: #f0f0f0; padding: 10px; border-radius: 5px;">
                                <p><strong>"ID:"</strong> {user.id}</p>
                                <p><strong>"Tên:"</strong> {user.name}</p>
                                <p><strong>"Email:"</strong> {user.email}</p>
                                <small style="color: green;">"✅ Đã tải xong (Kèm Cookie)"</small>
                            </div>
                        }.into_view(),
                        
                        Some(Err(e)) => view! { 
                            <p class="error" style="color: red;">
                                <strong>"Lỗi: "</strong> {e}
                            </p> 
                        }.into_view()
                    }
                }}
            </Suspense>
        </div>
    }
}

fn main() {
    mount_to_body(|| view! { <UserProfile/> })
}