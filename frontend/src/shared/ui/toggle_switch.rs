// src/shared/ui/toggle_switch.rs
use leptos::*;

#[component]
pub fn ToggleSwitch(
    /// Trạng thái Bật/Tắt hiện tại
    #[prop(into)]
    is_on: Signal<bool>,

    /// Bắn sự kiện kèm giá trị mới (true/false) khi người dùng gạt
    #[prop(into)]
    on_toggle: Callback<bool>,

    /// Trạng thái khóa (Tùy chọn)
    #[prop(optional, default = false.into(), into)]
    disabled: MaybeSignal<bool>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            role="switch"
            aria-checked=move || is_on.get().to_string()
            class="atm-toggle-switch"
            // Thêm class is-on nếu true
            class=("is-on", move || is_on.get())
            disabled=move || disabled.get()
            on:click=move |_| {
                if !disabled.get() {
                    on_toggle.call(!is_on.get());
                }
            }
        ></button>
    }
}
