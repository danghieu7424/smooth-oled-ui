// src/shared/ui/atoms/glass_surface.rs
use leptos::*;

/****
 * Component: GlassSurface
 * Chức năng: Tạo bề mặt kính trong suốt theo kiến trúc Atomic của Glassmorphism.
 * Không chứa logic chuyển động (transform) để tránh lỗi Stacking Context.
 ****/
#[component]
pub fn GlassSurface(
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("a-glass-surface {}", class)>
            {children()}
        </div>
    }
}
