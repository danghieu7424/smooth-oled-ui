// src/shared/ui/action_btns.rs
use crate::shared::ui::like_btn::LikeFireworksBtn;
use leptos::*;

// 1. CỤM NÚT LIKE / DISLIKE
#[component]
pub fn LikeDislikeGroup() -> impl IntoView {
    let (is_liked, set_is_liked) = create_signal(false);
    let (is_disliked, set_is_disliked) = create_signal(false);
    let (like_count, set_like_count) = create_signal(12400);

    let handle_like = move |_| {
        if is_liked.get() {
            set_is_liked.set(false);
            set_like_count.update(|c| *c -= 1);
        } else {
            set_is_liked.set(true);
            set_is_disliked.set(false);
            set_like_count.update(|c| *c += 1);
        }
    };

    let handle_dislike = move |_| {
        if is_disliked.get() {
            set_is_disliked.set(false);
        } else {
            set_is_disliked.set(true);
            set_is_liked.set(false);
            if is_liked.get() {
                set_like_count.update(|c| *c -= 1);
            }
        }
    };

    let format_count = move || {
        let c = like_count.get();
        if c >= 10000 {
            format!("{:.1}N", c as f64 / 1000.0)
        } else {
            c.to_string()
        }
    };

    view! {
        <div class="yt-pill-group">
            // 🆕 DÙNG ATOM ĐÃ ĐƯỢC ĐÓNG GÓI
            <LikeFireworksBtn is_liked=is_liked on_click=handle_like>
                {format_count}
            </LikeFireworksBtn>

            <div class="yt-divider"></div>

            // Nút Dislike giữ nguyên dạng standalone
            <button
                class="yt-pill-btn right-side"
                class=("is-active", move || is_disliked.get())
                on:click=handle_dislike
            >
                <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path d="M10 15v4a3 3 0 0 0 3 3l4-9V2H5.72a2 2 0 0 0-2 1.7l-1.38 9a2 2 0 0 0 2 2.3zm7-13h3a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-3"></path>
                </svg>
            </button>
        </div>
    }
}

// 2. NÚT CHỨC NĂNG ĐỘC LẬP (Share, Download, Save...)
#[component]
pub fn ActionPillBtn(
    #[prop(into)] text: String,
    children: Children,
    #[prop(into, optional)] on_click: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    view! {
        <button
            class="yt-pill-btn standalone"
            on:click=move |e| {
                if let Some(cb) = on_click {
                    cb.call(e);
                }
            }
        >
            <span class="icon-slot">{children()}</span>
            <span>{text}</span>
        </button>
    }
}
