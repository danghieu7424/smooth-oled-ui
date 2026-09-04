// src/shared/ui/minimize_btn.rs
use leptos::*;
use std::time::Duration;

#[component]
pub fn MinimizeMorphBtn(
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
    #[prop(optional, default = false.into(), into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    let (is_animating, set_is_animating) = create_signal(false);

    let handle_click = move |e: ev::MouseEvent| {
        if !disabled.get() && !is_animating.get() {
            set_is_animating.set(true);
            set_timeout(
                move || set_is_animating.set(false),
                Duration::from_millis(400),
            );
            on_click.call(e);
        }
    };

    view! {
        <button
            class="yt-minimize-btn"
            class=("is-animating", move || is_animating.get())
            disabled=move || disabled.get()
            on:click=handle_click
        >
            <svg viewBox="0 0 36 36" fill="none" xmlns="http://www.w3.org/2000/svg">
                // 🪄 2 Path để làm cú lừa thị giác nối tiếp nhau
                <path class="icon-chevron-incoming"></path>
                <path class="icon-chevron-main"></path>
            </svg>
        </button>
    }
}
