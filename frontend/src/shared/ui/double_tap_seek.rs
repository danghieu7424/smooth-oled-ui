use leptos::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SeekDirection {
    Left,
    Right,
}

#[component]
pub fn DoubleTapSeekOverlay(
    #[prop(into)] show: Signal<bool>,
    #[prop(into)] direction: Signal<SeekDirection>,
    #[prop(into)] accumulated_seconds: Signal<i32>,
) -> impl IntoView {
    view! {
        <Show when=move || show.get() fallback=|| ()>
            <div class=move || {
                let dir = if direction.get() == SeekDirection::Left { "left" } else { "right" };
                format!("yt-double-tap-seek {}", dir)
            }>
                <For
                    each=move || {
                        let taps = accumulated_seconds.get().abs() / 10;
                        (0..taps).collect::<Vec<_>>()
                    }
                    key=|&i| i
                    children=|_| view! { <div class="ripple"></div> }
                />
                <div class="content">
                    <div class=move || {
                        let dir = if direction.get() == SeekDirection::Left { "left-arrows" } else { "right-arrows" };
                        format!("arrows {}", dir)
                    }>
                        <svg class="arrow-1" viewBox="0 0 36 36" fill="currentColor">
                            <path d="M 12 10 L 18 14 L 18 22 L 12 26 Z M 18 14 L 24 18 L 24 18 L 18 22 Z"></path>
                        </svg>
                        <svg class="arrow-2" viewBox="0 0 36 36" fill="currentColor">
                            <path d="M 12 10 L 18 14 L 18 22 L 12 26 Z M 18 14 L 24 18 L 24 18 L 18 22 Z"></path>
                        </svg>
                        <svg class="arrow-3" viewBox="0 0 36 36" fill="currentColor">
                            <path d="M 12 10 L 18 14 L 18 22 L 12 26 Z M 18 14 L 24 18 L 24 18 L 18 22 Z"></path>
                        </svg>
                    </div>
                    <span class="text">{move || format!("{} giây", accumulated_seconds.get().abs())}</span>
                </div>
            </div>
        </Show>
    }
}
