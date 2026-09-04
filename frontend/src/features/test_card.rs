// src/features/test_card.rs
use leptos::*;

/****
 * Component: TestCard
 * Chức năng: Lớp bao bọc Feature để thiết lập Stacking Context và chuyển động xuất hiện (GSAP-like).
 ****/
#[component]
pub fn TestCard(
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    // Kích hoạt class "is-entered" sau khi DOM render để chạy Animation
    let (is_entered, set_entered) = create_signal(false);
    
    create_effect(move |_| {
        set_timeout(move || set_entered.set(true), std::time::Duration::from_millis(50));
    });

    view! {
        <div class=format!("f-test-card {}", class) class:is-entered=is_entered>
            <div class="f-test-card__visual-container">
                {children()}
            </div>
        </div>
    }
}
