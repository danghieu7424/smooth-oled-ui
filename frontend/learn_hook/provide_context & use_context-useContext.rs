// Bước 1: Định nghĩa Struct chứa dữ liệu
use leptos::*;

// Clone và Copy để truyền đi dễ dàng mà không tốn bộ nhớ
#[derive(Clone, Copy, Debug)]
struct ThemeContext {
    is_dark: RwSignal<bool>, // RwSignal = Read-Write Signal
}

// Bước 2: Cung cấp ở Component cha (provide_context)
#[component]
fn App() -> impl IntoView {
    // Tạo signal
    let is_dark = create_rw_signal(false);
    
    // Đóng gói vào struct và Cung cấp cho toàn bộ cây con
    provide_context(ThemeContext { is_dark });

    view! {
        <Layout />
    }
}

// Bước 3: Sử dụng ở Component con (use_context)
#[component]
fn Layout() -> impl IntoView {
    // Lấy context dựa trên kiểu ThemeContext
    // use_context trả về Option, nên cần unwrap hoặc expect
    let theme = use_context::<ThemeContext>().expect("Chưa cung cấp ThemeContext");

    view! {
        // Đọc giá trị
        <div class:dark=move || theme.is_dark.get()>
            "Giao diện hiện tại: " {move || if theme.is_dark.get() { "Tối" } else { "Sáng" }}
            
            // Sửa giá trị
            <button on:click=move |_| theme.is_dark.update(|d| *d = !*d)>
                "Đổi Theme"
            </button>
        </div>
    }
}