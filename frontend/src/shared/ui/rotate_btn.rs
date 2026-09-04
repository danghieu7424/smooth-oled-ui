use leptos::*;

#[component]
pub fn RotateBtn(
    #[prop(into)] is_rotated: Signal<bool>,
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <button 
            class="yt-btn rotate-btn" 
            class:is-rotated=move || is_rotated.get()
            on:click=move |e| on_click.call(e)
            data-title="Xoay màn hình"
        >
            <svg class="icon-rotate" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M 4.5 6 A 7.5 7.5 0 0 0 4.5 18" />
                <path d="M 2 17.7 L 4.5 18 L 4.5 15.7" />
                
                <path d="M 19.5 18 A 7.5 7.5 0 0 0 19.5 6" />
                <path d="M 22 6.3 L 19.5 6 L 19.5 8.3" />

                <rect x="8" y="4" width="8" height="16" rx="1.5" ry="1.5" />
                <circle cx="12" cy="6" r="0.8" fill="currentColor" stroke="none" />
            </svg>
        </button>
    }
}
