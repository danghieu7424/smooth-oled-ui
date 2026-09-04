// src/shared/ui/progress_bar.rs
use leptos::*;

use wasm_bindgen::JsCast;

fn format_time(seconds: f64) -> String {
    if seconds.is_nan() || seconds.is_infinite() {
        return "00:00".to_string();
    }
    let s = seconds as u64;
    let m = s / 60;
    let h = m / 60;
    let m = m % 60;
    let s = s % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

#[component]
pub fn ProgressBar(
    #[prop(into)] progress: Signal<f64>,
    #[prop(into)] buffer: Signal<f64>,
    #[prop(optional, into)] duration: Option<Signal<f64>>,
    #[prop(into)] on_seek: Callback<f64>,
    #[prop(optional, into)] on_seek_start: Option<Callback<()>>,
    #[prop(optional, into)] on_seek_end: Option<Callback<()>>,
) -> impl IntoView {
    let (hover_percent, set_hover_percent) = create_signal(None::<f64>);

    let handle_mousemove = move |e: leptos::ev::MouseEvent| {
        if let Some(target) = e.target() {
            if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                let rect = el.get_bounding_client_rect();
                let x = e.client_x() as f64 - rect.left();
                let width = rect.width();
                let mut p = (x / width) * 100.0;
                if p < 0.0 { p = 0.0; }
                if p > 100.0 { p = 100.0; }
                set_hover_percent.set(Some(p));
            }
        }
    };

    let handle_mouseleave = move |_| {
        set_hover_percent.set(None);
    };

    let handle_input = move |e: leptos::ev::Event| {
        if let Ok(val) = event_target_value(&e).parse::<f64>() {
            set_hover_percent.set(Some(val));
            on_seek.call(val);
        }
    };

    view! {
        <div class="yt-progress-atom" on:mousemove=handle_mousemove on:mouseleave=handle_mouseleave>
            <Show when=move || hover_percent.get().is_some() && duration.is_some()>
                <div 
                    class="progress-tooltip" 
                    style=move || format!("left: {}%", hover_percent.get().unwrap_or(0.0))
                >
                    {move || {
                        if let (Some(p), Some(dur_sig)) = (hover_percent.get(), duration) {
                            let d = dur_sig.get();
                            format_time(d * (p / 100.0))
                        } else {
                            "".to_string()
                        }
                    }}
                </div>
            </Show>
            <div class="progress-track-wrapper">
                // 1. Lớp Buffer (Đoạn video đã tải trước)
                <div
                    class="progress-buffer"
                    style=move || format!("width: {}%", buffer.get())
                ></div>

                // 2. Lớp Progress (Đoạn video đã xem)
                <div
                    class="progress-fill"
                    style=move || format!("width: {}%", progress.get())
                ></div>
            </div>

            // 3. Lớp Input Tàng hình (Chịu trách nhiệm nhận sự kiện vuốt/kéo)
            <input
                type="range"
                min="0"
                max="100"
                step="0.1"
                class="progress-ghost-slider"
                prop:value=move || progress.get().to_string()
                on:mousedown=move |ev| { ev.stop_propagation(); if let Some(cb) = on_seek_start { cb.call(()); } }
                on:touchstart=move |ev| { ev.stop_propagation(); if let Some(cb) = on_seek_start { cb.call(()); } }
                on:touchmove=move |ev| ev.stop_propagation()
                on:touchend=move |ev| { ev.stop_propagation(); set_hover_percent.set(None); }
                on:input=handle_input
                on:change=move |_| if let Some(cb) = on_seek_end { cb.call(()); }
            />
        </div>
    }
}
