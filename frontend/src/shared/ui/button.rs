// src/shared/ui/button.rs
use leptos::*;

// 1. Định nghĩa các trạng thái (Variants) của Nút
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonVariant {
    Primary,
    #[allow(dead_code)]
    Secondary,
    #[allow(dead_code)]
    Danger,
    Ghost,
}

impl ButtonVariant {
    pub fn as_class(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "atm-btn--primary",
            ButtonVariant::Secondary => "atm-btn--secondary",
            ButtonVariant::Danger => "atm-btn--danger",
            ButtonVariant::Ghost => "atm-btn--ghost",
        }
    }
}

// 2. Component Nút Nhất quán
#[component]
pub fn Button(
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
    #[prop(optional, default = ButtonVariant::Primary)] variant: ButtonVariant,

    // 🆕 SỬA: Chuyển thành MaybeSignal để nút có tính Phản ứng (Reactivity)
    #[prop(optional, default = false.into(), into)] disabled: MaybeSignal<bool>,

    #[prop(into, optional)] extra_class: String,
    children: Children,
) -> impl IntoView {
    let class_name = move || format!("atm-btn {} {}", variant.as_class(), extra_class);

    view! {
        <button
            class=class_name
            // 🆕 SỬA: Cập nhật thuộc tính HTML liên tục mỗi khi Signal thay đổi
            disabled=move || disabled.get()
            on:click=move |e| {
                if !disabled.get() {
                    on_click.call(e);
                }
            }
        >
            {children()}
        </button>
    }
}
