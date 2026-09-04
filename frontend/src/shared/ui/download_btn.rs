// src/shared/ui/download_btn.rs
use leptos::*;
use std::time::Duration;

#[component]
pub fn DownloadMorphBtn(
    #[prop(into, optional)] on_click: Option<Callback<ev::MouseEvent>>,
    #[prop(optional, default = false.into(), into)] disabled: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    let (is_animating, set_is_animating) = create_signal(false);

    let handle_click = move |e: ev::MouseEvent| {
        if !disabled.get() && !is_animating.get() {
            set_is_animating.set(true);
            // 🪄 Tăng lên 800ms để nhịp điệu animation từ tốn hơn
            set_timeout(
                move || set_is_animating.set(false),
                Duration::from_millis(800),
            );
            if let Some(cb) = on_click {
                cb.call(e);
            }
        }
    };

    view! {
        <button
            class="yt-pill-btn standalone yt-download-atom"
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
                    <path class="dl-box" d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>

                    // 🪄 Mũi tên chính: Góp mặt 2 phần để co về dấu chấm
                    <g class="dl-arrow-main">
                        <line class="arrow-line" x1="12" y1="3" x2="12" y2="15"></line>
                        <polyline class="arrow-head" points="7 10 12 15 17 10"></polyline>
                    </g>

                    // 🪄 Mũi tên dự bị: Sẽ xuất hiện thấp hơn và kéo dãn
                    <g class="dl-arrow-incoming">
                        <line x1="12" y1="3" x2="12" y2="15"></line>
                        <polyline points="7 10 12 15 17 10"></polyline>
                    </g>
                </svg>
            </span>
            <span>{children()}</span>
        </button>
    }
}
