// src/shared/ui/checkbox.rs
use leptos::*;

#[component]
pub fn Checkbox(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_change: Callback<bool>,
    #[prop(into)] label: String,
    #[prop(optional, default = false.into(), into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    view! {
        <label class="atm-checkbox-wrapper">
            <input
                type="checkbox"
                class="atm-checkbox-input"
                checked=move || checked.get()
                disabled=move || disabled.get()
                on:change=move |_| {
                    if !disabled.get() {
                        on_change.call(!checked.get());
                    }
                }
            />
            <span class="atm-checkbox-box">
                // Dấu Tick SVG nguyên bản
                <svg viewBox="0 0 24 24">
                    <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
            </span>
            <span class="atm-checkbox-label">{label}</span>
        </label>
    }
}
