// src/shared/ui/atoms/protected_text.rs
use leptos::*;

/****
 * Component: ProtectedText
 * Chức năng: Atom Text có Dual-tone shadow để chống chói/tương phản trên nền gradient.
 ****/
#[component]
pub fn ProtectedText(
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <span class=format!("a-protected-text {}", class)>
            {children()}
        </span>
    }
}
