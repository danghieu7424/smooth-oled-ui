// src/shared/ui/share_btn.rs
use leptos::*;
use std::time::Duration;

#[component]
pub fn ShareMorphBtn(
    #[prop(into, optional)] on_click: Option<Callback<ev::MouseEvent>>,
    #[prop(optional, default = false.into(), into)] disabled: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    let (is_animating, set_is_animating) = create_signal(false);

    let handle_click = move |e: ev::MouseEvent| {
        if !disabled.get() && !is_animating.get() {
            set_is_animating.set(true);
            // 🪄 SỬA LỖI: Tăng thời gian lên 600ms (250ms bay + 100ms nghỉ + 250ms mọc)
            set_timeout(
                move || set_is_animating.set(false),
                Duration::from_millis(1000),
            );
            if let Some(cb) = on_click {
                cb.call(e);
            }
        }
    };

    view! {
        <button
            class="yt-pill-btn standalone yt-share-atom"
            class=("is-animating", move || is_animating.get())
            disabled=move || disabled.get()
            on:click=handle_click
        >
            <span class="icon-slot">
                <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path
                        class="share-box"
                        d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h2"
                    ></path>

                    <g class="share-arrow-main">
                        <polyline points="15 3 21 3 21 9" transform="rotate(25 21 3)"></polyline>
                        <path d="M10 14 C 10 8, 15 3, 21 3"></path>
                    </g>

                    <g class="share-arrow-incoming">
                        <polyline points="15 3 21 3 21 9" transform="rotate(25 21 3)"></polyline>
                        <path d="M10 14 C 10 8, 15 3, 21 3"></path>
                    </g>
                </svg>
            </span>
            <span>{children()}</span>
        </button>
    }
}
