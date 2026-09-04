// src/shared/ui/file_item.rs
use leptos::*;
use web_sys::File;

#[component]
pub fn FileItem(
    #[prop(into)] file: File,
    #[prop(into)] index: usize,
    #[prop(into)] on_delete: Callback<usize>,

    // 🆕 ĐÃ SỬA: Ép kiểu sang Signal để nhận "Đường ống" phản ứng từ Component cha
    #[prop(into, optional)] is_uploading: Signal<bool>,
    #[prop(into, optional)] progress: Signal<u8>,
) -> impl IntoView {
    let file_name = file.name();
    let extension = file_name
        .rsplit('.')
        .next()
        .unwrap_or("FILE")
        .to_uppercase();

    let badge_text = extension.chars().take(4).collect::<String>();

    let size_bytes = file.size();
    let size_display = if size_bytes < 1024.0 {
        format!("{} B", size_bytes)
    } else if size_bytes < 1048576.0 {
        format!("{:.1} KB", size_bytes / 1024.0)
    } else {
        format!("{:.2} MB", size_bytes / 1048576.0)
    };

    let handle_delete = move |_| {
        on_delete.call(index);
    };

    view! {
        // 🆕 Bọc logic tính class vào closure `move ||` và dùng .get()
        <div class=move || {
            format!("mol-file-item {}", if is_uploading.get() { "is-uploading" } else { "" })
        }>
            <div class="file-badge">{badge_text}</div>

            <div class="file-info">
                <div class="file-name" title=file_name.clone()>
                    {file_name.clone()}
                </div>
                <div class="file-meta">
                    // 🆕 Bọc text hiển thị dung lượng vào closure
                    {move || {
                        if is_uploading.get() {
                            let current_mb = (size_bytes * (progress.get() as f64 / 100.0))
                                / 1048576.0;
                            format!("{:.1} MB of {}", current_mb, size_display)
                        } else {
                            size_display.clone()
                        }
                    }}
                </div>
            </div>

            <button class="file-action-btn" on:click=handle_delete>
                // 🆕 Bọc icon vào closure để đổi Icon động
                {move || {
                    if is_uploading.get() {
                        view! {
                            <svg
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                            >
                                <line x1="18" y1="6" x2="6" y2="18"></line>
                                <line x1="6" y1="6" x2="18" y2="18"></line>
                            </svg>
                        }
                            .into_any()
                    } else {
                        view! {
                            <svg
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            >
                                <polyline points="3 6 5 6 21 6"></polyline>
                                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                            </svg>
                        }
                            .into_any()
                    }
                }}
            </button>

            // 🆕 Bọc Progress Bar vào closure
            {move || {
                if is_uploading.get() {
                    view! {
                        <div class="file-progress-track">
                            <div
                                class="file-progress-fill"
                                style=format!("width: {}%", progress.get())
                            ></div>
                        </div>
                    }
                        .into_any()
                } else {
                    view! { <span style="display: none"></span> }.into_any()
                }
            }}
        </div>
    }
}
