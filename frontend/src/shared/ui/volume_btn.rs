use leptos::*;

#[component]
pub fn VolumeBtn(
    #[prop(into)] volume: Signal<f64>,
    #[prop(into)] is_muted: Signal<bool>,
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <button 
            class="yt-volume-btn" 
            class=("state-mute", move || is_muted.get())
            class=("state-0", move || !is_muted.get() && volume.get() == 0.0)
            class=("state-1", move || !is_muted.get() && volume.get() > 0.0 && volume.get() < 0.5)
            class=("state-2", move || !is_muted.get() && volume.get() >= 0.5)
            on:click=move |e| on_click.call(e)
        >
            <div class="icon-wrapper">
                <svg class="icon-volume" viewBox="0 0 24 24" fill="currentColor">
                    // Base Speaker
                    <path class="vol-base"></path>
                    // Small Wave
                    <path class="vol-wave-1"></path>
                    // Large Wave
                    <path class="vol-wave-2"></path>
                    // Cross Slash (Mute)
                    <path class="vol-cross"></path>
                </svg>
            </div>
        </button>
    }
}
