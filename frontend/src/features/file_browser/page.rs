use leptos::*;
use leptos_router::*;
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use std::collections::HashSet;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = processDropEvent)]
    async fn process_drop_event(ev: &web_sys::DragEvent) -> Result<js_sys::Array, wasm_bindgen::JsValue>;

    #[wasm_bindgen(js_name = initCodeMirror)]
    fn init_code_mirror(textarea_id: &str, ext: &str, is_dark: bool) -> JsValue;

    #[wasm_bindgen(js_name = getCodeMirrorValue)]
    fn get_code_mirror_value() -> String;
}

use crate::features::file_browser::api::{fetch_files, fetch_disk_usage, upload_chunk, DiskUsageInfo, regen_single_thumbnail};

use crate::features::file_browser::media_viewer::MediaViewer;
use crate::shared::ui::checkbox::Checkbox;

#[derive(Clone)]
pub struct ConflictInfo {
    pub source_name: String,
    pub dest_name: String,
    pub source_size: u64,
    pub dest_size: u64,
    pub source_mtime: u64,
    pub dest_mtime: u64,
}

#[derive(Clone, PartialEq)]
pub enum ConflictResolution {
    Overwrite,
    Skip,
    Rename,
}

#[derive(Clone)]
pub struct ConflictState {
    pub info: ConflictInfo,
    pub tx: std::rc::Rc<std::cell::RefCell<Option<futures::channel::oneshot::Sender<(ConflictResolution, bool)>>>>,
}

#[derive(Clone, Debug, Default)]
pub struct UploadProgressState {
    pub is_visible: bool,
    pub is_minimized: bool,
    pub current_file_name: String,
    pub current_file_progress: f64,
    pub files_completed: usize,
    pub total_files: usize,
    pub total_bytes_uploaded: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub time_remaining_sec: u64,
}

