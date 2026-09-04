use leptos::*;
use leptos::html::{Video, Div};
use wasm_bindgen::JsCast;
use web_sys::{Event, MouseEvent, HtmlVideoElement};
use crate::shared::ui::play_pause_btn::PlayPauseMorphBtn;
use crate::shared::ui::volume_btn::VolumeBtn;
use crate::shared::ui::cc_btn::CcBtn;
use crate::shared::ui::speed_btn::SpeedBtn;
use crate::shared::ui::fullscreen_btn::FullscreenBtn;
use crate::shared::ui::prev_btn::PrevMorphBtn;
use crate::shared::ui::next_btn::NextMorphBtn;
use crate::shared::ui::progress_bar::ProgressBar;
use crate::shared::ui::skip_btn::SkipBtn;
// use crate::shared::ui::rotate_btn::RotateBtn;
use crate::shared::ui::playback_settings::PlaybackSettingsMenu;
use crate::shared::ui::speed_settings_modal::SpeedSettingsModal;
use crate::shared::ui::subtitle_settings_modal::SubtitleSettingsModal;
use web_sys::TouchEvent;

#[derive(Clone, Copy, PartialEq, Debug)]
enum SpeedGestureState {
    Idle,
    Holding,
}

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
pub fn YoutubePlayer(
    #[prop(into)] src: Signal<String>,
    #[prop(into)] on_ended: Callback<Event>,
    #[prop(into)] on_next: Callback<()>,
    #[prop(into)] on_prev: Callback<()>,
    #[prop(into)] has_next: Signal<bool>,
    #[prop(into)] has_prev: Signal<bool>,
    #[prop(into)] playback_mode: Signal<String>,
    #[prop(into)] on_playback_change: Callback<String>,
    #[prop(into)] video_speed: Signal<f64>,
    #[prop(into)] on_video_speed_change: Callback<f64>,
    #[prop(into)] subtitle_mode: Signal<u8>,
    #[prop(into)] on_subtitle_mode_change: Callback<u8>,
    #[prop(into)] show_remaining_time: Signal<bool>,
    #[prop(into)] on_show_remaining_time_change: Callback<bool>,
    #[prop(into)] auto_skip_enabled: Signal<bool>,
    #[prop(into)] on_auto_skip_enabled_change: Callback<bool>,
) -> impl IntoView {
    

    let (show_sub_modal, set_show_sub_modal) = create_signal(false);
    let (is_buffering, set_is_buffering) = create_signal(false);
    
    // Subtitle variables
    let (vtt_text, set_vtt_text) = create_signal(String::new());
    let (active_vtt_url, set_active_vtt_url) = create_signal(String::new());
    
    let (available_langs, set_available_langs) = create_signal(Vec::<String>::new());
    let (active_lang, set_active_lang) = create_signal(String::new());

    create_effect(move |_| {
        let active = active_lang.get();
        let s = src.get();
        if s.is_empty() || active.is_empty() { return; }
        
        let rel_path = if let Some(stripped) = s.strip_prefix("/storages/") {
            stripped.to_string()
        } else {
            s.clone()
        };
        let base_rel_path = if let Some(idx) = rel_path.rfind('.') {
            rel_path[..idx].to_string()
        } else {
            rel_path
        };
        
        spawn_local(async move {
            let vsub_url = format!("/api/v1/subtitle?file={}.vsub&lang={}", base_rel_path, active);
            if let Ok(resp) = gloo_net::http::Request::get(&vsub_url).send().await {
                if resp.ok() {
                    if let Ok(text) = resp.text().await {
                        set_vtt_text.set(text);
                    }
                }
            }
        });
    });

    create_effect(move |_| {
        let s = src.get();
        set_vtt_text.set(String::new());
        if s.is_empty() { return; }
        
        spawn_local(async move {
            let base = if let Some(idx) = s.rfind('.') { &s[..idx] } else { &s };
            let vsub_url = format!("{}.vsub", base);
            let vtt_url = format!("{}.vtt", base);

            let mut loaded_vtt = false;
            
            // Try .vtt first
            if let Ok(resp) = gloo_net::http::Request::get(&vtt_url).send().await {
                if resp.ok() {
                    if let Ok(text) = resp.text().await {
                        loaded_vtt = true;
                        set_vtt_text.set(text);
                        set_available_langs.set(vec![]);
                        set_active_lang.set(String::new());
                    }
                }
            }
            
            // Try .vsub if .vtt failed
            if !loaded_vtt {
                if let Ok(resp) = gloo_net::http::Request::get(&vsub_url).send().await {
                    if resp.ok() {
                        if let Ok(text) = resp.text().await {
                            match text.parse::<toml::Table>() {
                                Ok(table) => {
                                    // Extract languages in order of appearance
                                    let mut langs = Vec::new();
                                    for line in text.lines() {
                                        let line = line.trim();
                                        if line.starts_with('[') && line.ends_with(']') {
                                            let lang = line[1..line.len()-1].trim().to_string();
                                            if table.contains_key(&lang) && !langs.contains(&lang) {
                                                langs.push(lang);
                                            }
                                        }
                                    }
                                    // Fallback if no [section] found
                                    if langs.is_empty() {
                                        langs = table.keys().cloned().collect();
                                    }
                                    
                                    set_available_langs.set(langs.clone());
                                    if let Some(first) = langs.first() {
                                        set_active_lang.set(first.clone());
                                    }
                                }
                                Err(e) => {
                                    let err_msg = format!("Lỗi Parse TOML: {:?}", e);
                                    log::error!("Failed to parse .vsub TOML: {:?}", e);
                                    log::error!("File content: {}", text);
                                    set_available_langs.set(vec![err_msg.clone()]);
                                    set_active_lang.set(err_msg);
                                }
                            }
                        }
                    }
                }
            }
        });
    });

    create_effect(move |_| {
        let s = src.get();
        if s.is_empty() { return; }
        
        spawn_local(async move {
            let _ = gloo_timers::future::TimeoutFuture::new(50).await;
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(video) = document.query_selector(".youtube-video").unwrap_or(None) {
                        if let Ok(video_el) = video.dyn_into::<web_sys::HtmlVideoElement>() {
                            if !video_el.src().ends_with(&s) {
                                video_el.set_src(&s);
                            }
                        }
                    }
                }
            }
        });
    });

    let video_ref = create_node_ref::<Video>();

    window_event_listener(leptos::ev::online, move |_| {
        if let Some(video) = video_ref.get_untracked() {
            let net_state = video.network_state();
            if video.error().is_some() || net_state == 3 || net_state == 2 {
                let current_time = video.current_time();
                let is_paused = video.paused();
                log::info!("Network reconnected, reloading video at {}", current_time);
                video.load();
                
                let v2 = video.clone();
                let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
                    v2.set_current_time(current_time);
                    if !is_paused {
                        let _ = v2.play();
                    }
                });
                
                let _ = video.add_event_listener_with_callback("loadedmetadata", cb.unchecked_ref());
            }
        }
    });

    create_effect(move |_| {
        let text = vtt_text.get();
        let mode = subtitle_mode.get();
        
        let disable_tracks = || {
            if let Some(video) = video_ref.get_untracked() {
                if let Some(tracks) = video.text_tracks() {
                    for i in 0..tracks.length() {
                        if let Some(track) = tracks.get(i) {
                            track.set_mode(web_sys::TextTrackMode::Disabled);
                        }
                    }
                }
            }
        };

        if text.is_empty() || mode == 0 {
            disable_tracks();
            set_active_vtt_url.set(String::new());
            return;
        }
        
        let blocks = text.split("\n\n");
        let mut new_text = String::new();
        
        for block in blocks {
            if block.is_empty() { continue; }
            let mut lines: Vec<&str> = block.split('\n').collect();
            
            let mut is_cue = false;
            let mut text_start = 0;
            let mut timestamp_line = "";
            
            for (i, line) in lines.iter().enumerate() {
                if line.contains("-->") {
                    is_cue = true;
                    text_start = i + 1;
                    timestamp_line = line;
                    break;
                }
            }
            
            if is_cue && text_start < lines.len() {
                let mut text_lines = lines[text_start..].to_vec();
                
                let mut has_position = false;
                let mut is_top = timestamp_line.contains("{\\an7}") || timestamp_line.contains("{\\an8}") || timestamp_line.contains("{\\an9}");
                
                if let Some(idx) = timestamp_line.find("line:") {
                    has_position = true;
                    let rest = &timestamp_line[idx + 5..];
                    let end_idx = rest.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(rest.len());
                    let num_str = &rest[..end_idx];
                    if let Ok(num) = num_str.parse::<f32>() {
                        if num >= 0.0 && num < 50.0 {
                            is_top = true;
                        } else {
                            is_top = false;
                        }
                    }
                }
                
                if is_top {
                    has_position = true;
                }
                
                if has_position {
                    if mode == 2 && !is_top {
                        continue;
                    } else if mode == 3 && is_top {
                        continue;
                    }
                } else {
                    if text_lines.len() >= 2 {
                        if mode == 2 {
                            text_lines = vec![text_lines[0]];
                        } else if mode == 3 {
                            text_lines = vec![text_lines[text_lines.len() - 1]];
                        }
                    } else if text_lines.len() == 1 {
                        if mode == 2 {
                            continue;
                        }
                    }
                }
                
                lines.truncate(text_start);
                lines.extend(text_lines);
            }
            
            new_text.push_str(&lines.join("\n"));
            new_text.push_str("\n\n");
        }
        
        let encoded = js_sys::encode_uri_component(&new_text).as_string().unwrap_or_default();
        let data_uri = format!("data:text/vtt;charset=utf-8,{}", encoded);

        disable_tracks();

        set_active_vtt_url.set(data_uri);
    });

    let container_ref = create_node_ref::<Div>();

    
    let (is_playing, set_is_playing) = create_signal(true);
    let (current_time, set_current_time) = create_signal(0.0);
    let (duration, set_duration) = create_signal(0.0);
    let (volume, set_volume) = create_signal(1.0);
    let (is_muted, set_is_muted) = create_signal(false);
    let (is_fullscreen, set_is_fullscreen) = create_signal(false);
    let (show_controls, set_show_controls) = create_signal(true);
    let (is_seeking, set_is_seeking) = create_signal(false);
    let (was_playing, set_was_playing) = create_signal(false);
    let (buffer_percent, set_buffer_percent) = create_signal(0.0);
    
    let (skip_rules, set_skip_rules) = create_signal::<Vec<crate::utils::skip_parser::SkipRule>>(Vec::new());
    let skip_toast_msg = create_rw_signal::<Option<String>>(None);
    let skip_toast_timer = std::rc::Rc::new(std::cell::RefCell::new(None::<leptos::leptos_dom::helpers::TimeoutHandle>));

    // Keep video playback rate in sync with video_speed state
    create_effect(move |_| {
        let vs = video_speed.get();
        if let Some(video) = video_ref.get() {
            video.set_playback_rate(vs);
        }
    });

    // Auto play when src changes
    create_effect(move |_| {
        let current_src = src.get();
        if let Some(video) = video_ref.get() {
            set_is_playing.set(true);
            video.set_playback_rate(video_speed.get_untracked());
            let _ = video.play();
            
            if let Some(last_slash) = current_src.rfind('/') {
                let dir = &current_src[..=last_slash];
                let encoded_filename = &current_src[last_slash + 1..];
                // Manual url decode fallback, or just use it if we can't
                let filename = js_sys::decode_uri_component(encoded_filename)
                    .unwrap_or(js_sys::JsString::from(encoded_filename))
                    .as_string()
                    .unwrap_or(encoded_filename.to_string());
                
                let timestamp = js_sys::Date::now();
                let vskip_url = format!("{}.vskip?t={}", dir, timestamp);
                
                spawn_local(async move {
                    if let Ok(resp) = gloo_net::http::Request::get(&vskip_url).send().await {
                        if resp.ok() {
                            if let Ok(text) = resp.text().await {
                                let all_rules = crate::utils::skip_parser::parse_skip_rules(&text);
                                let mut active_rules = Vec::new();
                                let mut is_ignored = false;
                                for r in all_rules {
                                    if r.is_match(&filename) {
                                        if matches!(r.start, crate::utils::skip_parser::TimeSpec::Ignore) {
                                            is_ignored = true;
                                            break;
                                        }
                                        active_rules.push(r);
                                    }
                                }
                                if is_ignored {
                                    set_skip_rules.set(Vec::new());
                                } else {
                                    set_skip_rules.set(active_rules);
                                }
                            }
                        } else {
                            set_skip_rules.set(Vec::new());
                        }
                    }
                });
            }
        }
    });

    // 60Hz Progress Bar Update (16ms)
    create_effect(move |_| {
        if is_playing.get() {
            let toast_timer = skip_toast_timer.clone();
            let handle = set_interval_with_handle(
                move || {
                    if let Some(seeking) = is_seeking.try_get_untracked() {
                        if !seeking {
                            if let Some(video) = video_ref.get_untracked() {
                                let t = video.current_time();
                                let _ = set_current_time.try_set(t);
                                
                                // Auto skip logic
                                if auto_skip_enabled.get_untracked() {
                                    let rules = skip_rules.get_untracked();
                                    let dur = video.duration();
                                    if dur > 0.0 {
                                        for rule in rules {
                                            let s = rule.resolve_start(dur);
                                            let e = rule.resolve_end(dur);
                                            if t >= s && t < e - 0.2 { // Skip if we are inside the region
                                                video.set_current_time(e);
                                                
                                                // Handle Toast
                                                let msg = rule.message.clone().unwrap_or_else(|| "Tự động bỏ qua".to_string());
                                            skip_toast_msg.set(Some(msg));
                                            
                                            if let Some(h) = toast_timer.borrow_mut().take() { h.clear(); }
                                            let timer_inner = toast_timer.clone();
                                            let handle = set_timeout_with_handle(move || {
                                                skip_toast_msg.set(None);
                                            }, std::time::Duration::from_millis(3000));
                                            if let Ok(h) = handle { *timer_inner.borrow_mut() = Some(h); }
                                            
                                            break;
                                        }
                                    }
                                }
                                }
                            }
                        }
                    }
                },
                std::time::Duration::from_millis(16)
            );
            if let Ok(h) = handle {
                on_cleanup(move || h.clear());
            }
        }
    });



    let toggle_play = move || {
        if let Some(video) = video_ref.get() {
            if is_playing.get() {
                let _ = video.pause();
                set_is_playing.set(false);
            } else {
                let _ = video.play();
                set_is_playing.set(true);
            }
        }
    };

    let toggle_mute = move || {
        if let Some(video) = video_ref.get() {
            let m = !is_muted.get();
            video.set_muted(m);
            set_is_muted.set(m);
        }
    };

    let toggle_fullscreen = move || {
        if let Some(container) = container_ref.get() {
            let document = leptos::document();
            let is_full = document.fullscreen_element().is_some();
            
            if is_full {
                let _ = document.exit_fullscreen();
                set_is_fullscreen.set(false);
                let _ = js_sys::eval(
                    "if (window.activeWakeLock) {
                        window.activeWakeLock.release().then(() => {
                            window.activeWakeLock = null;
                        }).catch(e => console.log(e));
                    }"
                );
            } else {
                let _ = container.request_fullscreen();
                set_is_fullscreen.set(true);
                let _ = js_sys::eval(
                    "if ('wakeLock' in navigator) {
                        navigator.wakeLock.request('screen').then(lock => {
                            window.activeWakeLock = lock;
                        }).catch(e => console.log(e));
                    }"
                );
            }
        }
    };

    // Auto-release wake lock if user exits fullscreen via system gestures / Esc key
    create_effect(move |_| {
        let _ = js_sys::eval(
            "if (!window.hasFullscreenWakeLockListener) {
                window.hasFullscreenWakeLockListener = true;
                document.addEventListener('fullscreenchange', () => {
                    if (!document.fullscreenElement && window.activeWakeLock) {
                        window.activeWakeLock.release().then(() => {
                            window.activeWakeLock = null;
                        }).catch(e => console.log(e));
                    }
                });
            }"
        );
    });

    let update_buffer = move |target: &HtmlVideoElement| {
        let buffered = target.buffered();
        let dur = target.duration();
        let current = target.current_time();
        if dur > 0.0 && buffered.length() > 0 {
            let mut current_end = 0.0;
            for i in 0..buffered.length() {
                if let (Ok(start), Ok(end)) = (buffered.start(i), buffered.end(i)) {
                    // Find the buffer range that contains our current playback time
                    if current >= start && current <= end + 0.5 { // +0.5s tolerance
                        current_end = end;
                        break;
                    }
                }
            }
            // If we're not in any buffered range, it defaults to 0 (or we could keep the old value, but 0 correctly shows it's stalling)
            let _ = set_buffer_percent.try_set((current_end / dur) * 100.0);
        }
    };

    let handle_timeupdate = move |ev: Event| {
        if let Some(seeking) = is_seeking.try_get() {
            if seeking { return; }
        } else { return; }
        let target = ev.target().unwrap().unchecked_into::<HtmlVideoElement>();
        let _ = set_current_time.try_set(target.current_time());
        update_buffer(&target);
        
        // Aggressively disable embedded tracks that might have loaded late
        if let Some(tracks) = target.text_tracks() {
            for i in 0..tracks.length() {
                if let Some(track) = tracks.get(i) {
                    if track.id() != "custom-vtt-track" && track.mode() != web_sys::TextTrackMode::Disabled {
                        track.set_mode(web_sys::TextTrackMode::Disabled);
                    }
                }
            }
        }
    };

    let handle_loadedmetadata = move |ev: Event| {
        let target = ev.target().unwrap().unchecked_into::<HtmlVideoElement>();
        let _ = set_duration.try_set(target.duration());
        target.set_playback_rate(video_speed.get_untracked());
        
        // Disable embedded tracks that came with the video
        if let Some(tracks) = target.text_tracks() {
            for i in 0..tracks.length() {
                if let Some(track) = tracks.get(i) {
                    if track.id() != "custom-vtt-track" {
                        track.set_mode(web_sys::TextTrackMode::Disabled);
                    }
                }
            }
        }

        let _ = target.play();
    };

    let handle_ended = move |ev: Event| {
        let _ = set_is_playing.try_set(false);
        on_ended.call(ev);
    };

    let handle_progress = move |_ev| {
        if let Some(target) = video_ref.get() {
            update_buffer(&target);
        }
    };

    let handle_seek_start = move |_: ()| {
        let _ = set_is_seeking.try_set(true);
        if let Some(playing) = is_playing.try_get_untracked() {
            let _ = set_was_playing.try_set(playing);
            if playing {
                if let Some(video) = video_ref.get() {
                    let _ = video.pause(); // Pause while seeking
                }
            }
        }
    };

    let handle_seek = move |val: f64| {
        let time = (val / 100.0) * duration.get();
        set_is_seeking.set(true); // Failsafe
        set_current_time.set(time);
        if let Some(video) = video_ref.get() {
            video.set_current_time(time);
        }
    };

    let handle_seek_end = move |_: ()| {
        if was_playing.get_untracked() {
            if let Some(video) = video_ref.get() {
                let _ = video.play();
            }
        }
        
        set_timeout(move || {
            let _ = set_is_seeking.try_set(false);
        }, std::time::Duration::from_millis(100));
    };

    let (show_vol_slider, set_show_vol_slider) = create_signal(false);
    let (vol_hide_counter, set_vol_hide_counter) = create_signal(0_u32);

    let reset_vol_timeout = move || {
        let current = if let Some(v) = vol_hide_counter.try_get() { v + 1 } else { return; };
        let _ = set_vol_hide_counter.try_set(current);
        
        set_timeout(move || {
            if let Some(counter) = vol_hide_counter.try_get() {
                if counter == current {
                    let _ = set_show_vol_slider.try_set(false);
                }
            }
        }, std::time::Duration::from_secs(3));
    };

    let handle_volume = move |ev: Event| {
        if let Some(target) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
            if let Ok(v) = target.value().parse::<f64>() {
                set_volume.set(v);
                if v > 0.0 && is_muted.get() {
                    set_is_muted.set(false);
                }
                if let Some(video) = video_ref.get() {
                    video.set_volume(v);
                    video.set_muted(is_muted.get());
                }
                reset_vol_timeout();
            }
        }
    };

    let is_touch_device = create_rw_signal(false);
    let last_touch_time = create_rw_signal(0.0);
    
    let hide_controls_timer = std::rc::Rc::new(std::cell::RefCell::new(None::<leptos::leptos_dom::helpers::TimeoutHandle>));

    /****
     * show_controls_with_timer – Hàm dùng chung cho cả container và video element.
     * Khi chuột di chuyển trên bất kỳ phần tử nào, hiện controls rồi đặt timer 3s để tự ẩn.
     * Nhận vào Rc<RefCell<Option<TimeoutHandle>>> để huỷ timer cũ trước khi đặt timer mới.
     ****/
    let show_controls_with_timer = {
        let timer = hide_controls_timer.clone();
        std::rc::Rc::new(move || {
            if is_touch_device.get_untracked() { return; }
            
            set_show_controls.set(true);
            if let Some(h) = timer.borrow_mut().take() {
                h.clear();
            }
            let timer_inner = timer.clone();
            let handle = set_timeout_with_handle(move || {
                if let Some(playing) = is_playing.try_get_untracked() {
                    if playing {
                        let _ = set_show_controls.try_set(false);
                    }
                }
            }, std::time::Duration::from_millis(3000));
            if let Ok(h) = handle {
                *timer_inner.borrow_mut() = Some(h);
            }
        })
    };

    let show_controls_fn1 = show_controls_with_timer.clone();
    let handle_mousemove = move |_| {
        show_controls_fn1();
    };


    
    let handle_mouseleave = move |_| {
        if let Some(playing) = is_playing.try_get() {
            if playing {
                let _ = set_show_controls.try_set(false);
            }
        }
    };
    
    let apply_speed = move |s: f64| {
        let s = (s * 100.0).round() / 100.0;
        let s = s.clamp(0.25, 4.0);
        on_video_speed_change.call(s);
        if let Some(video) = video_ref.get_untracked() {
            video.set_playback_rate(s);
        }
    };

    let (show_speed_modal, set_show_speed_modal) = create_signal(false);
    let speed_state = create_rw_signal(SpeedGestureState::Idle);
    let base_speed = create_rw_signal(1.0);
    let start_point = create_rw_signal((0.0, 0.0));
    let is_interacting = create_rw_signal(false);
    let mousedown_time = create_rw_signal(js_sys::Date::now());
    let hold_timer = std::rc::Rc::new(std::cell::RefCell::new(None::<leptos::leptos_dom::helpers::TimeoutHandle>));

    let (show_seek_overlay, set_show_seek_overlay) = create_signal(false);
    let seek_dir = create_rw_signal(crate::shared::ui::double_tap_seek::SeekDirection::Left);
    let seek_accumulated = create_rw_signal(0);
    let seek_timer = std::rc::Rc::new(std::cell::RefCell::new(None::<leptos::leptos_dom::helpers::TimeoutHandle>));

    let (is_rotated, set_is_rotated) = create_signal(false);
    let _handle_rotate = move |_: leptos::ev::MouseEvent| {
        set_is_rotated.update(|r| *r = !*r);
    };
    let last_tap_time = create_rw_signal(0.0);
    let tap_count = create_rw_signal(0);
    
    let start_gesture = move |x: f64, y: f64, timer: std::rc::Rc<std::cell::RefCell<Option<leptos::leptos_dom::helpers::TimeoutHandle>>>| {
        is_interacting.set(true);
        mousedown_time.set(js_sys::Date::now());
        start_point.set((x, y));
        
        base_speed.set(video_speed.get_untracked());

        if let Some(h) = timer.borrow_mut().take() { h.clear(); }
        
        let handle = set_timeout_with_handle(move || {
            let _ = speed_state.try_set(SpeedGestureState::Holding);
            apply_speed(2.0);
        }, std::time::Duration::from_millis(300));
        if let Ok(h) = handle { *timer.borrow_mut() = Some(h); }
    };

    let move_gesture = move |_x: f64, _y: f64| {
        // No dragging logic anymore, just holding
    };

    // Clone riêng cho video element – đảm bảo mousemove trên <video> ở fullscreen cũng hiện controls
    let show_controls_fn2 = show_controls_with_timer.clone();
    let handle_video_mousemove_controls = move |ev: MouseEvent| {
        show_controls_fn2();
        // Đồng thời vẫn gọi logic speed gesture nếu có
        move_gesture(ev.client_x() as f64, ev.client_y() as f64);
    };

    let end_gesture = move |timer: std::rc::Rc<std::cell::RefCell<Option<leptos::leptos_dom::helpers::TimeoutHandle>>>| {
        if let Some(interacting) = is_interacting.try_get_untracked() {
            if !interacting { return; }
        } else {
            return;
        }
        let _ = is_interacting.try_set(false);
        if let Some(h) = timer.borrow_mut().take() {
            h.clear();
        }
        
        if let Some(state) = speed_state.try_get_untracked() {
            if state == SpeedGestureState::Holding {
                let _ = speed_state.try_set(SpeedGestureState::Idle);
                if let Some(s) = base_speed.try_get_untracked() {
                    apply_speed(s);
                }
            }
        }
    };

    let ht1 = hold_timer.clone();
    let on_video_mousedown = move |ev: MouseEvent| {
        if ev.button() == 0 { start_gesture(ev.client_x() as f64, ev.client_y() as f64, ht1.clone()); }
    };
    let ht2 = hold_timer.clone();
    let on_video_mouseup = move |_: MouseEvent| { end_gesture(ht2.clone()); };
    let ht3 = hold_timer.clone();
    let on_video_mouseleave = move |_: MouseEvent| { end_gesture(ht3.clone()); };
    
    let ht4 = hold_timer.clone();
    let on_video_touchstart = move |ev: TouchEvent| {
        is_touch_device.set(true);
        last_touch_time.set(js_sys::Date::now());

        if let Some(touch) = ev.touches().item(0) {
            start_gesture(touch.client_x() as f64, touch.client_y() as f64, ht4.clone());
        }
    };
    let on_video_touchmove = move |ev: TouchEvent| {
        if let Some(touch) = ev.touches().item(0) {
            move_gesture(touch.client_x() as f64, touch.client_y() as f64);
        }
    };
    let ht5 = hold_timer.clone();
    let on_video_touchend = move |_: TouchEvent| { end_gesture(ht5.clone()); };

    let timer_clone6 = seek_timer.clone();

    let handle_video_click = move |ev: MouseEvent| {
        let duration = js_sys::Date::now() - mousedown_time.get_untracked();
        if duration > 200.0 { return; }
        
        let now = js_sys::Date::now();
        let diff = now - last_tap_time.get_untracked();
        
        if diff < 300.0 {
            tap_count.update(|c| *c += 1);
            
            use wasm_bindgen::JsCast;
            if let Some(target) = ev.target() {
                if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                    let x = ev.offset_x() as f64;
                    let width = el.client_width() as f64;
                    let ratio = x / width;
                if ratio < 0.33 || ratio > 0.67 {
                    // Undo first tap action on 2nd tap
                    if tap_count.get_untracked() == 2 {
                        if is_touch_device.get_untracked() {
                            let current = show_controls.get_untracked();
                            set_show_controls.set(!current);
                        } else {
                            toggle_play();
                        }
                    }

                    if ratio < 0.33 {
                        seek_dir.set(crate::shared::ui::double_tap_seek::SeekDirection::Left);
                        seek_accumulated.update(|s| *s -= 10);
                        if let Some(v) = video_ref.get_untracked() {
                            v.set_current_time(v.current_time() - 10.0);
                        }
                    } else {
                        seek_dir.set(crate::shared::ui::double_tap_seek::SeekDirection::Right);
                        seek_accumulated.update(|s| *s += 10);
                        if let Some(v) = video_ref.get_untracked() {
                            v.set_current_time(v.current_time() + 10.0);
                        }
                    }
                    
                    set_show_seek_overlay.set(true);
                    
                    if let Some(h) = seek_timer.borrow_mut().take() { h.clear(); }
                    let timer_inner = seek_timer.clone();
                    let handle = set_timeout_with_handle(move || {
                        seek_accumulated.set(0);
                        set_show_seek_overlay.set(false);
                    }, std::time::Duration::from_millis(500));
                    if let Ok(h) = handle { *timer_inner.borrow_mut() = Some(h); }
                    
                } else {
                    // Middle 34% -> Fullscreen
                    if tap_count.get_untracked() == 2 {
                        if is_touch_device.get_untracked() {
                            let current = show_controls.get_untracked();
                            set_show_controls.set(!current);
                        } else {
                            toggle_play();
                        }
                        toggle_fullscreen();
                    }
                }
                }
            }
            last_tap_time.set(now);
            return;
        }
        
        last_tap_time.set(now);
        tap_count.set(1);
        
        if is_touch_device.get_untracked() {
            let current = show_controls.get_untracked();
            set_show_controls.set(!current);
        } else {
            toggle_play();
        }
    };

    let progress_percent = move || {
        let d = duration.get();
        if d > 0.0 {
            (current_time.get() / d) * 100.0
        } else {
            0.0
        }
    };

    let timer_clone4 = hide_controls_timer.clone();
    let timer_clone5 = hold_timer.clone();
    on_cleanup(move || {
        if let Some(h) = timer_clone4.borrow_mut().take() { h.clear(); }
        if let Some(h) = timer_clone5.borrow_mut().take() { h.clear(); }
        if let Some(h) = timer_clone6.borrow_mut().take() { h.clear(); }
    });

    view! {
        <div class="youtube-player-container" node_ref=container_ref
             on:mousemove=handle_mousemove 
             on:mouseleave=handle_mouseleave
             class:fullscreen=move || is_fullscreen.get()
             class:hide-controls=move || !show_controls.get() || speed_state.get() != SpeedGestureState::Idle
             class:is-rotated-container=move || is_rotated.get()
        >
             <div class="youtube-player-inner" class:is-rotated=move || is_rotated.get()>
             <div class="yt-video-title">
                 {move || {
                    let current_src = src.get();
                    if let Some(last_slash) = current_src.rfind('/') {
                        let encoded_filename = &current_src[last_slash + 1..];
                        js_sys::decode_uri_component(encoded_filename)
                            .unwrap_or(js_sys::JsString::from(encoded_filename))
                            .as_string()
                            .unwrap_or(encoded_filename.to_string())
                    } else {
                        String::new()
                    }
                 }}
             </div>

             <div style={move || if show_controls.get() { "opacity: 1; transition: opacity 0.3s; z-index: 1000;" } else { "opacity: 0; pointer-events: none; transition: opacity 0.3s; z-index: 1000;" }}>
                 <PlaybackSettingsMenu playback_mode=playback_mode on_playback_change=on_playback_change />
             </div>
             <Show when=move || show_speed_modal.get() fallback=|| ()>
                 <SpeedSettingsModal speed=video_speed set_speed=apply_speed on_close=move |_| set_show_speed_modal.set(false) />
             </Show>
             
             <Show when=move || show_sub_modal.get() fallback=|| ()>
                 <SubtitleSettingsModal 
                     mode=subtitle_mode 
                     set_mode=on_subtitle_mode_change 
                     on_close=move |_| set_show_sub_modal.set(false) 
                     available_langs=available_langs
                     active_lang=active_lang
                     set_active_lang=move |lang| set_active_lang.set(lang)
                 />
             </Show>
             
             <crate::shared::ui::double_tap_seek::DoubleTapSeekOverlay 
                 show=show_seek_overlay
                 direction=seek_dir
                 accumulated_seconds=seek_accumulated
             />

             <div class="yt-loading-spinner" style=move || if is_buffering.get() { "opacity: 1; pointer-events: auto;" } else { "opacity: 0; pointer-events: none;" }>
                <svg class="spinner" viewBox="0 0 50 50">
                    <circle class="path" cx="25" cy="25" r="20" fill="none" stroke-width="4"></circle>
                </svg>
             </div>
             
             <video 
                 node_ref=video_ref
                 class="youtube-video"
                 style="touch-action: none;"
                 autoplay=true
                 preload="auto"
                 on:timeupdate=handle_timeupdate
                 on:loadedmetadata=handle_loadedmetadata
                 on:ended=handle_ended
                 on:progress=handle_progress
                 on:waiting=move |_| { set_is_buffering.set(true); }
                 on:playing=move |_| { set_is_buffering.set(false); let _ = set_is_playing.try_set(true); }
                 on:canplay=move |_| { set_is_buffering.set(false); }
                 on:loadeddata=move |_| { set_is_buffering.set(false); }
                 on:pause=move |_| { let _ = set_is_playing.try_set(false); }
                 on:click=handle_video_click
                 on:mousedown=on_video_mousedown
                 on:mousemove=handle_video_mousemove_controls
                 on:mouseup=on_video_mouseup
                 on:mouseleave=on_video_mouseleave
                 on:touchstart=on_video_touchstart
                 on:touchmove=on_video_touchmove
                 on:touchend=on_video_touchend
                 on:contextmenu=move |ev| ev.prevent_default()
             >
                 <For
                    each=move || {
                        let url = active_vtt_url.get();
                        if url.is_empty() { vec![] } else { vec![url] }
                    }
                    key=|url| url.clone()
                    children=move |url| {
                        view! {
                            <track id="custom-vtt-track" src=url kind="subtitles" srclang="vi" label="Subtitles" default />
                        }
                    }
                />
             </video>
             
             <div class="yt-speed-overlay" 
                  style={move || {
                      if speed_state.get() == SpeedGestureState::Holding {
                          "opacity: 1; transform: translateX(-50%) translateY(0);" 
                      } else { 
                          "opacity: 0; transform: translateX(-50%) translateY(-10px);" 
                      }
                  }}>
                 <div class="yt-speed-content" style="display: flex; flex-direction: column; align-items: center; gap: 4px;">
                     <div class="yt-speed-main" style="display: flex; align-items: center; gap: 4px;">
                         <span class="yt-speed-text">"2.0x"</span>
                         <div class="yt-speed-arrows">
                             <svg class="arrow1" viewBox="0 0 36 36" fill="currentColor" width="24" height="24">
                                 <path d="M 12 10 L 18 14 L 18 22 L 12 26 Z M 18 14 L 24 18 L 24 18 L 18 22 Z"></path>
                             </svg>
                             <svg class="arrow2" viewBox="0 0 36 36" fill="currentColor" width="24" height="24">
                                 <path d="M 12 10 L 18 14 L 18 22 L 12 26 Z M 18 14 L 24 18 L 24 18 L 18 22 Z"></path>
                             </svg>
                         </div>
                     </div>
                 </div>
             </div>


             <div class="skip-toast-notification" class=("show", move || skip_toast_msg.get().is_some())>
                 {move || skip_toast_msg.get().unwrap_or_default()}
             </div>

             <div class="youtube-controls-wrapper">
                 <div class="youtube-controls-top" style="display: flex; justify-content: flex-end; width: 100%; padding: 0 16px; margin-bottom: -4px; gap: 8px;">
                    <SkipBtn 
                        enabled=auto_skip_enabled 
                        on_click=move |_| on_auto_skip_enabled_change.call(!auto_skip_enabled.get_untracked()) 
                    />
                    {/* <RotateBtn is_rotated=is_rotated on_click=_handle_rotate /> */}

                    <CcBtn 
                        mode=subtitle_mode
                        on_toggle=move |_| {
                            let current = subtitle_mode.get_untracked();
                            if current == 0 {
                                on_subtitle_mode_change.call(1);
                            } else {
                                on_subtitle_mode_change.call(0);
                            }
                        }
                        on_long_press=move |_| set_show_sub_modal.set(true)
                    />
                    <SpeedBtn on_click=move |_| set_show_speed_modal.set(true) />
                </div>
                 <ProgressBar 
                    progress=Signal::derive(move || progress_percent()) 
                    buffer=buffer_percent 
                    duration=duration
                    on_seek=handle_seek 
                    on_seek_start=handle_seek_start 
                    on_seek_end=handle_seek_end 
                />

                <div class="youtube-controls-bottom">
                    <div class="controls-left">
                        <PrevMorphBtn 
                            on_click=move |_| on_prev.call(())
                            disabled=Signal::derive(move || !has_prev.get())
                        />
                        
                        <PlayPauseMorphBtn 
                            is_playing=is_playing
                            on_click=move |_| toggle_play()
                        />

                        <NextMorphBtn 
                            on_click=move |_| on_next.call(())
                            disabled=Signal::derive(move || !has_next.get())
                        />

                        <div class="volume-container" class=("force-show", move || show_vol_slider.get())>
                            <VolumeBtn 
                                volume=volume
                                is_muted=is_muted
                                on_click=move |e: ev::MouseEvent| {
                                    use wasm_bindgen::JsCast;
                                    if let Some(pe) = e.dyn_ref::<web_sys::PointerEvent>() {
                                        if pe.pointer_type() == "touch" {
                                            if !show_vol_slider.get() {
                                                set_show_vol_slider.set(true);
                                                reset_vol_timeout();
                                                return;
                                            }
                                            reset_vol_timeout();
                                        }
                                    }
                                    toggle_mute();
                                }
                            />
                            <input type="range" class="volume-slider" min="0" max="1" step="0.05" 
                                   value=move || if is_muted.get() { 0.0 } else { volume.get() }
                                   on:input=handle_volume 
                            />
                        </div>

                        <div class="time-display" on:click=move |_| on_show_remaining_time_change.call(!show_remaining_time.get()) style="cursor: pointer; user-select: none;">
                            <span>{move || {
                                if show_remaining_time.get() {
                                    let remaining = duration.get() - current_time.get();
                                    if remaining > 0.0 {
                                        format!("-{}", format_time(remaining))
                                    } else {
                                        format_time(0.0)
                                    }
                                } else {
                                    format_time(current_time.get())
                                }
                            }}</span>
                            <span>" / "</span>
                            <span>{move || format_time(duration.get())}</span>
                        </div>
                    </div>

                    <div class="controls-right">
                        <FullscreenBtn 
                            is_fullscreen=is_fullscreen
                            on_click=move |_| toggle_fullscreen()
                        />
                    </div>
                </div>
            </div>
             </div>
        </div>
    }
}
