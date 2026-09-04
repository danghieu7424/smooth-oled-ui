// src/shared/ui/play_pause_btn.rs
use leptos::*;

#[component]
pub fn PlayPauseMorphBtn(
    #[prop(into)] is_playing: Signal<bool>,
    #[prop(optional, default = false.into(), into)] is_loading: MaybeSignal<bool>,
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
    #[prop(optional, default = false.into(), into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    view! {
        <button
            class="yt-morph-btn"
            class=("is-loading", move || is_loading.get())
            class=("paused", move || is_playing.get())
            disabled=move || disabled.get()
            on:click=move |e| {
                if !disabled.get() && !is_loading.get() {
                    on_click.call(e);
                }
            }
        >
            <svg
                class="icon-play-pause"
                viewBox="0 0 36 36"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path class="morph-path" fill="currentColor"></path>
            </svg>

            // 🚀 KIẾN TRÚC 12 VÒNG TRÒN ĐỘC LẬP THEO THUẬT TOÁN RING BUFFER
            <svg class="icon-spinner" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
                <circle class="ring ring-1" cx="50" cy="50" r="40" pathLength="360"></circle>
                <circle class="ring ring-2" cx="50" cy="50" r="40" pathLength="360"></circle>
                <circle class="ring ring-3" cx="50" cy="50" r="40" pathLength="360"></circle>
                <circle class="ring ring-4" cx="50" cy="50" r="40" pathLength="360"></circle>
                <circle class="ring ring-5" cx="50" cy="50" r="40" pathLength="360"></circle>
                <circle class="ring ring-6" cx="50" cy="50" r="40" pathLength="360"></circle>
                <circle class="ring ring-7" cx="50" cy="50" r="40" pathLength="360"></circle>
                <circle class="ring ring-8" cx="50" cy="50" r="40" pathLength="360"></circle>
                <circle class="ring ring-9" cx="50" cy="50" r="40" pathLength="360"></circle>
                <circle class="ring ring-10" cx="50" cy="50" r="40" pathLength="360"></circle>
                <circle class="ring ring-11" cx="50" cy="50" r="40" pathLength="360"></circle>
                <circle class="ring ring-12" cx="50" cy="50" r="40" pathLength="360"></circle>
            </svg>
        </button>
    }
}
