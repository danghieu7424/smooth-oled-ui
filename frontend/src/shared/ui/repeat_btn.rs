// src/shared/ui/repeat_btn.rs
use leptos::*;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

#[component]
pub fn RepeatMorphBtn(
    #[prop(into)] mode: Signal<RepeatMode>,
    #[prop(into)] on_change: Callback<RepeatMode>,
) -> impl IntoView {
    let (is_animating, set_is_animating) = create_signal(false);

    let handle_click = move |_| {
        let current = mode.get();
        let next_mode = match current {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        };

        // 🪄 Chỉ kích hoạt hoạt cảnh Vẽ đuổi nhau khi chuyển từ trạng thái Tắt -> Bật All
        if current == RepeatMode::Off && next_mode == RepeatMode::All {
            set_is_animating.set(true);
            set_timeout(
                move || set_is_animating.set(false),
                Duration::from_millis(600),
            );
        }

        on_change.call(next_mode);
    };

    let btn_class = move || {
        let mode_cls = match mode.get() {
            RepeatMode::Off => "is-off",
            RepeatMode::All => "is-all",
            RepeatMode::One => "is-one",
        };
        let anim_cls = if is_animating.get() {
            "is-animating"
        } else {
            ""
        };
        format!("mode-btn yt-repeat-atom {} {}", mode_cls, anim_cls)
    };

    view! {
        <button class=btn_class on:click=handle_click>
            <svg
                width="24" height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                // 1. Nửa Vòng trên (Sẽ được vẽ trước)
                <g class="rp-top-group">
                    <path class="rp-line" d="M4 11 V9 a4 4 0 0 1 4-4 h12"></path>
                    // 🪄 Đầu mũi tên thu nhỏ gọn lại (x=20, y=5)
                    <path class="rp-head" d="M17 2 L20 5 L17 8"></path>
                </g>

                // 2. Nửa Vòng dưới (Sẽ được vẽ sau)
                <g class="rp-bot-group">
                    <path class="rp-line" d="M20 13 V15 a4 4 0 0 1 -4 4 H4"></path>
                    // 🪄 Đầu mũi tên thu nhỏ gọn lại (x=4, y=19)
                    <path class="rp-head" d="M7 22 L4 19 L7 16"></path>
                </g>

                // 3. Số 1 tàng hình
                <path class="repeat-digit-one" d="M11 10 h1 v5"></path>
            </svg>
        </button>
    }
}
