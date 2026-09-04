// src/shared/ui/shuffle_btn.rs
use leptos::*;
use std::time::Duration;

#[component]
pub fn ShuffleMorphBtn(
    #[prop(into)] is_on: Signal<bool>,
    #[prop(into)] on_toggle: Callback<bool>,
    #[prop(optional, default = false.into(), into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    let (is_animating, set_is_animating) = create_signal(false);

    let handle_click = move |_e: ev::MouseEvent| {
        if disabled.get() {
            return;
        }

        let new_state = !is_on.get();

        if new_state {
            set_is_animating.set(true);
            set_timeout(
                move || set_is_animating.set(false),
                Duration::from_millis(500),
            );
        }

        on_toggle.call(new_state);
    };

    view! {
        <button
            class="mode-btn yt-shuffle-atom"
            class=("is-active", move || is_on.get())
            class=("is-animating", move || is_animating.get())
            disabled=move || disabled.get()
            on:click=handle_click
        >
            <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                // 🪄 Giảm độ dày nét vẽ xuống 1.5 để thanh thoát hơn
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                // 1. MŨI TÊN TRÊN (Đâm xuống)
                <g class="sf-top-group">
                    // 🪄 Đường cong liên tục mềm mại từ x=4 tới x=16, nối nhẹ với đoạn thẳng ở x=20
                    <path class="sf-line" d="M 4 7 C 9 7, 11 17, 16 17 L 20 17"></path>
                    // 🪄 Đầu mũi tên cũng mỏng manh và gọn gàng hơn
                    <path class="sf-head" d="M 17 14 L 20 17 L 17 20"></path>
                </g>

                // 2. MŨI TÊN DƯỚI (Đâm lên)
                <g class="sf-bot-group">
                    // 🪄 Vuốt mềm đường lượn
                    <path class="sf-line" d="M 4 17 C 9 17, 11 7, 16 7 L 20 7"></path>
                    // 🪄 Đầu mũi tên nhỏ gọn
                    <path class="sf-head" d="M 17 4 L 20 7 L 17 10"></path>
                </g>
            </svg>
        </button>
    }
}
