// TẠI FILE src/shared/ui/follow_btn.rs
use crate::shared::ui::spinner::RingSpinner;
use leptos::*;

#[component]
pub fn FollowIconBtn(
    #[prop(into)] is_following: Signal<bool>,
    #[prop(into)] is_pending: Signal<bool>,
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <abbr
            title=move || if is_following.get() { "Đang theo dõi" } else { "Theo dõi" }
            style="text-decoration: none; cursor: pointer;"
        >
            <button
                class="icon-action-btn btn-follow-icon"
                class=("is-following", is_following)
                prop:disabled=move || is_pending.get()
                on:click=move |e| on_click.call(e)
            >
                <Show
                    when=move || is_pending.get()
                    fallback=move || {
                        if is_following.get() {
                            // 🚀 ICON: NGƯỜI CÓ DẤU CHECK ("v")
                            view! {
                                <svg
                                    viewBox="0 0 24 24"
                                    width="20"
                                    height="20"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                >
                                    <path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
                                    <circle cx="8.5" cy="7" r="4"></circle>
                                    <polyline points="17 11 19 13 23 9"></polyline>
                                </svg>
                            }
                                .into_view()
                        } else {
                            // 🚀 ICON: NGƯỜI CÓ DẤU CỘNG ("+")
                            view! {
                                <svg
                                    viewBox="0 0 24 24"
                                    width="20"
                                    height="20"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                >
                                    <path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
                                    <circle cx="8.5" cy="7" r="4"></circle>
                                    <line x1="20" y1="8" x2="20" y2="14"></line>
                                    <line x1="23" y1="11" x2="17" y2="11"></line>
                                </svg>
                            }
                                .into_view()
                        }
                    }
                >
                    <div style="width: 18px; height: 18px;">
                        <RingSpinner />
                    </div>
                </Show>
            </button>
        </abbr>
    }
}
