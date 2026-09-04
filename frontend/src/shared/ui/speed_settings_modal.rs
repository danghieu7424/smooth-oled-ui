use leptos::*;

#[component]
pub fn SpeedSettingsModal(
    #[prop(into)] speed: Signal<f64>,
    #[prop(into)] set_speed: Callback<f64>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let preset_speeds = vec![
        0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5
    ];

    let handle_slider_input = move |ev: ev::Event| {
        let val = event_target_value(&ev).parse::<f64>().unwrap_or(1.0);
        set_speed.call(val);
    };

    let step_down = move |_| {
        let current = speed.get();
        if current > 0.25 {
            set_speed.call((current - 0.25).max(0.25));
        }
    };

    let step_up = move |_| {
        let current = speed.get();
        if current < 4.0 {
            set_speed.call((current + 0.25).min(4.0));
        }
    };

    view! {
        <div class="speed-modal-overlay" on:click=move |_| on_close.call(())>
            <div class="speed-modal-content" on:click=move |ev| ev.stop_propagation()>
                <div class="modal-handle"></div>
                <div class="modal-header">
                    <span class="speed-display">{move || format!("{:.2}x", speed.get())}</span>
                    <button class="close-btn" on:click=move |_| on_close.call(())>
                        <svg viewBox="0 0 24 24" fill="currentColor" width="24" height="24">
                            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"></path>
                        </svg>
                    </button>
                </div>
                
                <div class="slider-container">
                    <button class="step-btn" on:click=step_down>
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M19 13H5v-2h14v2z"></path>
                        </svg>
                    </button>
                    <input type="range" class="speed-slider" 
                           min="0.25" max="4.0" step="0.05"
                           value=move || speed.get()
                           on:input=handle_slider_input 
                    />
                    <button class="step-btn" on:click=step_up>
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"></path>
                        </svg>
                    </button>
                </div>

                <div class="preset-grid">
                    {preset_speeds.into_iter().map(|s| {
                        let is_active = move || (speed.get() - s).abs() < 0.01;
                        let s_clone = s;
                        view! {
                            <button class="preset-btn" class:active=is_active on:click=move |_| set_speed.call(s_clone)>
                                <span class="preset-val">{format!("{:.2}", s)}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
