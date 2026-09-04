// src/shared/ui/save_btn.rs
use leptos::*;

#[component]
pub fn SaveMorphBtn(
    #[prop(into, optional)] on_click: Option<Callback<ev::MouseEvent>>,
    #[prop(optional, default = false.into(), into)] disabled: MaybeSignal<bool>,
    #[prop(into)] is_saved: Signal<bool>, // 🚀 ĐÓN TÍN HIỆU TỪ CHA
    children: Children,
) -> impl IntoView {
    let handle_click = move |e: ev::MouseEvent| {
        if disabled.get() {
            return;
        }
        if let Some(cb) = on_click {
            cb.call(e);
        }
    };

    // 🚀 BẬT TẮT CLASS is-success ĐỂ KÍCH HOẠT ANIMATION DẤU TICK CỦA SẾP
    let btn_class = move || {
        if is_saved.get() {
            "yt-pill-btn standalone yt-save-atom is-success"
        } else {
            "yt-pill-btn standalone yt-save-atom"
        }
    };

    view! {
        <button class=btn_class disabled=move || disabled.get() on:click=handle_click>
            <span class="icon-slot">
                <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path class="save-lines" d="M5 7h14 M5 12h14 M5 17h6"></path>
                    <g class="save-plus">
                        <path d="M17 15v6 M14 18h6"></path>
                    </g>
                    <path class="save-check" d="M13.5 17.5l2.5 2.5 5-6"></path>
                </svg>
            </span>
            <span>{children()}</span>
        </button>
    }
}
