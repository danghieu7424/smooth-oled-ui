use leptos::*;

#[component]
pub fn FullscreenBtn(
    #[prop(into)] is_fullscreen: Signal<bool>,
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <button 
            class="yt-fullscreen-btn" 
            class=("is-fullscreen", move || is_fullscreen.get())
            on:click=move |e| on_click.call(e)
        >
            <div class="icon-wrapper">
                <svg class="icon-fullscreen" viewBox="0 0 24 24" fill="currentColor">
                    <path class="fs-path"></path>
                </svg>
            </div>
        </button>
    }
}
