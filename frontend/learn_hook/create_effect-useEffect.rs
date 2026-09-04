// 1. Chạy 1 lần duy nhất (Mount)
#[component]
fn Demo() -> impl IntoView {
    // --- React: useEffect(..., []) ---
    // Leptos: Viết thẳng ở đây!
    logging::log!("Dòng này chỉ chạy đúng 1 lần khi component sinh ra");

    // Nếu cần thao tác DOM sau khi render xong:
    create_effect(move |_| {
        logging::log!("Chạy 1 lần sau khi DOM đã mount (nếu không phụ thuộc signal nào)");
    });

    view! { ... }
}

// 2. Chạy khi biến thay đổi (Dependencies)
#[component]
fn Demo() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    let (name, set_name) = create_signal("A");

    // --- React: useEffect(..., [count]) ---
    // Leptos: Tự động lắng nghe `count` vì bạn gọi `count.get()`
    create_effect(move |_| {
        logging::log!("Count đã đổi thành: {}", count.get());
        // Hàm này KHÔNG chạy khi `name` đổi, vì ta không gọi name.get() ở đây.
        // Leptos rất thông minh, nó soi code bạn để biết cần nghe cái gì.
    });

    view! { ... }
}

// 3. Cleanup (Return một giá trị)
#[component]
fn Timer() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    create_effect(move |_| {
        // Ví dụ: Mỗi lần count đổi, ta set một cái interval mới
        let handle = set_interval_handle(...);

        // --- React: return () => clearInterval(handle) ---
        // Leptos: Gọi hàm on_cleanup
        on_cleanup(move || {
            logging::log!("Dọn dẹp timer cũ trước khi tạo timer mới (hoặc khi unmount)");
            clear_interval(handle);
        });
        
        // Logic effect chạy ở đây...
        count.get(); 
    });

    view! { ... }
}