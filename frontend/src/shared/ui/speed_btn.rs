use leptos::*;

#[component]
pub fn SpeedBtn(
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <button class="yt-btn speed-btn" on:click=move |e| on_click.call(e) data-title="Tốc độ phát">
            <svg class="icon-speed" viewBox="0 0 24 24" fill="currentColor">
                <path class="speed-path"></path>
            </svg>
        </button>
    }
}
