use leptos::*;

#[component]
pub fn ImageModal(
    url: String,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="media-modal-overlay" on:click=move |_| on_close.call(())>
            <div class="media-modal-content" on:click=move |e| e.stop_propagation()>
                <img src=url class="image-viewer" alt="Full size" />
                <button class="close-btn" on:click=move |_| on_close.call(())>"X"</button>
            </div>
        </div>
    }
}
