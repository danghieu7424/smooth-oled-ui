use leptos::*;

#[component]
pub fn SkipBtn(
    #[prop(into)] enabled: Signal<bool>,
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <button 
            class="yt-btn skip-toggle-btn" 
            class:is-active=move || enabled.get()
            on:click=move |e| on_click.call(e)
            data-title="Tự động tua (Bật/Tắt)"
        >
            <svg class="icon-skip" viewBox="0 0 24 24" fill="currentColor">
                <path d="M4 18l8.5-6L4 6v12zm9-12v12l8.5-6L13 6z"></path>
            </svg>
        </button>
    }
}
