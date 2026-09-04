// src/shared/ui/radio.rs
use leptos::*;

#[component]
pub fn Radio(
    /// 🆕 BẮT BUỘC: Tên nhóm để Trình duyệt biết các nút này thuộc về nhau
    #[prop(into)]
    name: String,

    #[prop(into)] value: String,
    #[prop(into)] selected_value: Signal<String>,
    #[prop(into)] on_change: Callback<String>,
    #[prop(into)] label: String,
    #[prop(optional, default = false.into(), into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    let val_for_check = value.clone();
    let is_checked = move || selected_value.get() == val_for_check;

    let val_for_change = value.clone();
    let name_clone = name.clone();

    view! {
        <label class="atm-radio-wrapper">
            <input
                type="radio"
                class="atm-radio-input"
                // 🆕 SỬA: Gắn tên nhóm vào input
                name=name_clone
                // 🆕 SỬA: Dùng `prop:checked` thay vì `checked`
                // Cú pháp này ép Trình duyệt BẮT BUỘC phải đồng bộ với biến is_checked của Rust
                prop:checked=is_checked
                disabled=move || disabled.get()
                on:change=move |_| {
                    if !disabled.get() {
                        on_change.call(val_for_change.clone());
                    }
                }
            />
            <span class="atm-radio-circle"></span>
            <span class="atm-radio-label">{label}</span>
        </label>
    }
}
