use leptos::*;

#[component]
pub fn VideoModal(
    url: String,
    on_close: Callback<()>,
) -> impl IntoView {
    // Basic video modal
    view! {
        <div class="media-modal-overlay" on:click=move |_| on_close.call(())>
            <div class="media-modal-content" on:click=move |e| e.stop_propagation()>
                <video controls autoplay class="video-player">
                    <source src=url type="video/mp4" />
                    "Trình duyệt của bạn không hỗ trợ video thẻ."
                </video>
                <button class="close-btn" on:click=move |_| on_close.call(())>"X"</button>
            </div>
        </div>
    }
}
