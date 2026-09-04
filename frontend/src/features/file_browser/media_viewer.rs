use leptos::*;
use wasm_bindgen::JsCast;
use crate::features::file_browser::api::FileItem;
use crate::shared::ui::playback_settings::PlaybackSettingsMenu;
use crate::features::file_browser::youtube_player::YoutubePlayer;
#[component]
pub fn MediaViewer(
    files: Memo<Vec<FileItem>>,
    active_index: RwSignal<Option<usize>>,
    #[prop(into)] playback_mode: ReadSignal<String>,
    #[prop(into)] on_playback_change: Callback<String>,
    #[prop(into)] video_speed: ReadSignal<f64>,
    #[prop(into)] on_video_speed_change: Callback<f64>,
    #[prop(into)] subtitle_mode: ReadSignal<u8>,
    #[prop(into)] on_subtitle_mode_change: Callback<u8>,
    #[prop(into)] show_remaining_time: ReadSignal<bool>,
    #[prop(into)] on_show_remaining_time_change: Callback<bool>,
    #[prop(into)] auto_skip_enabled: ReadSignal<bool>,
    #[prop(into)] on_auto_skip_enabled_change: Callback<bool>,
) -> impl IntoView {
    let current_file = move || {
        if let Some(idx) = active_index.get() {
            files.with(|f| f.get(idx).cloned())
        } else {
            None
        }
    };
    
    let get_category = |name: &str| -> String {
        let lower = name.to_lowercase();
        if lower.ends_with(".mp4") || lower.ends_with(".mkv") || lower.ends_with(".webm") || lower.ends_with(".avi") || lower.ends_with(".mov") || lower.ends_with(".flv") || lower.ends_with(".wmv") || lower.ends_with(".mpd") {
            "video".to_string()
        } else if lower.ends_with(".mp3") || lower.ends_with(".wav") || lower.ends_with(".flac") || lower.ends_with(".aac") {
            "audio".to_string()
        } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png") || lower.ends_with(".webp") || lower.ends_with(".gif") || lower.ends_with(".bmp") || lower.ends_with(".ico") || lower.ends_with(".svg") {
            "image".to_string()
        } else {
            "other".to_string()
        }
    };
    
    let has_prev = move || {
        if let Some(idx) = active_index.get() {
            let cat = files.with(|f| f.get(idx).map(|file| get_category(&file.name)).unwrap_or_else(|| "other".to_string()));
            files.with(|f| {
                for i in (0..idx).rev() {
                    if get_category(&f[i].name) == cat { return true; }
                }
                false
            })
        } else { false }
    };
    
    let has_next = move || {
        if let Some(idx) = active_index.get() {
            let cat = files.with(|f| f.get(idx).map(|file| get_category(&file.name)).unwrap_or_else(|| "other".to_string()));
            files.with(|f| {
                for i in (idx + 1)..f.len() {
                    if get_category(&f[i].name) == cat { return true; }
                }
                false
            })
        } else { false }
    };

    let go_prev = move || {
        if let Some(idx) = active_index.get() {
            let cat = files.with(|f| f.get(idx).map(|file| get_category(&file.name)).unwrap_or_else(|| "other".to_string()));
            let prev_idx = files.with(|f| {
                for i in (0..idx).rev() {
                    if get_category(&f[i].name) == cat { return Some(i); }
                }
                None
            });
            if let Some(i) = prev_idx { active_index.set(Some(i)); }
        }
    };

    let go_next = move || {
        if let Some(idx) = active_index.get() {
            let cat = files.with(|f| f.get(idx).map(|file| get_category(&file.name)).unwrap_or_else(|| "other".to_string()));
            let next_idx = files.with(|f| {
                for i in (idx + 1)..f.len() {
                    if get_category(&f[i].name) == cat { return Some(i); }
                }
                None
            });
            if let Some(i) = next_idx { active_index.set(Some(i)); }
        }
    };
    
    let handle_ended = move |ev: leptos::ev::Event| {
        let mode = playback_mode.get();
        if mode == "next" {
            go_next();
        } else if mode == "loop" {
            if let Some(target) = ev.target() {
                if let Ok(media_elem) = target.dyn_into::<web_sys::HtmlMediaElement>() {
                    media_elem.set_current_time(0.0);
                    let _ = media_elem.play();
                }
            }
        }
    };
    
    let close = move |_| {
        active_index.set(None);
    };



    let touch_start_x = create_rw_signal::<Option<f64>>(None);
    let touch_start_y = create_rw_signal::<Option<f64>>(None);

    let on_touchstart = move |ev: leptos::ev::TouchEvent| {
        if let Some(touch) = ev.touches().get(0) {
            touch_start_x.set(Some(touch.client_x() as f64));
            touch_start_y.set(Some(touch.client_y() as f64));
        }
    };
    
    let on_touchend = move |ev: leptos::ev::TouchEvent| {
        if let Some(changed_touch) = ev.changed_touches().get(0) {
            if let (Some(start_x), Some(start_y)) = (touch_start_x.get(), touch_start_y.get()) {
                let end_x = changed_touch.client_x() as f64;
                let end_y = changed_touch.client_y() as f64;
                let diff_x = start_x - end_x;
                let diff_y = start_y - end_y;
                
                if diff_x.abs() > 50.0 && diff_x.abs() > diff_y.abs() {
                    if diff_x > 0.0 {
                        go_next();
                    } else {
                        go_prev();
                    }
                }
            }
        }
        touch_start_x.set(None);
        touch_start_y.set(None);
    };

    let media_src = move || {
        if let Some(file) = current_file() {
            let path = file.path.clone();
            let encoded_path = path.split('/').map(|part| js_sys::encode_uri_component(part).as_string().unwrap()).collect::<Vec<_>>().join("/");
            format!("/storages/{}", encoded_path)
        } else {
            String::new()
        }
    };

    let is_video = move || {
        if let Some(file) = current_file() {
            let lower_path = file.path.to_lowercase();
            lower_path.ends_with(".mp4") || lower_path.ends_with(".mkv") || lower_path.ends_with(".webm") || lower_path.ends_with(".avi") || lower_path.ends_with(".mov") || lower_path.ends_with(".flv") || lower_path.ends_with(".wmv") || lower_path.ends_with(".mpd")
        } else { false }
    };
    
    let is_audio = move || {
        if let Some(file) = current_file() {
            let lower_path = file.path.to_lowercase();
            lower_path.ends_with(".mp3") || lower_path.ends_with(".wav") || lower_path.ends_with(".flac") || lower_path.ends_with(".aac")
        } else { false }
    };
    
    let is_image = move || {
        if current_file().is_some() {
            !is_video() && !is_audio()
        } else { false }
    };

    view! {
        <div class="media-viewer-overlay" 
             style={move || if active_index.get().is_some() { "display: flex;" } else { "display: none;" }}>
            
            <div class="media-viewer-backdrop" on:click=close></div>
            
            <div class="media-viewer-content" on:touchstart=on_touchstart on:touchend=on_touchend>
                <div class="media-viewer-controls" style="display: flex; gap: 10px; align-items: flex-start;">

                    <button class="close-btn" on:click=close style="position: static;">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <line x1="18" y1="6" x2="6" y2="18"></line>
                            <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                    </button>
                </div>
                
                <>
                            <Show when=is_video fallback=|| ()>
                                <YoutubePlayer 
                                    src=Signal::derive(media_src)
                                    on_ended=handle_ended.clone()
                                    on_next=Callback::new(move |_| go_next())
                                    on_prev=Callback::new(move |_| go_prev())
                                    has_next=Signal::derive(has_next)
                                    has_prev=Signal::derive(move || has_prev())
                                    playback_mode=playback_mode
                                    on_playback_change=on_playback_change
                                    video_speed=video_speed
                                    on_video_speed_change=on_video_speed_change
                                    subtitle_mode=subtitle_mode
                                    on_subtitle_mode_change=on_subtitle_mode_change
                                    show_remaining_time=show_remaining_time
                                    on_show_remaining_time_change=on_show_remaining_time_change
                                    auto_skip_enabled=auto_skip_enabled
                                    on_auto_skip_enabled_change=on_auto_skip_enabled_change
                                />
                            </Show>

                            <Show when=is_audio fallback=|| ()>
                                <div class="audio-container" style="position: relative; display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; width: 100%; max-width: 600px; margin: 0 auto; background: rgba(0,0,0,0.2); border-radius: 16px; padding: 40px 20px;">
                                    <PlaybackSettingsMenu playback_mode=playback_mode on_playback_change=on_playback_change />
                                    <svg viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1.5" style="margin-bottom: 24px; color: #fff;">
                                        <path d="M9 18V5l12-2v13"></path>
                                        <circle cx="6" cy="18" r="3"></circle>
                                        <circle cx="18" cy="16" r="3"></circle>
                                    </svg>
                                    <audio src=media_src controls autoplay 
                                           on:ended=handle_ended.clone()
                                           style="width: 100%;">
                                    </audio>
                                </div>
                            </Show>

                            <Show when=is_image fallback=|| ()>
                                <div class="image-container">
                                    <img src=media_src alt="media" />
                                </div>
                            </Show>
                        </>
                
                <button class="nav-btn prev-btn" 
                        on:click=move |_| go_prev() 
                        style={move || if has_prev() && !is_video() { "visibility: visible;" } else { "visibility: hidden;" }}>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="15 18 9 12 15 6"></polyline>
                    </svg>
                </button>
                
                <button class="nav-btn next-btn" 
                        on:click=move |_| go_next() 
                        style={move || if has_next() && !is_video() { "visibility: visible;" } else { "visibility: hidden;" }}>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="9 18 15 12 9 6"></polyline>
                    </svg>
                </button>
                
                <div class="media-title">
                    {move || current_file().map(|f| f.name).unwrap_or_default()}
                </div>
            </div>
        </div>
    }
}
