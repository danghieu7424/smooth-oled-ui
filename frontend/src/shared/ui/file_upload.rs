// src/shared/ui/file_upload.rs
use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, File, HtmlInputElement};

#[component]
pub fn FileUploadDropzone(
    #[prop(into)] on_files_select: Callback<Vec<File>>,
    #[prop(into)] on_clear: Callback<()>,
    #[prop(into, optional, default = "Kéo thả file vào đây".to_string())] title: String,
    #[prop(into, optional, default = "Hoặc click để chọn từ máy tính".to_string())]
    description: String,
    #[prop(into, optional, default = "*/*".to_string())] accept: String,
    #[prop(optional, default = false)] multiple: bool,
    #[prop(optional, default = false.into(), into)] disabled: MaybeSignal<bool>,
    #[prop(optional, default = true)] show_badge: bool,
) -> impl IntoView {
    let (is_dragging, set_is_dragging) = create_signal(false);
    let (file_count, set_file_count) = create_signal(0);
    let input_ref = create_node_ref::<html::Input>();

    let accept_val = accept.clone();
    let process_files = move |file_list: Option<web_sys::FileList>| {
        if disabled.get() {
            return;
        }

        if let Some(files) = file_list {
            let mut valid_files = Vec::new();
            let limit = if multiple { files.length() } else { 1 };

            let allowed_extensions: Vec<String> = accept_val
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .collect();

            for i in 0..limit {
                if let Some(file) = files.item(i) {
                    let file_name = file.name().to_lowercase();

                    let is_allowed = if accept_val == "*/*" {
                        true
                    } else {
                        allowed_extensions.iter().any(|ext| {
                            if ext.starts_with('.') {
                                // Nếu là đuôi file (.srt, .vtt)
                                file_name.ends_with(ext)
                            } else if ext.ends_with("/*") {
                                // 🆕 FIX LỖI: Nếu là dạng wildcard (video/*, image/*)
                                // Ta cắt bỏ dấu * đi (thành "video/") và kiểm tra phần đầu
                                let base_type = ext.trim_end_matches('*');
                                file.type_().to_lowercase().starts_with(base_type)
                            } else {
                                // Các chuẩn MIME cụ thể (vd: application/json)
                                file.type_().to_lowercase() == *ext
                            }
                        })
                    };

                    if is_allowed {
                        valid_files.push(file);
                    }
                }
            }

            if !valid_files.is_empty() {
                set_file_count.set(valid_files.len());
                on_files_select.call(valid_files);
            }
        }
    };

    let clear_selection = move |e: ev::MouseEvent| {
        e.stop_propagation();
        set_file_count.set(0);
        if let Some(input) = input_ref.get() {
            input.set_value("");
        }
        on_clear.call(());
    };

    // FIX LỖI E0382 TẠI ĐÂY: Tạo bản sao của closure cho sự kiện Drop
    let process_files_for_drop = process_files.clone();

    let on_input_change = move |e: Event| {
        let target = e
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
        if let Some(input) = target {
            process_files(input.files());
        }
    };

    view! {
        <div
            class=move || {
                format!(
                    "atm-dropzone {} {}",
                    if disabled.get() { "is-disabled" } else { "" },
                    if is_dragging.get() { "is-dragging" } else { "" },
                )
            }
            on:click=move |_| {
                if !disabled.get()
                    && let Some(input) = input_ref.get() {
                        input.click();
                    }
            }
            on:dragover=move |e| {
                e.prevent_default();
                if !disabled.get() {
                    set_is_dragging.set(true);
                }
            }
            on:dragleave=move |_| {
                set_is_dragging.set(false);
            }
            on:drop=move |e| {
                e.prevent_default();
                set_is_dragging.set(false);
                if !disabled.get()
                    && let Some(dt) = e.data_transfer() {
                        process_files_for_drop(dt.files());
                    }
            }
        >
            <input
                node_ref=input_ref
                type="file"
                class="dropzone-input-hidden"
                accept=accept
                multiple=multiple
                on:change=on_input_change
            />

            <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                <polyline points="17 8 12 3 7 8"></polyline>
                <line x1="12" y1="3" x2="12" y2="15"></line>
            </svg>

            <div class="dropzone-title">{title}</div>
            <div class="dropzone-desc">{description}</div>

            <Show when=move || { show_badge && file_count.get() > 0 }>
                <div class="dropzone-status">
                    <span>{format!("Đã nhận {} file", file_count.get())}</span>
                    <button class="dropzone-clear-btn" on:click=clear_selection>
                        "Hủy chọn"
                    </button>
                </div>
            </Show>
        </div>
    }
}
