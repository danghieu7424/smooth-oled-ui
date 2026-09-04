use leptos::*;

#[component]
fn Counter() -> impl IntoView {
    // React: const [count, setCount] = useState(0);
    // Leptos: Trả về 2 hàm: (getter, setter)
    let (count, set_count) = create_signal(0);

    view! {
        <div>
            // 1. ĐỌC GIÁ TRỊ:
            // Phải gọi nó như một hàm: count() hoặc count.get()
            // Bắt buộc dùng move || để bọc lại, giúp Leptos lắng nghe thay đổi
            <p>"Số hiện tại: " {move || count.get()}</p>

            // 2. CẬP NHẬT GIÁ TRỊ:
            // Cách A: Thay thế hoàn toàn (giống setCount(newVal))
            <button on:click=move |_| set_count.set(10)>"Set 10"</button>

            // Cách B: Update dựa trên giá trị cũ (giống setCount(c => c + 1))
            <button on:click=move |_| set_count.update(|n| *n += 1)>"Tăng"</button>
        </div>
    }
}