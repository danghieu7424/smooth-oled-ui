// [TÊN FILE]: src/shared/ui/atoms/logo.rs
use leptos::*;

#[component]
pub fn LogoVibe() -> impl IntoView {
    view! {
        <div class="verivibe-logo-atom">
            // 🚀 SVG TỐI ƯU HÓA: Chỉ dùng path đơn giản để vẽ sóng Vibes & Âm thanh
            <svg
                viewBox="0 0 100 100"
                xmlns="http://www.w3.org/2000/svg"
                class="logo-svg-icon"
                fill="none"
                // Sử dụng màu text của parent (Teal)
                stroke="currentColor"
                stroke-width="10"
                stroke-linecap="round"
            >
                // Vẽ 3 cột sóng âm Vibes cao thấp xen kẽ
                <path d="M20 30 V70" class="wave-1" />
                <path d="M50 10 V90" class="wave-2" />
                <path d="M80 45 V55" class="wave-3" />
            </svg>
            <span class="logo-text-brand">"VeriVibe"</span>
        </div>
    }
}
