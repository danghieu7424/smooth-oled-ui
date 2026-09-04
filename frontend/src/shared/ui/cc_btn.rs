use leptos::*;

#[component]
pub fn CcBtn(
    #[prop(into)] mode: Signal<u8>,
    #[prop(into)] on_toggle: Callback<()>,
    #[prop(into)] on_long_press: Callback<()>,
) -> impl IntoView {
    let (is_long_press, set_is_long_press) = create_signal(false);
    let (timer, set_timer) = create_signal(None::<gloo_timers::callback::Timeout>);
    
    let handle_pointer_down = move |_| {
        set_is_long_press.set(false);
        let t = gloo_timers::callback::Timeout::new(1000, move || {
            set_is_long_press.set(true);
            on_long_press.call(());
        });
        set_timer.set(Some(t));
    };

    let handle_pointer_up = move |_| {
        set_timer.update_untracked(|t| {
            if let Some(timeout) = t.take() {
                timeout.cancel();
            }
        });
    };

    let handle_click = move |e: ev::MouseEvent| {
        if is_long_press.get_untracked() {
            e.prevent_default();
            e.stop_propagation();
        } else {
            on_toggle.call(());
        }
    };

    view! {
        <button class="yt-btn cc-btn" 
                on:click=handle_click
                on:pointerdown=handle_pointer_down
                on:pointerup=handle_pointer_up
                on:pointerleave=handle_pointer_up
                on:pointercancel=handle_pointer_up
                data-title="Phụ đề">
            <svg class="icon-cc" class:on=move || mode.get() != 0 viewBox="0 0 24 24" fill="currentColor">
                <defs>
                    <mask id="cc-mask">
                        <rect width="24" height="24" fill="white" />
                        <path d="M11 10.5 A2 2 0 0 0 7 10.5 V13.5 A2 2 0 0 0 11 13.5 M17 10.5 A2 2 0 0 0 13 10.5 V13.5 A2 2 0 0 0 17 13.5" fill="none" stroke="black" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
                    </mask>
                </defs>
                <path class="bg" mask="url(#cc-mask)" d="M19 4H5C3.89 4 3.01 4.9 3.01 6L3 18C3 19.1 3.89 20 5 20H19C20.1 20 21 19.1 21 18V6C21 4.9 20.1 4 19 4z" />
                <path class="text" d="M11 10.5 A2 2 0 0 0 7 10.5 V13.5 A2 2 0 0 0 11 13.5 M17 10.5 A2 2 0 0 0 13 10.5 V13.5 A2 2 0 0 0 17 13.5" />
            </svg>
        </button>
    }
}
