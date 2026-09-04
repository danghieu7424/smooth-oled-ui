// src/shared/ui/toggle_group.rs
use leptos::*;

#[component]
pub fn ToggleGroup(
    /// Danh sách các nhãn (VD: vec!["Tất cả", "Âm nhạc", "Trực tiếp"])
    #[prop(into)]
    options: Vec<String>,

    /// Trạng thái lưu giá trị đang được chọn
    #[prop(into)]
    selected: Signal<String>,

    /// Bắn sự kiện ra ngoài khi user click chọn 1 Chip
    #[prop(into)]
    on_select: Callback<String>,
) -> impl IntoView {
    view! {
        // Đóng gói toàn bộ trong 1 container để quản lý Scroll
        <div class="atm-toggle-group">
            {options
                .into_iter()
                .map(|opt| {
                    let opt_for_class = opt.clone();
                    let opt_for_click = opt.clone();
                    let is_active = move || selected.get() == opt_for_class;
                    // Clone data để đưa vào các closure của Leptos

                    // Reactive check: Nút này có trùng với giá trị đang select không?

                    view! {
                        <button
                            class="atm-chip"
                            // Leptos sẽ tự động thêm/bớt class 'active' dựa vào Signal
                            class=("active", is_active)
                            on:click=move |e| {
                                e.stop_propagation();
                                on_select.call(opt_for_click.clone());
                            }
                        >
                            {opt}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}
