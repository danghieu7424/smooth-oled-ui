use leptos::*;

#[component]
pub fn NextMorphBtn(
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
    #[prop(optional, default = false.into(), into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    let (is_animating, set_is_animating) = create_signal(false);

    let handle_click = move |e: ev::MouseEvent| {
        // 🆕 Chỉ chạy logic nếu KHÔNG bị disabled
        if !disabled.get() && !is_animating.get() {
            set_is_animating.set(true);
            set_timeout(
                move || set_is_animating.set(false),
                std::time::Duration::from_millis(300),
            );
            on_click.call(e);
        }
    };

    view! {
        <button
            class="yt-next-btn"
            class=("is-animating", move || is_animating.get())
            disabled=move || disabled.get()
            on:click=handle_click
        >
            <svg viewBox="0 0 36 36" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path class="icon-triangle-incoming"></path>
                <path class="icon-triangle-main"></path>
                <path class="icon-line"></path>
            </svg>
        </button>
    }
}