#[component]
pub fn FileBrowserPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = store_value(use_navigate());
    let current_path = move || {
        let p = params.with(|p| p.get("path").cloned().unwrap_or_default());
        if let Ok(decoded) = js_sys::decode_uri_component(&p) {
            String::from(decoded)
        } else {
            p
        }
    };
    
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);
    let (selected_files, set_selected_files) = create_signal::<HashSet<String>>(HashSet::new());
    
    // Tự động bỏ chọn tất cả khi chuyển thư mục
    create_effect(move |_| {
        let _ = current_path();
        set_selected_files.set(HashSet::new());
    });
    let (clipboard_action, set_clipboard_action) = create_signal::<Option<String>>(None);
    let (clipboard_files, set_clipboard_files) = create_signal::<HashSet<String>>(HashSet::new());
    
    let (rename_dialog_open, set_rename_dialog_open) = create_signal(false);
    let (rename_file_name, set_rename_file_name) = create_signal("".to_string());
    let (rename_new_name, set_rename_new_name) = create_signal("".to_string());

    let (bulk_rename_dialog_open, set_bulk_rename_dialog_open) = create_signal(false);
    let (bulk_common_name, set_bulk_common_name) = create_signal("".to_string());
    let (bulk_start_index, set_bulk_start_index) = create_signal("01".to_string());
    let (bulk_extension, set_bulk_extension) = create_signal("Mặc định".to_string());
    let (is_loading_bulk_rename, set_is_loading_bulk_rename) = create_signal(false);

    let (input_modal_open, set_input_modal_open) = create_signal(false);
    let (input_modal_title, set_input_modal_title) = create_signal("".to_string());
    let (input_modal_value, set_input_modal_value) = create_signal("".to_string());
    let (input_modal_mode, set_input_modal_mode) = create_signal("folder".to_string());
    
    let (show_delete_modal, set_show_delete_modal) = create_signal(false);
    let (delete_paths, set_delete_paths) = create_signal::<Vec<String>>(Vec::new());
    let (move_to_trash, set_move_to_trash) = create_signal(true);

    let (show_trash_modal, set_show_trash_modal) = create_signal(false);
    let (trash_refresh, set_trash_refresh) = create_signal(0);
    let trash_resource = create_resource(
        move || (show_trash_modal.get(), trash_refresh.get()),
        |(show, _)| async move { 
            if show { 
                crate::features::file_browser::api::get_trash().await.unwrap_or_default() 
            } else { 
                Vec::new() 
            } 
        }
    );
    
    let query = use_query_map();
    let upload_state = create_rw_signal(UploadProgressState::default());
    
    let conflict_state = create_rw_signal::<Option<ConflictState>>(None);
    let (conflict_apply_all, set_conflict_apply_all) = create_signal(false);
    
    // Sync conflict URL parameter with modal state
    create_effect(move |prev: Option<bool>| {
        let is_conflict_open = conflict_state.get().is_some();
        if is_conflict_open != prev.unwrap_or(false) {
            let mut q = query.get_untracked();
            if is_conflict_open {
                q.insert("conflict".to_string(), "true".to_string());
            } else {
                q.remove("conflict");
            }
            let qs = q.to_query_string();
            let pathname = use_location().pathname.get_untracked();
            let current_url = if qs.is_empty() { pathname } else { format!("{}?{}", pathname, qs) };
            navigate.with_value(|n| n(&current_url, NavigateOptions::default()));
        }
        is_conflict_open
    });
    
    // Popstate: when user presses Back and URL changes, if `conflict` parameter is gone, close the modal
    create_effect(move |_| {
        let has_conflict_param = query.with(|q| q.get("conflict").is_some());
        if !has_conflict_param && conflict_state.get_untracked().is_some() {
            // Close the modal and send a "Skip" resolution
            if let Some(state) = conflict_state.get_untracked() {
                if let Some(tx) = state.tx.borrow_mut().take() {
                    let _ = tx.send((ConflictResolution::Skip, false));
                }
            }
            conflict_state.set(None);
        }
    });

    // Giữ sáng màn hình khi modal xử lý file hiển thị (Upload, Cut, Copy)
    create_effect(move |_| {
        let is_visible = upload_state.get().is_visible;
        if is_visible {
            let _ = js_sys::eval(
                "if ('wakeLock' in navigator) {
                    navigator.wakeLock.request('screen').then(lock => {
                        window.activeUploadWakeLock = lock;
                    }).catch(e => console.log('WakeLock error:', e));
                }"
            );
        } else {
            let _ = js_sys::eval(
                "if (window.activeUploadWakeLock) {
                    window.activeUploadWakeLock.release().then(() => {
                        window.activeUploadWakeLock = null;
                    }).catch(e => console.log('WakeLock release error:', e));
                }"
            );
        }
    });

    let files_resource = create_resource(
        move || (current_path(), refresh_trigger.get()),
        |(path, _)| async move { fetch_files(&path).await.unwrap_or_default() }
    );

    
    let (smart_rename_enabled, set_smart_rename_enabled) = create_signal(false);
    let (smart_rename_template, set_smart_rename_template) = create_signal("".to_string());
    let (use_file_time, set_use_file_time) = create_signal(true);
    let (playback_mode, set_playback_mode) = create_signal("once".to_string());
    let (video_speed, set_video_speed) = create_signal(1.0);
    let (subtitle_mode, set_subtitle_mode) = create_signal(1_u8);
    let (show_remaining_time, set_show_remaining_time) = create_signal(false);
    let (auto_skip_enabled, set_auto_skip_enabled) = create_signal(true);
    
    let (sort_by, set_sort_by) = create_signal("name".to_string());
    let (sort_desc, set_sort_desc) = create_signal(false);

    let files_memo = create_memo(move |_| {
        let mut files = files_resource.get().unwrap_or_default();
        files.sort_by(|a, b| {
            let dir_cmp = b.is_dir.cmp(&a.is_dir);
            if dir_cmp != std::cmp::Ordering::Equal {
                return dir_cmp;
            }
            
            let sort_key = sort_by.get();
            let desc = sort_desc.get();
            
            let cmp = match sort_key.as_str() {
                "size" => a.size.cmp(&b.size),
                "time" => a.modified_at.cmp(&b.modified_at),
                "type" => {
                    let ext_a = a.name.rsplit('.').next().unwrap_or("").to_lowercase();
                    let ext_b = b.name.rsplit('.').next().unwrap_or("").to_lowercase();
                    ext_a.cmp(&ext_b).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                },
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            };
            
            if desc { cmp.reverse() } else { cmp }
        });
        files
    });

    create_effect(move |_| {
        let path = current_path();
        spawn_local(async move {
            if let Ok(settings) = crate::features::file_browser::api::fetch_folder_settings(&path).await {
                set_sort_by.set(settings.sort_by);
                set_sort_desc.set(settings.sort_desc);
                set_smart_rename_enabled.set(settings.rename_enabled);
                set_smart_rename_template.set(settings.rename_template);
                set_use_file_time.set(settings.use_file_time);
                set_playback_mode.set(settings.playback_mode);
                set_video_speed.set(settings.video_speed);
                set_subtitle_mode.set(settings.subtitle_mode);
                set_show_remaining_time.set(settings.show_remaining_time);
                set_auto_skip_enabled.set(settings.auto_skip_enabled);
            }
        });
    });

    let save_settings = move || {
        let path = current_path();
        let settings = crate::features::file_browser::api::FolderSettings {
            sort_by: sort_by.get_untracked(),
            sort_desc: sort_desc.get_untracked(),
            rename_enabled: smart_rename_enabled.get_untracked(),
            rename_template: smart_rename_template.get_untracked(),
            use_file_time: use_file_time.get_untracked(),
            playback_mode: playback_mode.get_untracked(),
            video_speed: video_speed.get_untracked(),
            subtitle_mode: subtitle_mode.get_untracked(),
            show_remaining_time: show_remaining_time.get_untracked(),
            auto_skip_enabled: auto_skip_enabled.get_untracked(),
        };
        spawn_local(async move {
            let _ = crate::features::file_browser::api::save_folder_settings(&path, settings).await;
        });
    };

    let update_sort = move |new_sort_by: &str, new_sort_desc: bool| {
        set_sort_by.set(new_sort_by.to_string());
        set_sort_desc.set(new_sort_desc);
        save_settings();
    };

    let update_rename_enabled = move |checked: bool| {
        set_smart_rename_enabled.set(checked);
        save_settings();
    };

    let update_rename_template = move |val: String| {
        set_smart_rename_template.set(val);
        save_settings();
    };
    
    let update_use_file_time = move |checked: bool| {
        set_use_file_time.set(checked);
        save_settings();
    };

    let update_playback_mode = move |mode: String| {
        set_playback_mode.set(mode);
        save_settings();
    };

    let update_video_speed = move |speed: f64| {
        set_video_speed.set(speed);
        save_settings();
    };

    let update_subtitle_mode = move |mode: u8| {
        set_subtitle_mode.set(mode);
        save_settings();
    };

    let update_show_remaining_time = move |show: bool| {
        set_show_remaining_time.set(show);
        save_settings();
    };

    let update_auto_skip_enabled = move |enabled: bool| {
        set_auto_skip_enabled.set(enabled);
        save_settings();
    };
    let (show_settings, set_show_settings) = create_signal(false);
    let (show_more_menu, set_show_more_menu) = create_signal(false);
    let (show_new_menu, set_show_new_menu) = create_signal(false);
    let (properties_modal_data, set_properties_modal_data) = create_signal::<Option<crate::features::file_browser::api::FileProperties>>(None);
    let (properties_thumb_data, set_properties_thumb_data) = create_signal::<Option<String>>(None);
    let (checksum_data, set_checksum_data) = create_signal::<Option<crate::features::file_browser::api::ChecksumResponse>>(None);
    let (show_checksum_modal, set_show_checksum_modal) = create_signal(false);
    let (is_loading_checksum, set_is_loading_checksum) = create_signal(false);
    
    let text_editor_file = create_memo(move |_| query.with(|q| q.get("edit").cloned()));
    let (text_editor_content, set_text_editor_content) = create_signal("".to_string());
    let (is_saving_text, set_is_saving_text) = create_signal(false);
    let (is_editing_text, set_is_editing_text) = create_signal(false);
    
    let update_text_editor_file = move |file: Option<String>| {
        let mut q = query.get_untracked();
        if let Some(f) = file {
            q.insert("edit".to_string(), f);
        } else {
            q.remove("edit");
        }
        let qs = q.to_query_string();
        let pathname = use_location().pathname.get_untracked();
        let current_url = if qs.is_empty() {
            pathname
        } else {
            format!("{}?{}", pathname, qs)
        };
        navigate.with_value(|n| n(&current_url, NavigateOptions::default()));
    };

    create_effect(move |prev: Option<Option<String>>| {
        let current = text_editor_file.get();
        if current != prev.unwrap_or_default() {
            if let Some(path) = current.clone() {
                leptos::spawn_local(async move {
                    if let Ok(content) = crate::features::file_browser::api::read_text_file(&path).await {
                        set_text_editor_content.set(content);
                        set_is_editing_text.set(false);
                    }
                });
            }
        }
        current
    });

    let active_media_index = create_rw_signal::<Option<usize>>(None);
    
    // Sync URL -> active_media_index
    create_effect(move |_| {
        let media_path = query.with(|q| q.get("media").cloned());
        if let Some(path) = media_path {
            let current_idx = active_media_index.get_untracked();
            let new_idx = files_memo.with(|f| f.iter().position(|item| item.path == path));
            if new_idx != current_idx {
                active_media_index.set(new_idx);
            }
        } else {
            if active_media_index.get_untracked().is_some() {
                active_media_index.set(None);
            }
        }
    });

    // Sync active_media_index -> URL
    create_effect(move |prev: Option<Option<usize>>| {
        let current_idx = active_media_index.get();
        if current_idx != prev.unwrap_or(None) {
            let path = if let Some(idx) = current_idx {
                files_memo.with(|f| f.get(idx).map(|item| item.path.clone()))
            } else {
                None
            };
            
            let current_url_media = query.with_untracked(|q| q.get("media").cloned());
            if current_url_media != path {
                let mut q = query.get_untracked();
                if let Some(p) = path {
                    q.insert("media".to_string(), p);
                } else {
                    q.remove("media");
                }
                
                let qs = q.to_query_string();
                let pathname = use_location().pathname.get_untracked();
                let current_url = if qs.is_empty() { pathname } else { format!("{}?{}", pathname, qs) };
                
                navigate.with_value(|n| n(&current_url, NavigateOptions::default()));
            }
        }
        current_idx
    });
    // --- Theme Logic ---
    let (theme, set_theme) = create_signal({
        let mut t = "dark".to_string();
        if let Some(w) = web_sys::window() {
            if let Ok(Some(ls)) = w.local_storage() {
                if let Ok(Some(st)) = ls.get_item("theme") {
                    if st == "light" || st == "dark" { t = st; }
                }
            }
        }
        t
    });

    let toggle_theme = move |_| {
        set_theme.update(|t| {
            *t = if *t == "dark" { "light".to_string() } else { "dark".to_string() };
            if let Some(w) = web_sys::window() {
                if let Some(doc) = w.document() {
                    if let Some(html) = doc.document_element() {
                        let _ = html.class_list().remove_2("light", "dark");
                        let _ = html.class_list().add_1(t);
                    }
                }
                if let Ok(Some(ls)) = w.local_storage() {
                    let _ = ls.set_item("theme", t);
                }
            }
        });
        set_show_settings.set(false);
    };



    let disk_resource = create_resource(
        move || refresh_trigger.get(),
        |_| async move { fetch_disk_usage().await.unwrap_or(DiskUsageInfo { total_space: 1, used_space: 0 }) }
    );

    let format_size = |bytes: u64| -> String {
        let kb = 1024_f64;
        let mb = kb * 1024_f64;
        let gb = mb * 1024_f64;
        let b = bytes as f64;
        if b >= gb {
            format!("{:.2} GB", b / gb)
        } else if b >= mb {
            format!("{:.2} MB", b / mb)
        } else if b >= kb {
            format!("{:.2} KB", b / kb)
        } else {
            format!("{} B", bytes)
        }
    };

    let format_date = |timestamp: u64| -> String {
        let date = js_sys::Date::new(&((timestamp as f64) * 1000.0).into());
        let month = date.get_month() + 1;
        let day = date.get_date();
        let year = date.get_full_year() % 100;
        let mut hours = date.get_hours();
        let minutes = date.get_minutes();
        let ampm = if hours >= 12 { "PM" } else { "AM" };
        if hours > 12 { hours -= 12; }
        if hours == 0 { hours = 12; }
        format!("{}/{}/{} {}:{:02} {}", month, day, year, hours, minutes, ampm)
    };

    let handle_create_folder = move |_| {
        set_input_modal_title.set("Nhập tên thư mục mới:".to_string());
        set_input_modal_value.set("".to_string());
        set_input_modal_mode.set("folder".to_string());
        set_input_modal_open.set(true);
    };

    let handle_create_file = move |_| {
        set_input_modal_title.set("Nhập tên file mới (vd: tailieu.txt):".to_string());
        set_input_modal_value.set("".to_string());
        set_input_modal_mode.set("file".to_string());
        set_input_modal_open.set(true);
    };

    let submit_input_modal = move || {
        let name = input_modal_value.get();
        if name.trim().is_empty() { return; }
        
        let path = current_path();
        let mode = input_modal_mode.get();
        
        set_input_modal_open.set(false);
        
        if mode == "folder" {
            leptos::spawn_local(async move {
                if let Ok(_) = crate::features::file_browser::api::create_folder(&path, &name).await {
                    set_refresh_trigger.update(|n| *n += 1);
                } else {
                    let _ = web_sys::window().unwrap().alert_with_message("Lỗi tạo thư mục");
                }
            });
        } else if mode == "file" {
            let full_path = if path.is_empty() { name.clone() } else { format!("{}/{}", path, name) };
            leptos::spawn_local(async move {
                if let Ok(_) = crate::features::file_browser::api::write_text_file(&full_path, "").await {
                    set_refresh_trigger.update(|n| *n += 1);
                } else {
                    let _ = web_sys::window().unwrap().alert_with_message("Lỗi tạo file");
                }
            });
        }
    };

    let process_files_array = move |files: Vec<web_sys::File>| {
        let path = current_path();
        let total_files = files.len();
        let use_smart_rename = smart_rename_enabled.get();
        let use_file_time_enabled = use_file_time.get();
        let template_str = smart_rename_template.get();
        let effective_template = if template_str.trim().is_empty() {
            "dva facebook yyyy-MM-dd HHhMM' [type]".to_string()
        } else {
            template_str.clone()
        };
        
        if total_files == 0 { return; }
        
        let mut total_bytes = 0;
        for file in &files {
            total_bytes += file.size() as u64;
        }

        upload_state.update(|s| {
            s.is_visible = true;
            s.is_minimized = false;
            s.total_files = total_files;
            s.files_completed = 0;
            s.total_bytes = total_bytes;
            s.total_bytes_uploaded = 0;
            s.speed_bytes_per_sec = 0.0;
            s.time_remaining_sec = 0;
        });
        
        leptos::spawn_local(async move {
            let start_time = js_sys::Date::now();
            let mut uploaded_so_far = 0;
            let mut global_conflict_res: Option<ConflictResolution> = None;
            
            // --- Pre-pass for Smart Rename frequencies ---
            let mut base_name_freqs: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            let mut file_base_names: Vec<Option<String>> = Vec::with_capacity(files.len());
            
            for file in &files {
                let name = file.name();
                let mut rel_path_opt = None;
                if let Ok(cp) = js_sys::Reflect::get(file, &"customPath".into()) {
                    if let Some(cp_str) = cp.as_string() { rel_path_opt = Some(cp_str); }
                }
                if rel_path_opt.is_none() || rel_path_opt.as_ref().unwrap().is_empty() {
                    if let Ok(rel_path_val) = js_sys::Reflect::get(file, &"webkitRelativePath".into()) {
                        if let Some(rp) = rel_path_val.as_string() { rel_path_opt = Some(rp); }
                    }
                }
                let mut is_in_subfolder = false;
                if let Some(rel_path) = &rel_path_opt {
                    if !rel_path.is_empty() {
                        let parts: Vec<&str> = rel_path.split('/').collect();
                        if parts.len() > 1 { is_in_subfolder = true; }
                    }
                }
                
                if use_smart_rename && !effective_template.trim().is_empty() && !is_in_subfolder {
                    let parts: Vec<&str> = name.rsplitn(2, '.').collect();
                    let ext = if parts.len() == 2 { parts[0] } else { "" };
                    let file_type = match ext.to_lowercase().as_str() {
                        "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" => "video",
                        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => "audio",
                        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" => "image",
                        "pdf" | "doc" | "docx" | "txt" | "xlsx" | "xls" | "csv" | "ppt" | "pptx" => "document",
                        "zip" | "rar" | "7z" | "tar" | "gz" => "archive",
                        _ => "file",
                    };
                    let date = if use_file_time_enabled {
                        js_sys::Date::new(&file.last_modified().into())
                    } else {
                        js_sys::Date::new_0()
                    };
                    let year = date.get_full_year();
                    let month = date.get_month() + 1;
                    let day = date.get_date();
                    let formatted_date = format!("{:04}-{:02}-{:02}", year, month, day);
                    let hours = date.get_hours();
                    let minutes = date.get_minutes();
                    let formatted_time = format!("{:02}h{:02}'", hours, minutes);
                    let formatted_time_alt = format!("{:02}h{:02}", hours, minutes);
                    
                    let mut generated = effective_template.replace("yyyy-MM-dd", &formatted_date).replace("HHhMM'", &formatted_time).replace("HHhMM", &formatted_time_alt).replace("[type]", file_type);
                    if !ext.is_empty() && generated.ends_with(&format!(".{}", ext)) {
                        generated = generated.strip_suffix(&format!(".{}", ext)).unwrap().to_string();
                    }
                    
                    let count = base_name_freqs.entry(generated.clone()).or_insert(0);
                    *count += 1;
                    file_base_names.push(Some(generated));
                } else {
                    file_base_names.push(None);
                }
            }

            let mut rename_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

            for (i, file) in files.into_iter().enumerate() {
                let mut name = file.name();
                let mut upload_path = path.clone();
                let file_size = file.size() as f64;
                
                let mut rel_path_opt = None;
                if let Ok(cp) = js_sys::Reflect::get(&file, &"customPath".into()) {
                    if let Some(cp_str) = cp.as_string() {
                        rel_path_opt = Some(cp_str);
                    }
                }
                if rel_path_opt.is_none() || rel_path_opt.as_ref().unwrap().is_empty() {
                    if let Ok(rel_path_val) = js_sys::Reflect::get(&file, &"webkitRelativePath".into()) {
                        if let Some(rp) = rel_path_val.as_string() {
                            rel_path_opt = Some(rp);
                        }
                    }
                }
                
                if let Some(rel_path) = rel_path_opt {
                    if !rel_path.is_empty() {
                        let parts: Vec<&str> = rel_path.split('/').collect();
                        if parts.len() > 1 {
                            name = parts.last().unwrap().to_string();
                            let sub_dir = parts[..parts.len() - 1].join("/");
                            upload_path = if upload_path.is_empty() {
                                sub_dir
                            } else {
                                format!("{}/{}", upload_path, sub_dir)
                            };
                        }
                    }
                }
                
                if let Some(generated) = &file_base_names[i] {
                    let parts: Vec<&str> = name.rsplitn(2, '.').collect();
                    let ext = if parts.len() == 2 { parts[0] } else { "" };
                    
                    let freq = *base_name_freqs.get(generated).unwrap_or(&0);
                    let count = rename_counts.entry(generated.clone()).or_insert(0);
                    
                    let idx_str = if freq > 1 { format!(" ({})", count) } else { "".to_string() };
                    *count += 1;
                    
                    let final_name = if ext.is_empty() {
                        format!("{}{}", generated, idx_str)
                    } else {
                        format!("{}{}.{}", generated, idx_str, ext)
                    };

                    if name != final_name { name = final_name; }
                }

                let mut skip_this_file = false;
                if !use_smart_rename {
                    let mut file_exists = true;
                    while file_exists {
                        let exists = files_memo.with(|f| f.iter().find(|item| item.name == name).cloned());
                        if let Some(existing_file) = exists {
                            if let Some(res) = &global_conflict_res {
                                match res {
                                    ConflictResolution::Skip => { file_exists = false; skip_this_file = true; },
                                    ConflictResolution::Overwrite => { file_exists = false; },
                                    ConflictResolution::Rename => {
                                        let parts: Vec<&str> = name.rsplitn(2, '.').collect();
                                        let ext = if parts.len() == 2 { parts[0] } else { "" };
                                        let stem = if parts.len() == 2 { parts[1] } else { parts[0] };
                                        name = if ext.is_empty() { format!("{} (1)", stem) } else { format!("{} (1).{}", stem, ext) };
                                    }
                                }
                            } else {
                                let (tx, rx) = futures::channel::oneshot::channel();
                                conflict_state.set(Some(ConflictState {
                                    info: ConflictInfo {
                                        source_name: file.name(),
                                        dest_name: name.clone(),
                                        source_size: file_size as u64,
                                        dest_size: existing_file.size,
                                        source_mtime: file.last_modified() as u64,
                                        dest_mtime: existing_file.modified_at,
                                    },
                                    tx: std::rc::Rc::new(std::cell::RefCell::new(Some(tx))),
                                }));
                                
                                if let Ok((res, apply_all)) = rx.await {
                                    if apply_all {
                                        global_conflict_res = Some(res.clone());
                                    }
                                    match res {
                                        ConflictResolution::Skip => { file_exists = false; skip_this_file = true; },
                                        ConflictResolution::Overwrite => { file_exists = false; },
                                        ConflictResolution::Rename => {
                                            let parts: Vec<&str> = name.rsplitn(2, '.').collect();
                                            let ext = if parts.len() == 2 { parts[0] } else { "" };
                                            let stem = if parts.len() == 2 { parts[1] } else { parts[0] };
                                            name = if ext.is_empty() { format!("{} (1)", stem) } else { format!("{} (1).{}", stem, ext) };
                                        }
                                    }
                                } else {
                                    file_exists = false;
                                    skip_this_file = true;
                                }
                            }
                        } else {
                            file_exists = false;
                        }
                    }
                }
                
                if skip_this_file {
                    upload_state.update(|s| { s.files_completed += 1; });
                    continue;
                }

                upload_state.update(|s| {
                    s.current_file_name = name.clone();
                    s.current_file_progress = 0.0;
                });
                
                let mtime = Some(file.last_modified());
                
                let chunk_size = 2_f64 * 1024.0 * 1024.0; // 2MB
                let total_chunks = (file_size / chunk_size).ceil() as usize;
                
                if total_chunks == 0 {
                    let mut retries = 0;
                    while retries < 3 {
                        if upload_chunk(&upload_path, &name, 0, 1, 0, mtime, file.slice().unwrap()).await.is_ok() {
                            break;
                        }
                        retries += 1;
                        // Optional sleep could be added here
                    }
                } else {
                    for chunk_idx in 0..total_chunks {
                        if !upload_state.get().is_visible {
                            break; // Cancelled
                        }
                        
                        let start = (chunk_idx as f64) * chunk_size;
                        let end = ((chunk_idx as f64 + 1.0) * chunk_size).min(file_size);
                        let blob = file.slice_with_f64_and_f64(start, end).unwrap();
                        
                        let mut retries = 0;
                        let mut success = false;
                        while retries < 3 {
                            if upload_chunk(&upload_path, &name, chunk_idx, total_chunks, start as u64, mtime, blob.clone()).await.is_ok() {
                                success = true;
                                break;
                            }
                            retries += 1;
                        }
                        
                        if !success {
                            leptos::logging::error!("Failed to upload chunk {} of file {}", chunk_idx, name);
                            break; // Stop uploading this file if a chunk completely fails
                        }
                        
                        let chunk_bytes = end - start;
                        uploaded_so_far += chunk_bytes as u64;
                        
                        let elapsed = (js_sys::Date::now() - start_time) / 1000.0;
                        let speed = if elapsed > 0.0 { (uploaded_so_far as f64) / elapsed } else { 0.0 };
                        let remaining_bytes = total_bytes.saturating_sub(uploaded_so_far);
                        let remaining_time = if speed > 0.0 { (remaining_bytes as f64 / speed) as u64 } else { 0 };

                        upload_state.update(|s| {
                            s.current_file_progress = (end / file_size) * 100.0;
                            s.total_bytes_uploaded = uploaded_so_far;
                            s.speed_bytes_per_sec = speed;
                            s.time_remaining_sec = remaining_time;
                        });
                    }
                }
                
                upload_state.update(|s| {
                    s.files_completed += 1;
                });
            }
            
            upload_state.update(|s| { s.is_visible = false; });
            set_refresh_trigger.update(|n| *n += 1);
        });
    };
    
    let handle_file_upload = move |ev: leptos::ev::Event| {
        let input: HtmlInputElement = event_target(&ev);
        let file_list = input.files().unwrap();
        let mut files = Vec::new();
        
        for i in 0..file_list.length() {
            files.push(file_list.item(i).unwrap());
        }
        
        if files.is_empty() { return; }
        
        process_files_array(files);
        
        input.set_value("");
    };

    let (is_dragging, set_is_dragging) = create_signal(false);

    let handle_dragover = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_is_dragging.set(true);
    };

    let handle_dragleave = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        let target = ev.target().unwrap().unchecked_into::<web_sys::Element>();
        // Only set false if leaving the main container
        if target.class_list().contains("file-browser-container") || target.class_list().contains("drag-overlay") {
            set_is_dragging.set(false);
        }
    };

    let handle_drop = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_is_dragging.set(false);
        
        leptos::spawn_local(async move {
            if let Ok(js_array) = process_drop_event(&ev).await {
                let mut files = Vec::new();
                for i in 0..js_array.length() {
                    let val = js_array.get(i);
                    if let Ok(f) = val.dyn_into::<web_sys::File>() {
                        files.push(f);
                    }
                }
                process_files_array(files);
            }
        });
    };
    
    let handle_rename = move |_| {
        let files = selected_files.get();
        if files.is_empty() { return; }
        
        if files.len() == 1 {
            let file_name = files.into_iter().next().unwrap();
            set_rename_file_name.set(file_name.clone());
            set_rename_new_name.set(file_name);
            set_rename_dialog_open.set(true);
        } else {
            let mut sorted_files: Vec<String> = files.into_iter().collect();
            sorted_files.sort();
            
            let first_file = &sorted_files[0];
            let base_name = if let Some(idx) = first_file.rfind('.') {
                first_file[..idx].to_string()
            } else {
                first_file.clone()
            };
            
            set_bulk_common_name.set(base_name);
            set_bulk_start_index.set("01".to_string());
            set_bulk_extension.set("Mặc định".to_string());
            set_bulk_rename_dialog_open.set(true);
        }
    };

    fn get_next_index(current: &str) -> String {
        if let Ok(num) = current.parse::<u32>() {
            let width = current.chars().count();
            let next_num = num + 1;
            if current.starts_with('0') {
                return format!("{:0>width$}", next_num, width=width);
            } else {
                return next_num.to_string();
            }
        }
        
        if current.len() == 1 {
            let c = current.chars().next().unwrap();
            if c >= 'a' && c < 'z' {
                return (((c as u8) + 1) as char).to_string();
            } else if c >= 'A' && c < 'Z' {
                return (((c as u8) + 1) as char).to_string();
            }
        }
        
        current.to_string()
    }

    let submit_bulk_rename = move || {
        if is_loading_bulk_rename.get() { return; }
        
        let common_name = bulk_common_name.get();
        let start_idx = bulk_start_index.get();
        let extension_input = bulk_extension.get();
        let files = selected_files.get();
        
        if files.is_empty() { return; }
        
        let mut sorted_files: Vec<String> = files.into_iter().collect();
        sorted_files.sort();
        
        set_is_loading_bulk_rename.set(true);
        
        spawn_local(async move {
            let mut current_idx = start_idx.clone();
            
            for file_name in sorted_files {
                let current_dir = current_path();
                let full_path = if current_dir.is_empty() { file_name.clone() } else { format!("{}/{}", current_dir, file_name) };
                
                let ext = if extension_input.is_empty() || extension_input == "Mặc định" {
                    if let Some(idx) = file_name.rfind('.') {
                        file_name[idx..].to_string()
                    } else {
                        "".to_string()
                    }
                } else {
                    let mut ext_str = extension_input.clone();
                    if !ext_str.starts_with('.') {
                        ext_str.insert(0, '.');
                    }
                    ext_str
                };
                
                let new_name = format!("{}{}{}", common_name, current_idx, ext);
                let _ = crate::features::file_browser::api::rename_file(full_path, new_name).await;
                
                current_idx = get_next_index(&current_idx);
            }
            
            set_is_loading_bulk_rename.set(false);
            set_bulk_rename_dialog_open.set(false);
            set_selected_files.set(HashSet::new());
            set_refresh_trigger.update(|n| *n += 1);
        });
    };

    let submit_rename = move || {
        let old_name = rename_file_name.get();
        let new_name = rename_new_name.get();
        
        if new_name.trim().is_empty() || new_name == old_name {
            set_rename_dialog_open.set(false);
            return;
        }
        
        let current = current_path();
        let full_path = if current.is_empty() { old_name.clone() } else { format!("{}/{}", current, old_name) };
        
        spawn_local(async move {
            if crate::features::file_browser::api::rename_file(full_path, new_name).await.is_ok() {
                set_selected_files.set(HashSet::new());
                set_refresh_trigger.update(|n| *n += 1);
            }
            set_rename_dialog_open.set(false);
        });
    };

    let handle_cut = move |_| {
        let current = current_path();
        let mut full_paths = HashSet::new();
        for name in selected_files.get() {
            let full_path = if current.is_empty() { name } else { format!("{}/{}", current, name) };
            full_paths.insert(full_path);
        }
        set_clipboard_action.set(Some("cut".to_string()));
        set_clipboard_files.set(full_paths);
        set_selected_files.set(HashSet::new());
    };

    let handle_copy = move |_| {
        let current = current_path();
        let mut full_paths = HashSet::new();
        for name in selected_files.get() {
            let full_path = if current.is_empty() { name } else { format!("{}/{}", current, name) };
            full_paths.insert(full_path);
        }
        set_clipboard_action.set(Some("copy".to_string()));
        set_clipboard_files.set(full_paths);
        set_selected_files.set(HashSet::new());
    };

    let handle_delete = move |_| {
        let current = current_path();
        let mut full_paths = Vec::new();
        for name in selected_files.get() {
            let full_path = if current.is_empty() { name.clone() } else { format!("{}/{}", current, name) };
            full_paths.push(full_path);
        }
        
        if full_paths.is_empty() { return; }
        
        set_delete_paths.set(full_paths);
        set_move_to_trash.set(true);
        set_show_delete_modal.set(true);
    };

    let confirm_delete = move |_| {
        let paths = delete_paths.get();
        let is_permanent = !move_to_trash.get();
        
        set_show_delete_modal.set(false);
        
        spawn_local(async move {
            if crate::features::file_browser::api::delete_files(paths, is_permanent).await.is_ok() {
                set_selected_files.set(HashSet::new());
                set_refresh_trigger.update(|n| *n += 1);
            }
        });
    };

    let handle_paste = move |_| {
        let action = clipboard_action.get().unwrap_or_default();
        let files = clipboard_files.get();
        if files.is_empty() { return; }
        
        let dest_path = current_path();
        
        let use_file_time_val = use_file_time.get();
        let template = if smart_rename_enabled.get() {
            let t = smart_rename_template.get();
            if t.trim().is_empty() {
                Some("dva facebook yyyy-MM-dd HHhMM' [type]".to_string())
            } else {
                Some(t)
            }
        } else {
            None
        };
        
        spawn_local(async move {
            let mut paths: Vec<String> = files.into_iter().collect();
            let mut overwrite_paths = Vec::new();
            
            if !smart_rename_enabled.get() {
                let mut global_conflict_res: Option<ConflictResolution> = None;
                let mut final_paths = Vec::new();
                
                for path in paths {
                    let file_name = path.split('/').last().unwrap_or("").to_string();
                    let mut file_exists = true;
                    let mut current_name = file_name.clone();
                    let mut skip_this_file = false;
                    
                    while file_exists {
                        let exists = files_memo.with(|f| f.iter().find(|item| item.name == current_name).cloned());
                        if let Some(existing_file) = exists {
                            if let Some(res) = &global_conflict_res {
                                match res {
                                    ConflictResolution::Skip => { file_exists = false; skip_this_file = true; },
                                    ConflictResolution::Overwrite => { file_exists = false; overwrite_paths.push(path.clone()); },
                                    ConflictResolution::Rename => {
                                        let parts: Vec<&str> = current_name.rsplitn(2, '.').collect();
                                        let ext = if parts.len() == 2 { parts[0] } else { "" };
                                        let stem = if parts.len() == 2 { parts[1] } else { parts[0] };
                                        current_name = if ext.is_empty() { format!("{} (1)", stem) } else { format!("{} (1).{}", stem, ext) };
                                    }
                                }
                            } else {
                                let (tx, rx) = futures::channel::oneshot::channel();
                                conflict_state.set(Some(ConflictState {
                                    info: ConflictInfo {
                                        source_name: file_name.clone(),
                                        dest_name: current_name.clone(),
                                        source_size: 0,
                                        dest_size: existing_file.size,
                                        source_mtime: 0,
                                        dest_mtime: existing_file.modified_at,
                                    },
                                    tx: std::rc::Rc::new(std::cell::RefCell::new(Some(tx))),
                                }));
                                
                                if let Ok((res, apply_all)) = rx.await {
                                    if apply_all {
                                        global_conflict_res = Some(res.clone());
                                    }
                                    match res {
                                        ConflictResolution::Skip => { file_exists = false; skip_this_file = true; },
                                        ConflictResolution::Overwrite => { file_exists = false; overwrite_paths.push(path.clone()); },
                                        ConflictResolution::Rename => {
                                            let parts: Vec<&str> = current_name.rsplitn(2, '.').collect();
                                            let ext = if parts.len() == 2 { parts[0] } else { "" };
                                            let stem = if parts.len() == 2 { parts[1] } else { parts[0] };
                                            current_name = if ext.is_empty() { format!("{} (1)", stem) } else { format!("{} (1).{}", stem, ext) };
                                        }
                                    }
                                } else {
                                    file_exists = false;
                                    skip_this_file = true;
                                }
                            }
                        } else {
                            file_exists = false;
                        }
                    }
                    
                    if !skip_this_file {
                        final_paths.push(path);
                    }
                }
                
                paths = final_paths;
            }
            
            if paths.is_empty() {
                set_clipboard_files.set(HashSet::new());
                set_clipboard_action.set(None);
                return;
            }
            
            let res = if action == "cut" {
                crate::features::file_browser::api::move_files(paths, dest_path, template.clone(), Some(use_file_time_val), Some(overwrite_paths)).await
            } else if action == "copy" {
                crate::features::file_browser::api::copy_files(paths, dest_path, template.clone(), Some(use_file_time_val), Some(overwrite_paths)).await
            } else {
                Ok(())
            };
            
            if res.is_ok() {
                set_clipboard_files.set(HashSet::new());
                set_clipboard_action.set(None);
                set_refresh_trigger.update(|n| *n += 1);
            }
        });
    };

    let text_get_ext = |name: &str| -> String {
        let lower = name.to_lowercase();
        if let Some(idx) = lower.rfind('.') {
            lower[idx..].to_string()
        } else {
            "other".to_string()
        }
    };

    let text_go = move |dir: i32| {
        if let Some(path) = text_editor_file.get() {
            let files = files_memo.get();
            let name_only = path.split('/').last().unwrap_or("").to_string();
            let ext = text_get_ext(&name_only);
            if let Some(idx) = files.iter().position(|f| f.name == name_only) {
                let mut target_idx = None;
                if dir == 1 {
                    for i in (idx + 1)..files.len() {
                        if text_get_ext(&files[i].name) == ext {
                            target_idx = Some(i);
                            break;
                        }
                    }
                } else {
                    for i in (0..idx).rev() {
                        if text_get_ext(&files[i].name) == ext {
                            target_idx = Some(i);
                            break;
                        }
                    }
                }
                
                if let Some(i) = target_idx {
                    let path_clone = files[i].path.clone();
                    update_text_editor_file(Some(path_clone));
                }
            }
        }
    };
    
    let has_text_prev = move || {
        if let Some(path) = text_editor_file.get() {
            let files = files_memo.get();
            let name_only = path.split('/').last().unwrap_or("").to_string();
            let ext = text_get_ext(&name_only);
            if let Some(idx) = files.iter().position(|f| f.name == name_only) {
                for i in (0..idx).rev() {
                    if text_get_ext(&files[i].name) == ext {
                        return true;
                    }
                }
            }
        }
        false
    };

    let has_text_next = move || {
        if let Some(path) = text_editor_file.get() {
            let files = files_memo.get();
            let name_only = path.split('/').last().unwrap_or("").to_string();
            let ext = text_get_ext(&name_only);
            if let Some(idx) = files.iter().position(|f| f.name == name_only) {
                for i in (idx + 1)..files.len() {
                    if text_get_ext(&files[i].name) == ext {
                        return true;
                    }
                }
            }
        }
        false
    };

    view! {
        <div class="file-browser-container"
             on:dragover=handle_dragover
             on:dragleave=handle_dragleave
             on:drop=handle_drop
             on:click=move |_| {
                 set_show_settings.set(false);
                 set_show_more_menu.set(false);
                 set_show_new_menu.set(false);
             }>
            
            <Show when=move || is_dragging.get() fallback=|| ()>
                <div class="drag-overlay">
                    <div class="drag-content">
                        <svg viewBox="0 0 24 24" width="64" height="64" stroke="currentColor" stroke-width="2" fill="none">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                            <polyline points="17 8 12 3 7 8"></polyline>
                            <line x1="12" y1="3" x2="12" y2="15"></line>
                        </svg>
                        <h2>"Thả file hoặc thư mục vào đây để tải lên"</h2>
                    </div>
                </div>
            </Show>

            <div class="sticky-top-area">
                <div class="browser-header">
                    <div class="breadcrumb">
                        <span class="path-part" style="cursor: pointer; font-weight: bold; padding: 4px;" on:click=move |_| navigate.with_value(|n| n("/", Default::default()))>"Home"</span>
                    {move || {
                        let path = current_path();
                        if !path.is_empty() {
                            let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
                            let mut views = Vec::new();
                            let mut accum_path = String::new();
                            for part in parts.iter() {
                                if !accum_path.is_empty() {
                                    accum_path.push('/');
                                }
                                accum_path.push_str(part);
                                let capture_path = accum_path.clone();
                                views.push(view! {
                                    <span class="separator">" > "</span>
                                    <span class="path-part" style="cursor: pointer; padding: 4px; border-radius: 4px;" on:mouseover=|_| {} on:click=move |_| navigate.with_value(|n| n(&format!("/{}", capture_path), Default::default()))>{part.to_string()}</span>
                                });
                            }
                            views.collect_view()
                        } else {
                            view! { <span></span> }.into_view()
                        }
                    }}
                </div>
                <div style="display: flex; flex-direction: column; align-items: flex-end; gap: 4px;">
                    <div class="disk-card-wrapper">
                        <Transition fallback=|| view! { <span>"..."</span> }>
                            {move || {
                                disk_resource.get().map(|disk| {
                                    let percent = (disk.used_space as f64 / disk.total_space as f64) * 100.0;
                                    let is_danger = percent >= 90.0;
                                    let class_name = if is_danger { "disk-card danger" } else { "disk-card normal" };
                                    
                                    let r = 20.0;
                                    let circ = 2.0 * std::f64::consts::PI * r;
                                    let offset = circ - (percent / 100.0) * circ;
                                    
                                    let format_size_local = |bytes: u64| -> String {
                                        let kb = 1024_f64;
                                        let mb = kb * 1024_f64;
                                        let gb = mb * 1024_f64;
                                        let b = bytes as f64;
                                        if b >= gb { format!("{:.2} GB", b / gb).replace('.', ",") }
                                        else if b >= mb { format!("{:.2} MB", b / mb).replace('.', ",") }
                                        else if b >= kb { format!("{:.2} KB", b / kb).replace('.', ",") }
                                        else { format!("{} B", bytes) }
                                    };
                                    
                                    view! {
                                        <div class=class_name>
                                            <div class="progress-ring">
                                                <svg viewBox="0 0 48 48">
                                                    <circle class="bg" cx="24" cy="24" r="20"></circle>
                                                    <circle class="fg" cx="24" cy="24" r="20" style=format!("stroke-dasharray: {}; stroke-dashoffset: {};", circ, offset)></circle>
                                                </svg>
                                                <span class="percent">{format!("{:.0}%", percent)}</span>
                                            </div>
                                            <div class="disk-info">
                                                <span class="disk-title">"Internal Storage"</span>
                                                <span class="disk-usage">{format!("{} / {}", format_size_local(disk.used_space), format_size_local(disk.total_space))}</span>
                                            </div>
                                        </div>
                                    }
                                })
                            }}
                        </Transition>
                    </div>
                </div>
            </div>

            <div class="toolbar">
                <div style="position: relative;">
                    <button class="btn primary" style="gap: 8px; padding: 10px 20px; font-size: 16px; border-radius: 20px; box-shadow: 0 4px 15px rgba(59,130,246,0.3);" on:click=move |ev| { ev.stop_propagation(); set_show_new_menu.update(|s| *s = !*s); }>
                        <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
                        "Tạo mới"
                    </button>
                    
                    <Show when=move || show_new_menu.get() fallback=|| ()>
                        <div class="new-menu-dropdown" on:click=move |ev| ev.stop_propagation()>
                            <div class="menu-item" on:click=move |ev| { set_show_new_menu.set(false); handle_create_folder(ev); } style="display: flex; align-items: center; gap: 10px; color: var(--text-main); padding: 10px; cursor: pointer; border-radius: 8px;">
                                <svg viewBox="0 0 24 24" width="18" height="18" stroke="#3b82f6" stroke-width="2" fill="none"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                                "Tạo thư mục mới"
                            </div>
                            <div class="menu-item" on:click=move |ev| { set_show_new_menu.set(false); handle_create_file(ev); } style="display: flex; align-items: center; gap: 10px; color: var(--text-main); padding: 10px; cursor: pointer; border-radius: 8px;">
                                <svg viewBox="0 0 24 24" width="18" height="18" stroke="#10b981" stroke-width="2" fill="none"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg>
                                "Tạo file mới"
                            </div>
                            <div style="height: 1px; background: var(--border-color, #e2e8f0); margin: 4px 0;"></div>
                            
                            <div class="upload-btn-wrapper" style="width: 100%; border: none; display: flex; overflow: hidden; position: relative;">
                                <div class="menu-item" style="display: flex; align-items: center; gap: 10px; color: var(--text-main); width: 100%; padding: 10px; cursor: pointer; border-radius: 8px;">
                                    <svg viewBox="0 0 24 24" width="18" height="18" stroke="#64748b" stroke-width="2" fill="none"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg>
                                    "Tải File lên"
                                </div>
                                <input type="file" multiple on:change=move |ev| { set_show_new_menu.set(false); handle_file_upload.clone()(ev); } style="position: absolute; left: 0; top: 0; opacity: 0; cursor: pointer; width: 100%; height: 100%; z-index: 10;" />
                            </div>
                            
                            <div class="upload-btn-wrapper" style="width: 100%; border: none; display: flex; overflow: hidden; position: relative;">
                                <div class="menu-item" style="display: flex; align-items: center; gap: 10px; color: var(--text-main); width: 100%; padding: 10px; cursor: pointer; border-radius: 8px;">
                                    <svg viewBox="0 0 24 24" width="18" height="18" stroke="#64748b" stroke-width="2" fill="none"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg>
                                    "Tải Thư mục lên"
                                </div>
                                <input type="file" attr:webkitdirectory="true" multiple on:change=move |ev| { set_show_new_menu.set(false); handle_file_upload(ev); } style="position: absolute; left: 0; top: 0; opacity: 0; cursor: pointer; width: 100%; height: 100%; z-index: 10;" />
                            </div>
                        </div>
                    </Show>
                </div>
                
                <div style="margin-left: auto; display: flex; align-items: center; gap: 15px;">
                    <div class="selection-toolbar" style=move || if selected_files.with(|s| s.is_empty()) { "display: none;" } else { "display: flex;" }>
                        <span class="select-count">{move || format!("{} mục", selected_files.with(|s| s.len()))}</span>
                        <button on:click=move |_| {
                            if let Some(files) = files_resource.get() {
                                let mut all = HashSet::new();
                                for f in files {
                                    all.insert(f.name.clone());
                                }
                                set_selected_files.set(all);
                            }
                        }>"Chọn tất cả"</button>
                        <button class="cancel-btn" on:click=move |_| set_selected_files.set(HashSet::new())>"Hủy"</button>
                    </div>
                    
                    <div class="settings-btn" style="position: relative; cursor: pointer; color: var(--text-muted); display: flex; align-items: center;" on:click=move |ev| { ev.stop_propagation(); set_show_settings.update(|s| *s = !*s); }>
                        <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="1.5"></circle>
                            <circle cx="12" cy="5" r="1.5"></circle>
                            <circle cx="12" cy="19" r="1.5"></circle>
                        </svg>
                        
                        <Show when=move || show_settings.get() fallback=|| ()>
                            <div class="settings-dropdown" on:click=move |ev| ev.stop_propagation()>
                                
                                <div class="settings-group-box">
                                    <div class="sort-header">
                                        <span class="line"></span>
                                        <span class="text">"Công cụ"</span>
                                        <span class="line"></span>
                                    </div>
                                    <div class="tools-grid">
                                        <button class="tool-item" on:click=move |_| { set_show_settings.set(false); set_show_trash_modal.set(true); }>
                                            <div class="tool-icon animated-gradient">
                                                <svg width="24" height="24" viewBox="0 0 24 24" fill="white" xmlns="http://www.w3.org/2000/svg">
                                                    <path fill-rule="evenodd" d="M16.5 4.478v.227a48.816 48.816 0 0 1 3.878.512.75.75 0 1 1-.256 1.478l-.209-.035-1.005 13.07a3 3 0 0 1-2.991 2.77H8.084a3 3 0 0 1-2.991-2.77L4.087 6.66l-.209.035a.75.75 0 0 1-.256-1.478A48.567 48.567 0 0 1 7.5 4.705v-.227c0-1.564 1.213-2.9 2.816-2.951a52.662 52.662 0 0 1 3.369 0c1.603.051 2.815 1.387 2.815 2.951zm-6.136-1.452a51.196 51.196 0 0 1 3.273 0C14.39 3.05 15 3.684 15 4.478v.113a49.488 49.488 0 0 0-6 0v-.113c0-.794.609-1.428 1.364-1.452zm-.355 5.945a.75.75 0 1 0-1.5.058l.347 9a.75.75 0 1 0 1.499-.058l-.346-9zm5.442.058a.75.75 0 1 0-1.498-.058l-.347 9a.75.75 0 0 0 1.5.058l.345-9z" clip-rule="evenodd" />
                                                </svg>
                                            </div>
                                            <span class="tool-name">"Thùng rác"</span>
                                        </button>
                                        <button class="tool-item" on:click=toggle_theme>
                                            <div class="tool-icon" style=move || if theme.get() == "dark" { "background: linear-gradient(135deg, #1e293b, #0f172a);" } else { "background: linear-gradient(135deg, #fcd34d, #f59e0b);" }>
                                                <Show when=move || theme.get() == "dark" fallback=|| view! {
                                                    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"></circle><line x1="12" y1="1" x2="12" y2="3"></line><line x1="12" y1="21" x2="12" y2="23"></line><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line><line x1="1" y1="12" x2="3" y2="12"></line><line x1="21" y1="12" x2="23" y2="12"></line><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line></svg>
                                                }>
                                                    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path></svg>
                                                </Show>
                                            </div>
                                            <span class="tool-name">{move || if theme.get() == "dark" { "Giao diện Tối" } else { "Giao diện Sáng" }}</span>
                                        </button>
                                    </div>
                                </div>

                                <div class="settings-group-box">
                                    <div class="sort-header">
                                        <span class="line"></span>
                                        <span class="text">"Tính năng nâng cao"</span>
                                        <span class="line"></span>
                                    </div>
                                    <label style="display: flex; align-items: center; gap: 8px; margin-bottom: 8px; font-size: 14px; color: var(--text-main);">
                                        <input type="checkbox" checked=smart_rename_enabled prop:checked=smart_rename_enabled on:input=move |ev| update_rename_enabled(event_target_checked(&ev)) />
                                        "Bật đổi tên thông minh khi Upload/Copy/Cut"
                                    </label>
                                    <label style="display: flex; align-items: center; gap: 8px; margin-bottom: 8px; font-size: 14px; color: var(--text-main);">
                                        <input type="checkbox" checked=use_file_time prop:checked=use_file_time on:input=move |ev| update_use_file_time(event_target_checked(&ev)) />
                                        "Áp dụng ngày/giờ của file (tắt sẽ dùng giờ hệ thống)"
                                    </label>
                                    <input class="settings-input" type="text" placeholder="dva facebook yyyy-MM-dd HHhMM' [type]" prop:value=smart_rename_template on:input=move |ev| update_rename_template(event_target_value(&ev)) />
                                </div>

                                <div class="settings-group-box sort-section">
                                    <div class="sort-header">
                                        <span class="line"></span>
                                        <span class="text">"Sắp xếp"</span>
                                        <span class="line"></span>
                                    </div>
                                    <div class="sort-grid">
                                        <div class="sort-col">
                                            <div class="col-title">"Tên"</div>
                                            <button class=move || if sort_by.get() == "name" && !sort_desc.get() { "sort-btn active" } else { "sort-btn" } on:click=move |_| update_sort("name", false)>
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><text x="4" y="11" font-size="10" font-weight="bold" fill="currentColor" stroke="none">Z</text><text x="4" y="21" font-size="10" font-weight="bold" fill="currentColor" stroke="none">A</text><path d="M16 20V4m0 0l-4 4m4 -4l4 4" stroke-width="2"></path></svg>
                                            </button>
                                            <button class=move || if sort_by.get() == "name" && sort_desc.get() { "sort-btn active" } else { "sort-btn" } on:click=move |_| update_sort("name", true)>
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><text x="4" y="11" font-size="10" font-weight="bold" fill="currentColor" stroke="none">Z</text><text x="4" y="21" font-size="10" font-weight="bold" fill="currentColor" stroke="none">A</text><path d="M16 4v16m0 0l-4 -4m4 4l4 -4" stroke-width="2"></path></svg>
                                            </button>
                                        </div>
                                        <div class="sort-col">
                                            <div class="col-title">"Kiểu"</div>
                                            <button class=move || if sort_by.get() == "type" && !sort_desc.get() { "sort-btn active" } else { "sort-btn" } on:click=move |_| update_sort("type", false)>
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15.59 13.41l-6.17 6.17a2 2 0 0 1-2.83 0L2 15V5h10l6.59 6.59a2 2 0 0 1 0 2.82z"></path><line x1="6" y1="9" x2="6.01" y2="9"></line><path d="M20 18V6m0 0l-3 3m3 -3l3 3"></path></svg>
                                            </button>
                                            <button class=move || if sort_by.get() == "type" && sort_desc.get() { "sort-btn active" } else { "sort-btn" } on:click=move |_| update_sort("type", true)>
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15.59 13.41l-6.17 6.17a2 2 0 0 1-2.83 0L2 15V5h10l6.59 6.59a2 2 0 0 1 0 2.82z"></path><line x1="6" y1="9" x2="6.01" y2="9"></line><path d="M20 6v12m0 0l-3 -3m3 3l3 -3"></path></svg>
                                            </button>
                                        </div>
                                        <div class="sort-col">
                                            <div class="col-title">"Kích thước"</div>
                                            <button class=move || if sort_by.get() == "size" && !sort_desc.get() { "sort-btn active" } else { "sort-btn" } on:click=move |_| update_sort("size", false)>
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="9" cy="12" r="7"></circle><path d="M9 12V5a7 7 0 0 1 7 7z"></path><path d="M20 18V6m0 0l-3 3m3 -3l3 3"></path></svg>
                                            </button>
                                            <button class=move || if sort_by.get() == "size" && sort_desc.get() { "sort-btn active" } else { "sort-btn" } on:click=move |_| update_sort("size", true)>
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="9" cy="12" r="7"></circle><path d="M9 12V5a7 7 0 0 1 7 7z"></path><path d="M20 6v12m0 0l-3 -3m3 3l3 -3"></path></svg>
                                            </button>
                                        </div>
                                        <div class="sort-col">
                                            <div class="col-title">"Ngày sửa"</div>
                                            <button class=move || if sort_by.get() == "time" && !sort_desc.get() { "sort-btn active" } else { "sort-btn" } on:click=move |_| update_sort("time", false)>
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="9" cy="12" r="7"></circle><polyline points="9 9 9 12 11 14"></polyline><path d="M20 18V6m0 0l-3 3m3 -3l3 3"></path></svg>
                                            </button>
                                            <button class=move || if sort_by.get() == "time" && sort_desc.get() { "sort-btn active" } else { "sort-btn" } on:click=move |_| update_sort("time", true)>
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="9" cy="12" r="7"></circle><polyline points="9 9 9 12 11 14"></polyline><path d="M20 6v12m0 0l-3 -3m3 3l3 -3"></path></svg>
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
            </div>

            <div class="file-list">
                <Transition fallback=|| view! { <div class="loader">"Đang tải..."</div> }>
                    {move || {
                        files_resource.get().map(|_| {
                            let files = files_memo.get();
                            if files.is_empty() {
                                view! { <div class="empty-state">"Thư mục trống"</div> }.into_view()
                            } else {
                                files.into_iter().enumerate().map(|(index, file)| {
                                    let is_dir = file.is_dir;
                                    let file_path = file.path.clone();
                                    let file_name = file.name.clone();
                                    let file_name_for_row_click = file_name.clone();
                                    let file_name_for_class = file_name.clone();
                                    let file_name_for_dot_click = file_name.clone();
                                    let file_name_for_cut = file_name.clone();
                                    let fade_class = move || {
                                        let current = current_path();
                                        let full_path = if current.is_empty() { file_name_for_cut.clone() } else { format!("{}/{}", current, file_name_for_cut) };
                                        
                                        let mut class = String::new();
                                        if clipboard_action.get().as_deref() == Some("cut") && clipboard_files.get().contains(&full_path) {
                                            class.push_str(" cut-fade");
                                        } else if file_name_for_cut.starts_with('.') {
                                            class.push_str(" hidden-item-fade");
                                        }
                                        
                                        if selected_files.with(|s| s.contains(&file_name_for_cut)) {
                                            class.push_str(" selected-row");
                                        }
                                        class
                                    };
                                    
                                    view! {
                                        <div class=move || format!("file-item-row{}", fade_class()) 
                                             on:click=move |ev: leptos::ev::MouseEvent| {
                                                 // Tránh navigate nếu click vào cục action dot
                                                 let target = ev.target().unwrap().unchecked_into::<web_sys::Element>();
                                                 if target.closest(".file-action-dot").unwrap().is_some() {
                                                     return;
                                                 }
                                                 
                                                 if !selected_files.with(|s| s.is_empty()) {
                                                     let is_selected = selected_files.with(|s| s.contains(&file_name_for_row_click));
                                                     set_selected_files.update(|s| {
                                                         if is_selected {
                                                             s.remove(&file_name_for_row_click);
                                                         } else {
                                                             s.insert(file_name_for_row_click.clone());
                                                         }
                                                     });
                                                     return;
                                                 }
                                                 
                                                 if is_dir {
                                                     navigate.with_value(|n| n(&format!("/{}", file_path), Default::default()));
                                                 } else {
                                                     let lower = file_name_for_row_click.to_lowercase();
                                                     let is_media = lower.ends_with(".mp4") || lower.ends_with(".mkv") || lower.ends_with(".webm") || lower.ends_with(".mpd") || lower.ends_with(".avi") || lower.ends_with(".mov") || lower.ends_with(".flv") || lower.ends_with(".wmv")
                                                         || lower.ends_with(".jpg") || lower.ends_with(".png") || lower.ends_with(".jpeg") || lower.ends_with(".gif") || lower.ends_with(".webp") || lower.ends_with(".svg") || lower.ends_with(".bmp") || lower.ends_with(".ico")
                                                         || lower.ends_with(".mp3") || lower.ends_with(".wav") || lower.ends_with(".flac") || lower.ends_with(".aac");
                                                     
                                                     let is_text = lower.ends_with(".txt") || lower.ends_with(".md") || lower.ends_with(".rs") 
                                                         || lower.ends_with(".js") || lower.ends_with(".css") || lower.ends_with(".html") 
                                                         || lower.ends_with(".json") || lower.ends_with(".env") || lower.ends_with(".scss")
                                                         || lower.ends_with(".yaml") || lower.ends_with(".toml") || lower.ends_with(".xml")
                                                         || lower.ends_with(".vskip") || lower.ends_with(".srt") || lower.ends_with(".vtt") 
                                                         || lower.ends_with(".ass") || lower.ends_with(".ssa") || lower.ends_with(".sub") || lower.ends_with(".vsub");

                                                     if is_media {
                                                         active_media_index.set(Some(index));
                                                     } else if is_text {
                                                          let path_clone = file_path.clone();
                                                          update_text_editor_file(Some(path_clone));
                                                     } else {
                                                          let url = format!("/api/v1/files/download?path={}", js_sys::encode_uri_component(&file_path));
                                                          let _ = web_sys::window().unwrap().open_with_url_and_target(&url, "_blank");
                                                     }
                                                 }
                                             }>
                                            <div class="file-icon-large">
                                                {if let Some(thumb) = &file.thumbnail {
                                                    let p = file.path.clone();
                                                    view! { <img src=thumb.clone() alt="thumb" on:error=move |_| {
                                                        let p_clone = p.clone();
                                                        spawn_local(async move {
                                                            let _ = regen_single_thumbnail(&p_clone).await;
                                                        });
                                                    } /> }.into_view()
                                                } else if is_dir {
                                                    view! { 
                                                        <svg viewBox="0 0 24 24" fill="#6ba4ff" stroke="none">
                                                            <path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                                                        </svg>
                                                     }.into_view()
                                                } else {
                                                    view! { 
                                                        <svg viewBox="0 0 24 24" fill="#a0aab5" stroke="none">
                                                            <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
                                                        </svg>
                                                     }.into_view()
                                                }}
                                            </div>
                                            <div class="file-info-col file-name-col">
                                                <div class="file-name">{file.name.clone()}</div>
                                                <div class="file-size">
                                                    {if is_dir {
                                                        format!("{} mục", file.children_count.unwrap_or(0))
                                                    } else {
                                                        format_size(file.size)
                                                    }}
                                                </div>
                                            </div>
                                            <div class="file-info-col file-date-col">
                                                <div class="date-text">{format_date(file.modified_at)}</div>
                                            </div>
                                            <div class=move || {
                                                let mut base = "file-action-dot".to_string();
                                                if selected_files.with(|s| s.contains(&file_name_for_class)) {
                                                    base.push_str(" selected");
                                                }
                                                base
                                            } on:click=move |_| {
                                                let name = file_name_for_dot_click.clone();
                                                set_selected_files.update(|s| {
                                                    if s.contains(&name) {
                                                        s.remove(&name);
                                                    } else {
                                                        s.insert(name);
                                                    }
                                                });
                                            }>
                                                <div class="dot"></div>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()
                            }
                        })
                    }}
                </Transition>
            </div>
            
            <div class=move || if selected_files.with(|s| s.is_empty()) && clipboard_files.with(|c| c.is_empty()) { "bottom-action-toolbar" } else { "bottom-action-toolbar visible" }>
                <Show when=move || !selected_files.with(|s| s.is_empty()) fallback=|| ()>
                    <button class="action-btn" on:click=handle_copy>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                        <span>"Copy"</span>
                    </button>
                    <button class="action-btn" on:click=handle_cut>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="6" cy="6" r="3"></circle><circle cx="6" cy="18" r="3"></circle><line x1="20" y1="4" x2="8.12" y2="15.88"></line><line x1="14.47" y1="14.48" x2="20" y2="20"></line><line x1="8.12" y1="8.12" x2="12" y2="12"></line></svg>
                        <span>"Cut"</span>
                    </button>
                    <button class="action-btn danger" on:click=handle_delete>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                        <span>"Xóa"</span>
                    </button>
                    <button class="action-btn" on:click=handle_rename>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>
                        <span>"Sửa tên"</span>
                    </button>

                    <div style="position: relative;">
                        <button class="action-btn" on:click=move |ev| { ev.stop_propagation(); set_show_more_menu.update(|s| *s = !*s); }>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="1"></circle><circle cx="19" cy="12" r="1"></circle><circle cx="5" cy="12" r="1"></circle></svg>
                            <span>"Thêm"</span>
                        </button>
                        <Show when=move || show_more_menu.get() fallback=|| ()>
                            <div class="more-menu-dropdown" on:click=move |ev| ev.stop_propagation()>
                                <div class="menu-item" on:click=move |_| {
                                    set_show_more_menu.set(false);
                                    let selected = selected_files.get();
                                    if selected.is_empty() { return; }
                                    
                                    if selected.len() == 1 {
                                        let item = selected.iter().next().unwrap().clone();
                                        let cpath = current_path();
                                        let full_path = if cpath.is_empty() { item.clone() } else { format!("{}/{}", cpath, item) };
                                        
                                        let mut thumb = None;
                                        let item_clone_for_thumb = item.clone();
                                        files_memo.with(|f| {
                                            if let Some(entry) = f.iter().find(|e| e.name == item_clone_for_thumb) {
                                                thumb = entry.thumbnail.clone();
                                            }
                                        });
                                        set_properties_thumb_data.set(thumb);
                                        
                                        spawn_local(async move {
                                            if let Ok(props) = crate::features::file_browser::api::get_file_properties(&full_path).await {
                                                set_properties_modal_data.set(Some(props));
                                                set_checksum_data.set(None);
                                            }
                                        });
                                    } else {
                                        let mut paths = Vec::new();
                                        let cpath = current_path();
                                        for item in selected {
                                            paths.push(if cpath.is_empty() { item.clone() } else { format!("{}/{}", cpath, item) });
                                        }
                                        set_properties_thumb_data.set(None);
                                        spawn_local(async move {
                                            if let Ok(props) = crate::features::file_browser::api::get_multi_file_properties(paths).await {
                                                set_properties_modal_data.set(Some(props));
                                                set_checksum_data.set(None);
                                            }
                                        });
                                    }
                                }>
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg>
                                    <span>"Thông tin"</span>
                                </div>
                                <div class="menu-item" on:click=move |_| {
                                    set_show_more_menu.set(false);
                                    let selected = selected_files.get();
                                    if selected.is_empty() { return; }
                                    let mut paths = Vec::new();
                                    for item in selected {
                                        let cpath = current_path();
                                        paths.push(if cpath.is_empty() { item.clone() } else { format!("{}/{}", cpath, item) });
                                    }
                                    spawn_local(async move {
                                        let _ = crate::features::file_browser::api::zip_files(paths, "archive.zip".to_string()).await;
                                        set_refresh_trigger.update(|n| *n += 1);
                                    });
                                }>
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>
                                    <span>"Nén zip"</span>
                                </div>
                                <div class="menu-item" on:click=move |_| {
                                    set_show_more_menu.set(false);
                                    let selected = selected_files.get();
                                    if selected.is_empty() { return; }
                                    let item = selected.iter().next().unwrap().clone();
                                    let cpath = current_path();
                                    let full_path = if cpath.is_empty() { item.clone() } else { format!("{}/{}", cpath, item) };
                                    let url = format!("/api/v1/files/download?path={}", js_sys::encode_uri_component(&full_path));
                                    let _ = web_sys::window().unwrap().open_with_url_and_target(&url, "_blank");
                                }>
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                                    <span>"Tải xuống"</span>
                                </div>
                                <div class="menu-item" on:click=move |_| {
                                    set_show_more_menu.set(false);
                                    let selected = selected_files.get();
                                    if selected.is_empty() { return; }
                                    let item = selected.iter().next().unwrap().clone();
                                    let cpath = current_path();
                                    let full_path = if cpath.is_empty() { item.clone() } else { format!("{}/{}", cpath, item) };
                                    let url = format!("{}/api/v1/files/download?path={}", web_sys::window().unwrap().location().origin().unwrap(), js_sys::encode_uri_component(&full_path));
                                    let script = format!("navigator.clipboard.writeText('{}');", url.replace("'", "\\'"));
                                    let _ = js_sys::eval(&script);
                                }>
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="5" r="3"></circle><circle cx="6" cy="12" r="3"></circle><circle cx="18" cy="19" r="3"></circle><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"></line><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"></line></svg>
                                    <span>"Chia sẻ"</span>
                                </div>
                            </div>
                        </Show>
                    </div>
                </Show>
                <Show when=move || !clipboard_files.with(|c| c.is_empty()) && selected_files.with(|s| s.is_empty()) fallback=|| ()>
                    <div style="display: flex; align-items: center; color: #3b82f6; font-weight: bold; padding: 0 12px; margin-right: 8px;">
                        {move || format!("{} mục đang lưu", clipboard_files.with(|s| s.len()))}
                    </div>
                    <button class="action-btn primary" on:click=handle_paste style="color: #3b82f6;">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"></path><rect x="8" y="2" width="8" height="4" rx="1" ry="1"></rect></svg>
                        <span style="font-weight: 600;">"Paste"</span>
                    </button>
                    <button class="action-btn danger" on:click=move |_| {
                        set_clipboard_files.set(HashSet::new());
                        set_clipboard_action.set(None);
                    }>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                        <span>"Hủy bỏ"</span>
                    </button>
                </Show>
            </div>

            // Upload Progress Modal overlay
            <div class="upload-progress-overlay" style=move || { if upload_state.get().is_visible && !upload_state.get().is_minimized { "display: flex;" } else { "display: none;" } }>
                <div class="upload-progress-modal">
                    <div class="modal-title">"Đang chép"</div>
                    
                    <div class="modal-info-text">
                        "Tên: " <strong>{move || upload_state.get().current_file_name.clone()}</strong>
                    </div>
                    <div class="modal-info-text">
                        {move || format!("Tổng cộng: {} mục, Tổng dung lượng: {}", upload_state.get().total_files, format_size(upload_state.get().total_bytes))}
                    </div>
                    
                    <div class="progress-section">
                        <div class="progress-header">
                            <span>"Tiến độ hiện nay:"</span>
                            <span>{move || format!("{:.1}%", upload_state.get().current_file_progress)}</span>
                        </div>
                        <div class="progress-bar-container">
                            <div class="progress-bar-fill" style=move || format!("width: {}%", upload_state.get().current_file_progress)></div>
                        </div>
                    </div>
                    
                    <div class="progress-section" style=move || { if upload_state.get().total_files > 1 { "display: block;" } else { "display: none;" } }>
                        <div class="progress-header">
                            <span>{move || format!("Tổng tiến độ: {}/{}", upload_state.get().files_completed, upload_state.get().total_files)}</span>
                            <span>{move || {
                                let state = upload_state.get();
                                let total_pct = if state.total_files > 0 {
                                    ((state.files_completed as f64) / (state.total_files as f64) * 100.0) as u64
                                } else { 0 };
                                format!("{}%", total_pct)
                            }}</span>
                        </div>
                        <div class="progress-bar-container">
                            <div class="progress-bar-fill" style=move || {
                                let state = upload_state.get();
                                let total_pct = if state.total_files > 0 {
                                    ((state.files_completed as f64) / (state.total_files as f64) * 100.0) as u64
                                } else { 0 };
                                format!("width: {}%", total_pct)
                            }></div>
                        </div>
                    </div>

                    <div class="footer-info">
                        <span>{move || {
                            let sec = upload_state.get().time_remaining_sec;
                            let h = sec / 3600;
                            let m = (sec % 3600) / 60;
                            let s = sec % 60;
                            format!("Thời gian còn lại: {:02}:{:02}:{:02}", h, m, s)
                        }}</span>
                        <span>{move || format!("{:.2} MB/s", upload_state.get().speed_bytes_per_sec / (1024.0 * 1024.0))}</span>
                    </div>

                    <div class="modal-actions">
                        <button on:click=move |_| upload_state.update(|s| s.is_visible = false)>"HỦY"</button>
                        <button on:click=move |_| upload_state.update(|s| s.is_minimized = true)>"ẨN"</button>
                    </div>
                </div>
            </div>
            
            
            <Show when=move || text_editor_file.get().is_some() fallback=|| ()>
                <div class="text-editor-overlay" on:click=move |_| {
                    update_text_editor_file(None);
                    set_is_editing_text.set(false);
                }>
                    <div class="text-editor-modal" on:click=move |ev| ev.stop_propagation()>
                        <div class="text-editor-header">
                            <h3 class="text-editor-title">{move || text_editor_file.get().unwrap_or_default()}</h3>
                            <div class="text-editor-actions">
                                <Show when=move || is_editing_text.get() fallback=move || view! {
                                    <Show when=has_text_prev fallback=|| ()>
                                        <button class="btn-prev" style="margin-right: 4px;" on:click=move |_| text_go(-1)>
                                            <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"><polyline points="15 18 9 12 15 6"></polyline></svg>
                                        </button>
                                    </Show>
                                    <Show when=has_text_next fallback=|| ()>
                                        <button class="btn-next" style="margin-right: 8px;" on:click=move |_| text_go(1)>
                                            <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"><polyline points="9 18 15 12 9 6"></polyline></svg>
                                        </button>
                                    </Show>
                                    <button class="btn-edit" on:click=move |_| {
                                        set_is_editing_text.set(true);
                                        let path = text_editor_file.get().unwrap_or_default();
                                        let parts: Vec<&str> = path.rsplitn(2, '.').collect();
                                        let ext = if parts.len() == 2 { parts[0].to_string() } else { "".to_string() };
                                        leptos::set_timeout(move || {
                                            init_code_mirror("text-editor-textarea", &ext, false);
                                        }, std::time::Duration::from_millis(50));
                                    }>
                                        <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>
                                        "Sửa"
                                    </button>
                                }>
                                    <button class="btn-save" disabled=move || is_saving_text.get() on:click=move |_| {
                                        if let Some(path) = text_editor_file.get() {
                                            set_is_saving_text.set(true);
                                            let content = get_code_mirror_value();
                                            leptos::spawn_local(async move {
                                                let _ = crate::features::file_browser::api::write_text_file(&path, &content).await;
                                                set_is_saving_text.set(false);
                                                set_is_editing_text.set(false);
                                                set_text_editor_content.set(content);
                                            });
                                        }
                                    }>
                                        <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path><polyline points="17 21 17 13 7 13 7 21"></polyline><polyline points="7 3 7 8 15 8"></polyline></svg>
                                        {move || if is_saving_text.get() { "Đang lưu..." } else { "Lưu" }}
                                    </button>
                                </Show>
                                <button class="btn-close" on:click=move |_| {
                                    update_text_editor_file(None);
                                    set_is_editing_text.set(false);
                                }>
                                    <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                                </button>
                            </div>
                        </div>
                        <div class="text-editor-body">
                            <Show when=move || is_editing_text.get() fallback=move || view! {
                                <crate::shared::ui::atoms::text_preview::TextPreview 
                                    content=text_editor_content 
                                    ext=move || {
                                        let path = text_editor_file.get().unwrap_or_default();
                                        let parts: Vec<&str> = path.rsplitn(2, '.').collect();
                                        if parts.len() == 2 { parts[0].to_string() } else { "".to_string() }
                                    }
                                />
                            }>
                                <textarea id="text-editor-textarea" prop:value=move || text_editor_content.get()></textarea>
                            </Show>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || properties_modal_data.get().is_some() fallback=|| ()>
                <div class="properties-modal-overlay" on:click=move |_| set_properties_modal_data.set(None)>
                    <div class="properties-modal" on:click=move |ev| ev.stop_propagation()>
                        <h2>"Thuộc tính"</h2>
                        
                        {move || {
                            let props = properties_modal_data.get().unwrap();
                            let modified_date = js_sys::Date::new(&((props.modified_at as f64) * 1000.0).into());
                            let modified_str = format!("{}/{}/{} {}:{:02}", 
                                modified_date.get_date(), modified_date.get_month() + 1, modified_date.get_full_year() % 100, 
                                modified_date.get_hours(), modified_date.get_minutes());
                            
                            let path_copy1 = props.path.clone();
                            let path_copy2 = props.path.clone();
                            // unused path_chk
                                
                            let thumb = properties_thumb_data.get();
                            
                            view! {
                                <div class="prop-header">
                                    <div class="prop-thumb" style="width: 48px; height: 48px; border-radius: 4px; overflow: hidden; display: flex; align-items: center; justify-content: center; background: #e2e8f0; flex-shrink: 0;">
                                        {if let Some(t) = thumb {
                                            let p = props.path.clone();
                                            view! { <img src=t style="width: 100%; height: 100%; object-fit: cover;" on:error=move |_| {
                                                let p_clone = p.clone();
                                                spawn_local(async move {
                                                    let _ = regen_single_thumbnail(&p_clone).await;
                                                });
                                            }/> }.into_view()
                                        } else if props.file_type == "Nhiều tập tin" {
                                            view! {
                                                <div style="background: white; width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;">
                                                    <svg viewBox="0 0 24 24" fill="#60a5fa" stroke="none" style="width: 32px; height: 32px;"><path d="M4 6H2v14c0 1.1.9 2 2 2h14v-2H4V6zm16-4H8c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm-1 9H9V9h10v2zm-4 4H9v-2h6v2zm4-8H9V5h10v2z"/></svg>
                                                </div>
                                            }.into_view()
                                        } else if props.file_type == "Thư mục" {
                                            view! { <svg viewBox="0 0 24 24" fill="#60a5fa" stroke="none"><path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/></svg> }.into_view()
                                        } else {
                                            view! {
                                                <div style="background: #a0aab5; width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;">
                                                    <svg viewBox="0 0 24 24" fill="white" style="width: 24px; height: 24px;" stroke="none"><path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/></svg>
                                                </div>
                                            }.into_view()
                                        }}
                                    </div>
                                    <div class="prop-title">{props.path.split('/').last().unwrap_or("").to_string()}</div>
                                </div>
                                
                                {if props.file_type != "Nhiều tập tin" {
                                    view! {
                                        <div class="prop-row">
                                            <span class="prop-label">"Kiểu:"</span>
                                            <span class="prop-value">{props.file_type.clone()}</span>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <div></div> }.into_view()
                                }}
                                
                                <div class="prop-row">
                                    <span class="prop-label">"Đường dẫn:"</span>
                                    <span class="prop-value path-value">{props.path.clone()}</span>
                                </div>
                                
                                {if let Some(c) = props.contains.clone() {
                                    view! {
                                        <div class="prop-row">
                                            <span class="prop-label">"Chứa:"</span>
                                            <span class="prop-value">{c}</span>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <div></div> }.into_view()
                                }}
                                
                                <div class="prop-actions">
                                    <button class="prop-btn" on:click=move |_| {
                                        let script = format!("navigator.clipboard.writeText('{}');", path_copy1.replace("'", "\\'").replace("\\", "\\\\"));
                                        let _ = js_sys::eval(&script);
                                    }>"Chép"</button>
                                    <button class="prop-btn" on:click=move |_| {
                                        let full_url = format!("{}/{}", web_sys::window().unwrap().location().origin().unwrap(), path_copy2);
                                        let script = format!("navigator.clipboard.writeText('{}');", full_url.replace("'", "\\'").replace("\\", "\\\\"));
                                        let _ = js_sys::eval(&script);
                                    }>"Copy fullpath"</button>
                                </div>
                                
                                <div class="prop-divider"></div>
                                
                                <div class="prop-row">
                                    <span class="prop-label">"Kích thước:"</span>
                                    <span class="prop-value">{format!("{} ({} Byte)", format_size(props.size), props.size)}</span>
                                </div>
                                <div class="prop-row">
                                    <span class="prop-label">"Đã dùng:"</span>
                                    <span class="prop-value">{format!("{} ({} Byte)", format_size(props.allocated_size), props.allocated_size)}</span>
                                </div>
                                
                                <div class="prop-divider"></div>
                                

                                
                                <Show when=move || {
                                    let props = properties_modal_data.get().unwrap();
                                    props.file_type != "Nhiều tập tin"
                                } fallback=|| ()>
                                    <div class="prop-row">
                                        <span class="prop-label">"Sửa đổi:"</span>
                                        <span class="prop-value">{modified_str.clone()}</span>
                                    </div>
                                    
                                    {if let Some(res) = props.resolution.clone() {
                                        view! {
                                            <div class="prop-row">
                                                <span class="prop-label">"Độ phân giải:"</span>
                                                <span class="prop-value">{res}</span>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! { <div></div> }.into_view()
                                    }}
                                    
                                    <div class="prop-divider"></div>
                                    
                                    <div class="prop-row">
                                        <span class="prop-label">"Đọc được:"</span>
                                        <span class="prop-value">{if props.is_readable { "Có" } else { "Không" }}</span>
                                    </div>
                                    <div class="prop-row">
                                        <span class="prop-label">"Ghi được:"</span>
                                        <span class="prop-value">{if props.is_writable { "Có" } else { "Không" }}</span>
                                    </div>
                                    <div class="prop-row">
                                        <span class="prop-label">"Ẩn:"</span>
                                        <span class="prop-value">{if props.is_hidden { "Có" } else { "Không" }}</span>
                                    </div>
                                    
                                    <div class="prop-divider"></div>
                                    
                                    <div class="prop-row checksum-row">
                                        <span class="prop-label">"Checksum tập tin"</span>
                                        <button class="prop-link-btn" on:click=move |_| {
                                            if is_loading_checksum.get() { return; }
                                            set_is_loading_checksum.set(true);
                                            let path_clone = properties_modal_data.get().unwrap().path;
                                            spawn_local(async move {
                                                if let Ok(c) = crate::features::file_browser::api::get_file_checksum(&path_clone).await {
                                                    set_checksum_data.set(Some(c));
                                                    set_show_checksum_modal.set(true);
                                                }
                                                set_is_loading_checksum.set(false);
                                            });
                                        }>
                                            {if is_loading_checksum.get() { "Đang tính..." } else { "Hiện checksum" }}
                                        </button>
                                    </div>
                                </Show>
                                
                                <div class="prop-footer">
                                    <button class="prop-cancel-btn" on:click=move |_| set_properties_modal_data.set(None)>"HỦY"</button>
                                </div>
                            }
                        }}
                    </div>
                </div>
            </Show>
            
            <Show when=move || show_checksum_modal.get() && checksum_data.get().is_some() fallback=|| ()>
                <div class="properties-modal-overlay" on:click=move |_| set_show_checksum_modal.set(false)>
                    <div class="properties-modal" on:click=move |ev| ev.stop_propagation()>
                        <h2>"Checksum tập tin"</h2>
                        
                        {move || {
                            let data = checksum_data.get().unwrap();
                            let md5_val = data.md5.clone();
                            let sha1_val = data.sha1.clone();
                            let md5_copy = data.md5.clone();
                            let sha1_copy = data.sha1.clone();
                            
                            view! {
                                <div style="margin-bottom: 24px;">
                                    <div style="color: #666; margin-bottom: 24px; font-size: 15px;">
                                        {format!("Tập tin: {}", properties_modal_data.get().unwrap().path.split('/').last().unwrap_or(""))}
                                    </div>
                                    
                                    <div style="margin-bottom: 20px;">
                                        <span style="font-weight: bold; margin-bottom: 8px; display: block; font-size: 16px;">"MD5"</span>
                                        <div style="display: flex; align-items: center; justify-content: space-between; gap: 12px;">
                                            <span style="font-size: 13px; word-break: break-all; color: #555; line-height: 1.4;">{md5_val}</span>
                                            <button class="prop-btn" style="background: #3b82f6; color: white; border-radius: 16px; padding: 6px 16px; font-weight: bold;" on:click=move |_| {
                                                let script = format!("navigator.clipboard.writeText('{}');", md5_copy);
                                                let _ = js_sys::eval(&script);
                                            }>"CHÉP"</button>
                                        </div>
                                    </div>
                                    <div style="margin-bottom: 20px;">
                                        <span style="font-weight: bold; margin-bottom: 8px; display: block; font-size: 16px;">"SHA-1"</span>
                                        <div style="display: flex; align-items: center; justify-content: space-between; gap: 12px;">
                                            <span style="font-size: 13px; word-break: break-all; color: #555; line-height: 1.4;">{sha1_val}</span>
                                            <button class="prop-btn" style="background: #3b82f6; color: white; border-radius: 16px; padding: 6px 16px; font-weight: bold;" on:click=move |_| {
                                                let script = format!("navigator.clipboard.writeText('{}');", sha1_copy);
                                                let _ = js_sys::eval(&script);
                                            }>"CHÉP"</button>
                                        </div>
                                    </div>
                                </div>
                                <div class="prop-footer" style="margin-top: 30px;">
                                    <button class="prop-cancel-btn" on:click=move |_| set_show_checksum_modal.set(false)>"HỦY"</button>
                                </div>
                            }
                        }}
                    </div>
                </div>
            </Show>
            
            <Show when=move || input_modal_open.get() fallback=|| ()>
                <div class="rename-modal-overlay" on:click=move |_| set_input_modal_open.set(false)>
                    <div class="rename-modal" on:click=move |ev| ev.stop_propagation()>
                        <h3>{move || input_modal_title.get()}</h3>
                        <div class="rename-input-group">
                            <input 
                                type="text" 
                                class="rename-input" 
                                value=input_modal_value.get_untracked()
                                on:input=move |ev| set_input_modal_value.set(event_target_value(&ev))
                                on:keydown=move |ev| {
                                    if ev.key() == "Enter" {
                                        submit_input_modal();
                                    }
                                }
                                autofocus=true
                            />
                        </div>
                        <div class="rename-actions">
                            <button class="rename-cancel" on:click=move |_| set_input_modal_open.set(false)>"Hủy"</button>
                            <button class="rename-submit" on:click=move |_| submit_input_modal()>"OK"</button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || rename_dialog_open.get() fallback=|| ()>
                <div class="rename-modal-overlay" on:click=move |_| set_rename_dialog_open.set(false)>
                    <div class="rename-modal" on:click=move |ev| ev.stop_propagation()>
                        <h2>"Đổi tên"</h2>
                        <input
                            type="text"
                            class="rename-input"
                            value=rename_new_name.get_untracked()
                            on:input=move |ev| set_rename_new_name.set(event_target_value(&ev))
                            on:keydown=move |ev| {
                                if ev.key() == "Enter" {
                                    submit_rename();
                                }
                            }
                            autofocus
                        />
                        <div class="rename-actions">
                            <button class="rename-cancel" on:click=move |_| set_rename_dialog_open.set(false)>"HỦY"</button>
                            <button class="rename-submit" on:click=move |_| submit_rename()>"OK"</button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || bulk_rename_dialog_open.get() fallback=|| ()>
                <div class="rename-modal-overlay" on:click=move |_| set_bulk_rename_dialog_open.set(false)>
                    <div class="rename-modal bulk-rename-modal glass-panel" on:click=move |ev| ev.stop_propagation()>
                        <h2>"Đổi tên hàng loạt"</h2>
                        
                        <div class="bulk-rename-form">
                            <div class="form-group">
                                <label>"Tên chung"</label>
                                <input
                                    type="text"
                                    class="rename-input"
                                    value=bulk_common_name.get_untracked()
                                    on:input=move |ev| set_bulk_common_name.set(event_target_value(&ev))
                                    placeholder="Tên chung cho tất cả file..."
                                />
                            </div>
                            
                            <div class="form-group">
                                <label>"Chỉ số (VD: 01, 1, a, A)"</label>
                                <input
                                    type="text"
                                    class="rename-input"
                                    value=bulk_start_index.get_untracked()
                                    on:input=move |ev| set_bulk_start_index.set(event_target_value(&ev))
                                />
                            </div>
                            
                            <div class="form-group">
                                <label>"Định dạng file"</label>
                                <input
                                    type="text"
                                    class="rename-input"
                                    value=bulk_extension.get_untracked()
                                    on:input=move |ev| set_bulk_extension.set(event_target_value(&ev))
                                    placeholder="Mặc định"
                                />
                            </div>
                        </div>

                        <div class="rename-actions">
                            <button class="rename-cancel" on:click=move |_| set_bulk_rename_dialog_open.set(false) disabled=move || is_loading_bulk_rename.get()>"HỦY"</button>
                            <button class="rename-submit" on:click=move |_| submit_bulk_rename() disabled=move || is_loading_bulk_rename.get()>
                                {move || if is_loading_bulk_rename.get() { "Đang xử lý..." } else { "OK" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
            
            <Show when=move || show_delete_modal.get() fallback=|| ()>
                <div class="rename-modal-overlay" on:click=move |_| set_show_delete_modal.set(false)>
                    <div class="rename-modal delete-modal glass-panel" on:click=move |ev| ev.stop_propagation()>
                        <h3>"Xóa"</h3>
                        <p class="delete-msg">
                            "Bạn có chắc chắn xóa "
                            <strong>{move || {
                                let paths = delete_paths.get();
                                if paths.len() == 1 {
                                    paths[0].split('/').last().unwrap_or("").to_string()
                                } else {
                                    format!("{} mục", paths.len())
                                }
                            }}</strong>"?"
                        </p>
                        
                        <div style="margin-top: 12px; margin-bottom: 4px;">
                            <Checkbox 
                                checked=move_to_trash 
                                on_change=move |val| set_move_to_trash.set(val) 
                                label="Chuyển vào thùng rác".to_string() 
                            />
                        </div>
                        
                        <Show when=move || !move_to_trash.get() fallback=|| ()>
                            <p class="delete-warning">"Một khi bị xóa, file sẽ không thể khôi phục được"</p>
                        </Show>

                        <div class="rename-actions">
                            <button class="rename-cancel" on:click=move |_| set_show_delete_modal.set(false)>"HỦY"</button>
                            <button class="rename-submit btn-confirm-delete" on:click=confirm_delete>"OK"</button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || conflict_state.get().is_some() fallback=|| ()>
                {move || {
                    let state = conflict_state.get().unwrap();
                    let info = state.info.clone();
                    
                    let resolve_conflict = move |res: ConflictResolution| {
                        let apply_all = conflict_apply_all.get();
                        if let Some(state_inner) = conflict_state.get_untracked() {
                            if let Some(tx) = state_inner.tx.borrow_mut().take() {
                                let _ = tx.send((res, apply_all));
                            }
                        }
                        conflict_state.set(None);
                        set_conflict_apply_all.set(false);
                    };

                    view! {
                        <div class="rename-modal-overlay">
                            <div class="rename-modal conflict-modal glass-panel" on:click=move |ev| ev.stop_propagation()>
                                <h3>"Ghi đè"</h3>
                                <p class="conflict-msg">
                                    "Trùng tên với tập tin đang có, Bạn có muốn ghi đè lên" <br/>
                                    <strong>{info.dest_name.clone()}</strong>
                                </p>
                                
                                <div class="conflict-details">
                                    <div class="conflict-file">
                                        <div class="conflict-label">"Source file: " <span>{info.source_name.clone()}</span></div>
                                        <div class="conflict-label">"Kích thước: " <span>{format_size(info.source_size)}</span></div>
                                        <div class="conflict-label">"Sửa đổi: " <span>{format_date(info.source_mtime)}</span></div>
                                    </div>
                                    <div class="conflict-file">
                                        <div class="conflict-label">"Dest file: " <span>{info.dest_name.clone()}</span></div>
                                        <div class="conflict-label">"Kích thước: " <span>{format_size(info.dest_size)}</span></div>
                                        <div class="conflict-label">"Sửa đổi: " <span>{format_date(info.dest_mtime)}</span></div>
                                    </div>
                                </div>
                                
                                <div style="margin-top: 16px; margin-bottom: 8px;">
                                    <Checkbox 
                                        checked=conflict_apply_all 
                                        on_change=move |val| set_conflict_apply_all.set(val) 
                                        label="Áp dụng cho tất cả".to_string() 
                                    />
                                </div>

                                <div class="conflict-actions">
                                    <button class="conflict-btn btn-rename" on:click=move |_| resolve_conflict(ConflictResolution::Rename)>"ĐỔI TÊN"</button>
                                    <button class="conflict-btn btn-skip" on:click=move |_| resolve_conflict(ConflictResolution::Skip)>"SKIP"</button>
                                    <button class="conflict-btn btn-overwrite" on:click=move |_| resolve_conflict(ConflictResolution::Overwrite)>"GHI ĐÈ"</button>
                                </div>
                            </div>
                        </div>
                    }
                }}
            </Show>

            <Show when=move || show_trash_modal.get() fallback=|| ()>
                <div class="rename-modal-overlay trash-overlay" on:click=move |_| set_show_trash_modal.set(false)>
                    <div class="rename-modal trash-modal glass-panel" on:click=move |ev| ev.stop_propagation()>
                        <div class="trash-header">
                            <h3>"Thùng rác"</h3>
                            <div class="trash-actions">
                                <button class="btn-empty-trash" on:click=move |_| {
                                    spawn_local(async move {
                                        if crate::features::file_browser::api::empty_trash(None).await.is_ok() {
                                            set_trash_refresh.update(|n| *n += 1);
                                        }
                                    });
                                }>
                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                                    "Tẩy sạch"
                                </button>
                                <button class="btn-close" on:click=move |_| set_show_trash_modal.set(false)>
                                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                                </button>
                            </div>
                        </div>
                        <div class="trash-body">
                            <Transition fallback=move || view! { <div class="loading">"Đang tải..."</div> }>
                                {move || {
                                    let items = trash_resource.get().unwrap_or_default();
                                    if items.is_empty() {
                                        view! { <div class="empty-state">"Thùng rác trống"</div> }.into_view()
                                    } else {
                                        items.into_iter().map(|item| {
                                            let id = item.id.clone();
                                            let id_for_delete = item.id.clone();
                                            view! {
                                                <div class="trash-item">
                                                    <div class="trash-item-info">
                                                        <div class="trash-item-name">{item.original_name}</div>
                                                        <div class="trash-item-path">{item.original_path}</div>
                                                    </div>
                                                    <div class="trash-item-actions">
                                                        <button class="btn-restore" on:click=move |_| {
                                                            let id = id.clone();
                                                            spawn_local(async move {
                                                                if crate::features::file_browser::api::restore_trash(id).await.is_ok() {
                                                                    set_trash_refresh.update(|n| *n += 1);
                                                                    set_refresh_trigger.update(|n| *n += 1);
                                                                }
                                                            });
                                                        }>"Khôi phục"</button>
                                                        <button class="btn-perm-delete" on:click=move |_| {
                                                            let id_for_delete = id_for_delete.clone();
                                                            spawn_local(async move {
                                                                if crate::features::file_browser::api::empty_trash(Some(vec![id_for_delete])).await.is_ok() {
                                                                    set_trash_refresh.update(|n| *n += 1);
                                                                }
                                                            });
                                                        }>"Xóa"</button>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()
                                    }
                                }}
                            </Transition>
                        </div>
                    </div>
                </div>
            </Show>
            
            <MediaViewer 
                files=files_memo 
                active_index=active_media_index 
                playback_mode=playback_mode 
                on_playback_change=update_playback_mode 
                video_speed=video_speed
                on_video_speed_change=update_video_speed
                subtitle_mode=subtitle_mode
                on_subtitle_mode_change=update_subtitle_mode
                show_remaining_time=show_remaining_time
                on_show_remaining_time_change=update_show_remaining_time
                auto_skip_enabled=auto_skip_enabled
                on_auto_skip_enabled_change=update_auto_skip_enabled
            />
        </div>
    }
}
