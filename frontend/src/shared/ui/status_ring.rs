// src/shared/ui/status_ring.rs
use leptos::*;

// 1. ĐỊNH NGHĨA MÁY TRẠNG THÁI CHUẨN RUST
#[derive(Clone, PartialEq, Debug)]
pub enum RingState {
    Idle,        // Trạng thái 1: Nét đứt chờ đợi
    Loading(u8), // Trạng thái 2: Đang chạy (kèm % tiến độ từ 0-100)
    Processing,  // 🆕 Tải VÔ ĐỊNH (Không biết tham số)
    Success,     // Trạng thái 3: Thành công (Dấu V xanh)
    #[allow(dead_code)]
    Warning, // Trạng thái 4: Cảnh báo (Dấu ! vàng)
    Error,       // Trạng thái 5: Lỗi (Dấu X đỏ)
    Skipped,
}

impl RingState {
    // Trả về class tương ứng để SCSS xử lý
    pub fn as_class(&self) -> &'static str {
        match self {
            RingState::Idle => "is-idle",
            RingState::Loading(_) => "is-loading",
            RingState::Processing => "is-processing", // 🆕 Class mới
            RingState::Success => "is-success",
            RingState::Warning => "is-warning",
            RingState::Error => "is-error",
            RingState::Skipped => "is-skipped",
        }
    }
}

#[component]
pub fn StatusRing(
    /// Nhận trạng thái hiện tại từ Component cha
    #[prop(into)]
    state: Signal<RingState>,
) -> impl IntoView {
    // Tính toán độ dài nét vẽ (offset) cho vòng Progress dựa trên %
    // Chu vi (Dash array) của r=10.5 là ~66
    let progress_offset = move || {
        if let RingState::Loading(percent) = state.get() {
            // Đảm bảo không vượt quá 100
            let p = percent.min(100) as f32;
            66.0 - (66.0 * p / 100.0)
        } else {
            66.0
        }
    };

    // Lấy con số để hiển thị text
    let progress_text = move || {
        if let RingState::Loading(percent) = state.get() {
            percent.min(100).to_string()
        } else {
            "".to_string()
        }
    };

    // 🆕 SỬA Ở ĐÂY: Gom chung class tĩnh và class động lại bằng format!
    let wrapper_class = move || format!("atm-status-ring {}", state.get().as_class());

    view! {
        <div class=wrapper_class>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
                <circle class="ring-base" cx="12" cy="12" r="10.5" />

                <circle
                    class="ring-progress"
                    cx="12"
                    cy="12"
                    r="10.5"
                    style:stroke-dashoffset=progress_offset
                />

                <circle class="ring-spinner" cx="12" cy="12" r="10.5" />

                // 4 vòng kết quả
                <circle class="ring-success" cx="12" cy="12" r="10.5" />
                <circle class="ring-error" cx="12" cy="12" r="10.5" />
                <circle class="ring-warning" cx="12" cy="12" r="10.5" />
                <circle class="ring-skipped" cx="12" cy="12" r="10.5" />
                /

                <text class="ring-text" x="12" y="12.5">
                    {progress_text}
                </text>

                <g class="icon-group">
                    <path class="icon-success" d="M7.5 12.5L10.5 15.5L16.5 9.5" />
                    <path class="icon-error" d="M8 8 L 16 16 M 16 8 L 8 16" />
                    <path class="icon-warning" d="M12 7 L 12 14 M 12 17 L 12 17.1" />
                    // 🆕 Icon dấu trừ
                    <path class="icon-skipped" d="M7 12 L 17 12" />
                </g>
            </svg>
        </div>
    }
}
