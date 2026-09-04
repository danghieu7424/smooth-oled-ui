// src/pages/home_page.rs
use leptos::*;

use crate::features::test_card::TestCard;
use crate::features::test_rest::TestRest;
use crate::features::test_jspb::TestJspb;
use crate::features::test_sse::TestSse;
use crate::features::test_ws::TestWs;
use crate::features::test_webrtc::TestWebrtc;
use crate::shared::ui::atoms::theme_toggle::ThemeToggle;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="home-page-base">
            <header class="dashboard-header">
                <div class="title-area">
                    <h1 class="a-protected-text">"Fluent Glass Dashboard"</h1>
                    <p class="a-protected-text">"Kiểm thử kết nối Backend & Giao diện Glassmorphism"</p>
                </div>
                <div class="actions">
                    <ThemeToggle />
                </div>
            </header>

            <div class="dashboard-grid">
                <TestCard><TestRest /></TestCard>
                <TestCard><TestJspb /></TestCard>
                <TestCard><TestSse /></TestCard>
                <TestCard><TestWs /></TestCard>
                <TestCard><TestWebrtc /></TestCard>
            </div>
        </div>
    }
}
