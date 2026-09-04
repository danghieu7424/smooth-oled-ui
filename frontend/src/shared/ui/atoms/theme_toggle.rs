// [TÊN FILE]: src/shared/ui/atoms/theme_toggle.rs
use leptos::*;

/****
 * [Atom] ThemeToggle
 * Chức năng: Component nút gạt (toggle) dùng để chuyển đổi giữa Light và Dark mode.
 * - Đọc/Ghi trạng thái vào `localStorage` key="theme".
 * - Đồng bộ class "light" hoặc "dark" lên thẻ `<html>`.
 ****/
#[component]
pub fn ThemeToggle() -> impl IntoView {
    // Đọc trạng thái ban đầu từ localStorage (nếu có), mặc định là "dark"
    let initial_theme = {
        let mut theme = "dark".to_string();
        if let Some(window) = web_sys::window() {
            if let Ok(Some(ls)) = window.local_storage() {
                if let Ok(Some(stored_theme)) = ls.get_item("theme") {
                    if stored_theme == "light" || stored_theme == "dark" {
                        theme = stored_theme;
                    }
                }
            }
        }
        theme
    };

    let (theme, set_theme) = create_signal(initial_theme.clone());

    // Effect để áp dụng class vào <html> và lưu localStorage mỗi khi `theme` thay đổi
    create_effect(move |_| {
        let current_theme = theme.get();
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(html) = document.document_element() {
                    let _ = html.class_list().remove_2("light", "dark");
                    let _ = html.class_list().add_1(&current_theme);
                }
                
                // Update highlight.js theme
                if let Some(hljs_link) = document.get_element_by_id("hljs-theme") {
                    let href = if current_theme == "dark" {
                        "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/atom-one-dark.min.css"
                    } else {
                        "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/atom-one-light.min.css"
                    };
                    let _ = hljs_link.set_attribute("href", href);
                }
            }
            if let Ok(Some(ls)) = window.local_storage() {
                let _ = ls.set_item("theme", &current_theme);
            }
        }
    });

    // Toggle function
    let toggle_theme = move |_| {
        set_theme.update(|t| {
            if *t == "dark" {
                *t = "light".to_string();
            } else {
                *t = "dark".to_string();
            }
        });
    };

    view! {
        <button class="theme-toggle-atom dropdown-item" on:click=toggle_theme>
            <div class="toggle-track" class=("is-light", move || theme.get() == "light")>
                <div class="toggle-thumb">
                    <Show
                        when=move || theme.get() == "dark"
                        fallback=|| view! {
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="12" cy="12" r="5"></circle>
                                <line x1="12" y1="1" x2="12" y2="3"></line>
                                <line x1="12" y1="21" x2="12" y2="23"></line>
                                <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                                <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                                <line x1="1" y1="12" x2="3" y2="12"></line>
                                <line x1="21" y1="12" x2="23" y2="12"></line>
                                <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                                <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                            </svg>
                        }
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                        </svg>
                    </Show>
                </div>
            </div>
            <span class="theme-label">
                {move || if theme.get() == "dark" { "Giao diện: Tối" } else { "Giao diện: Sáng" }}
            </span>
        </button>
    }
}
