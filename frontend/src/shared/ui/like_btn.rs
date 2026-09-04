// src/shared/ui/like_btn.rs
use leptos::*;
use std::time::Duration;

#[component]
pub fn LikeFireworksBtn(
    #[prop(into)] is_liked: Signal<bool>,
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
    children: Children,
) -> impl IntoView {
    let (is_animating, set_is_animating) = create_signal(false);

    let handle_click = move |e: ev::MouseEvent| {
        if !is_liked.get() {
            set_is_animating.set(true);
            set_timeout(
                move || set_is_animating.set(false),
                Duration::from_millis(1000),
            );
        }
        on_click.call(e);
    };

    view! {
        <button
            class="yt-pill-btn yt-like-atom left-side"
            class=("is-active", move || is_liked.get())
            class=("is-animating", move || is_animating.get())
            on:click=handle_click
        >
            <div class="icon-container">
                // 1. Icon Like (Gốc)
                <svg
                    class="main-like-icon"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                >
                    <path d="M14 9V5a3 3 0 0 0-3-3l-4 9v11h11.28a2 2 0 0 0 2-1.7l1.38-9a2 2 0 0 0-2-2.3zM7 22H4a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2h3"></path>
                </svg>

                // 2. Nốt nhạc CHÍNH (Màu Trắng, Nảy tại tâm)
                <svg class="big-note-center" viewBox="0 0 24 24" fill="#ffffff">
                    <path d="M12 3c-.55 0-1 .45-1 1v8.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4c.55 0 1-.45 1-1V4c0-.55-.45-1-1-1h-5z" />
                </svg>

                // 3. 4 Nốt nhạc CON (TÍM GRADIENT BÁM CỨNG VÀO TỪNG THẺ)
                {(1..=4)
                    .map(|i| {
                        let grad_id = format!("purple-grad-{}", i);
                        let fill_url = format!("url(#{})", grad_id);
                        // Cấp ID riêng biệt cho từng nốt để Router không bao giờ nhầm lẫn

                        view! {
                            <svg class=format!("small-note note-p{}", i) viewBox="0 0 24 24">
                                <defs>
                                    <linearGradient id=grad_id x1="0%" y1="0%" x2="100%" y2="100%">
                                        // Tím Neon
                                        <stop offset="0%" stop-color="#bc13fe" />
                                        // Hồng Đậm
                                        <stop offset="100%" stop-color="#ff0080" />
                                    </linearGradient>
                                </defs>
                                // Bơm thẳng URL Gradient vào path
                                <path
                                    fill=fill_url
                                    d="M12 3c-.55 0-1 .45-1 1v8.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4c.55 0 1-.45 1-1V4c0-.55-.45-1-1-1h-5z"
                                />
                            </svg>
                        }
                    })
                    .collect_view()}
            </div>

            <span class="like-count">{children()}</span>
        </button>
    }
}
