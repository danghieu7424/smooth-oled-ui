// [TÊN FILE]: src/shared/molecules/restricted_modal.rs
/****
 * RESTRICTED MODAL MOLECULE — Hiển thị thông báo khi bị lỗi 403 Forbidden
 *
 * Input: is_open (Signal<bool>), onClose (Callback)
 * Output: Glassmorphism Modal Dialog
 ****/
use leptos::*;
use leptos_router::*;

#[component]
pub fn RestrictedModal(is_open: Signal<bool>, on_close: Callback<()>) -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <Show when=move || is_open.get()>
            <div class="restricted-modal-overlay">
                <div class="restricted-modal-box glass-panel">
                    <div class="modal-icon-lock">
                        <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
                            <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
                        </svg>
                    </div>
                    <h2 class="modal-title">"Bạn không có quyền nghe!"</h2>
                    <p class="modal-desc">
                        "Bài hát này đã được giới hạn quyền nghe. Chỉ những người có tên trong danh sách cấp quyền mới có thể phát nhạc."
                    </p>
                    <div class="modal-actions">
                        <button class="btn-primary glass-btn" on:click={
                            let nav = navigate.clone();
                            move |_| {
                                on_close.call(());
                                nav("/", Default::default());
                            }
                        }>
                            "Về trang chủ"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
