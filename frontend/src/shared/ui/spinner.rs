// src/shared/ui/spinner.rs
use leptos::*;

#[component]
pub fn RingSpinner(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg
            class=format!("flaren-ring-spinner {}", class)
            viewBox="0 0 100 100"
            xmlns="http://www.w3.org/2000/svg"
        >
            <circle class="ring ring-1" cx="50" cy="50" r="40" pathLength="360"></circle>
            <circle class="ring ring-2" cx="50" cy="50" r="40" pathLength="360"></circle>
            <circle class="ring ring-3" cx="50" cy="50" r="40" pathLength="360"></circle>
            <circle class="ring ring-4" cx="50" cy="50" r="40" pathLength="360"></circle>
            <circle class="ring ring-5" cx="50" cy="50" r="40" pathLength="360"></circle>
            <circle class="ring ring-6" cx="50" cy="50" r="40" pathLength="360"></circle>
            <circle class="ring ring-7" cx="50" cy="50" r="40" pathLength="360"></circle>
            <circle class="ring ring-8" cx="50" cy="50" r="40" pathLength="360"></circle>
            <circle class="ring ring-9" cx="50" cy="50" r="40" pathLength="360"></circle>
            <circle class="ring ring-10" cx="50" cy="50" r="40" pathLength="360"></circle>
            <circle class="ring ring-11" cx="50" cy="50" r="40" pathLength="360"></circle>
            <circle class="ring ring-12" cx="50" cy="50" r="40" pathLength="360"></circle>
        </svg>
    }
}
