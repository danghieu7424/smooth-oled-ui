// src/store.rs
use leptos::{RwSignal, create_rw_signal};

// Tương lai có thể thêm UserProfile ở đây
#[derive(Clone, Debug)]
pub struct GlobalState {
    pub domain: String,
    // Trạng thái chung (Ví dụ: Theme Sáng/Tối)
    pub is_dark_mode: RwSignal<bool>,
}

pub fn init_global_state() -> GlobalState {
    GlobalState {
        domain: "http://localhost:5000".to_string(),
        is_dark_mode: create_rw_signal(true),
    }
}
