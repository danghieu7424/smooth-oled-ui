use leptos::*;
use crate::shared::ui::repeat_btn::{RepeatMorphBtn, RepeatMode};

#[component]
pub fn PlaybackSettingsMenu(
    #[prop(into)] playback_mode: Signal<String>,
    #[prop(into)] on_playback_change: Callback<String>,
) -> impl IntoView {
    let (show_playback_settings, set_show_playback_settings) = create_signal(false);
    let (settings_menu_state, set_settings_menu_state) = create_signal("main".to_string());

    view! {
        <div class="playback-settings-container atom-playback-settings">
            <button class="settings-btn" on:click=move |_| set_show_playback_settings.update(|s| *s = !*s)>
                <svg class="icon-settings" viewBox="0 0 24 24" fill="currentColor">
                    <path class="settings-path"></path>
                </svg>
            </button>
            <Show when=move || show_playback_settings.get() fallback=|| ()>
                <div class="playback-settings-dropdown">
                    <Show when=move || settings_menu_state.get() == "main">
                        <div class="menu-item main-item" on:click=move |_| set_settings_menu_state.set("playback".to_string())>
                            <div class="menu-item-left">
                                <div class="repeat-icon-wrapper" on:click=move |ev| ev.stop_propagation()>
                                    <RepeatMorphBtn 
                                        mode=Signal::derive(move || match playback_mode.get().as_str() {
                                            "once" => RepeatMode::Off,
                                            "next" => RepeatMode::All,
                                            "loop" => RepeatMode::One,
                                            _ => RepeatMode::Off,
                                        })
                                        on_change=move |new_mode| {
                                            let mode_str = match new_mode {
                                                RepeatMode::Off => "once",
                                                RepeatMode::All => "next",
                                                RepeatMode::One => "loop",
                                            };
                                            on_playback_change.call(mode_str.to_string());
                                        }
                                    />
                                </div>
                                <span class="menu-label">"Chế độ phát"</span>
                            </div>
                            <div class="menu-item-right">
                                <span>{move || match playback_mode.get().as_str() {
                                    "once" => "Một lần",
                                    "loop" => "Lặp lại",
                                    "next" => "Tiếp theo",
                                    _ => ""
                                }}</span>
                                <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                                    <polyline points="9 18 15 12 9 6"></polyline>
                                </svg>
                            </div>
                        </div>
                    </Show>

                    <Show when=move || settings_menu_state.get() == "playback">
                        <div class="menu-header" on:click=move |_| set_settings_menu_state.set("main".to_string())>
                            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
                                <polyline points="15 18 9 12 15 6"></polyline>
                            </svg>
                            <span>"Chế độ phát"</span>
                        </div>
                        
                        <div class="menu-options">
                            <label class="radio-label">
                                <input type="radio" name="playback_mv" value="once" checked=move || playback_mode.get() == "once" on:input=move |_| { on_playback_change.call("once".to_string()); set_show_playback_settings.set(false); set_settings_menu_state.set("main".to_string()); } />
                                "Chỉ một lần"
                            </label>
                            <label class="radio-label">
                                <input type="radio" name="playback_mv" value="loop" checked=move || playback_mode.get() == "loop" on:input=move |_| { on_playback_change.call("loop".to_string()); set_show_playback_settings.set(false); set_settings_menu_state.set("main".to_string()); } />
                                "Lặp lại"
                            </label>
                            <label class="radio-label">
                                <input type="radio" name="playback_mv" value="next" checked=move || playback_mode.get() == "next" on:input=move |_| { on_playback_change.call("next".to_string()); set_show_playback_settings.set(false); set_settings_menu_state.set("main".to_string()); } />
                                "Tiếp theo"
                            </label>
                        </div>
                    </Show>
                </div>
            </Show>
        </div>
    }
}
