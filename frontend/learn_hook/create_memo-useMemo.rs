#[component]
fn MemoDemo() -> impl IntoView {
    let (count, set_count) = create_signal(1);

    // React: const double = useMemo(() => count * 2, [count]);
    let double_count = create_memo(move |_| {
        logging::log!("Đang tính toán..."); // Chỉ chạy khi count đổi
        count.get() * 2
    });

    view! {
        <p>"Gốc: " {count}</p>
        <p>"Gấp đôi (Memo): " {double_count}</p> // Dùng y hệt signal
        <button on:click=move |_| set_count.update(|n| *n += 1)>"Tăng"</button>
    }
}