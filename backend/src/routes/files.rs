use axum::{
    extract::{Path, State, Query, Multipart, DefaultBodyLimit},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post, put, delete},
    Json, Router,
    body::Body,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use std::os::windows::fs::MetadataExt;
use tokio::fs::{File as TokioFile, read_dir, OpenOptions};
use tokio::io::{AsyncWriteExt, AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use tracing::{info, warn, error};
use sysinfo::System;
use zip::{ZipWriter, write::FileOptions};
use std::io::{Write, Read};
use crate::core::state::AppState;
use crate::core::thumbnail::generate_thumbnail;

pub fn get_storage_dir() -> PathBuf {
    PathBuf::from(std::env::var("STORAGE_PATH").unwrap_or_else(|_| "storages".to_string()))
}

pub fn get_thumbnails_dir() -> PathBuf {
    get_storage_dir().join(".thumbnails")
}

pub fn get_thumbnail_name(full_rel_path: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(full_rel_path.as_bytes());
    let result = hasher.finalize();
    format!("{}.jpg", hex::encode(result))
}

pub fn get_tmp_dir() -> PathBuf {
    get_storage_dir().join(".tmp")
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_files).delete(delete_file))
        .route("/trash", get(list_trash).delete(empty_trash))
        .route("/trash/restore", post(restore_trash))
        .route("/folder", post(create_folder))
        .route("/rename", put(rename_file))
        .route("/move", put(move_file))
        .route("/copy", post(copy_file))
        .route("/upload", post(upload_chunk).layer(DefaultBodyLimit::max(50 * 1024 * 1024)))
        .route("/download", get(download_file))
        .route("/zip", post(zip_files))
        .route("/search", get(search_files))
        .route("/disk", get(disk_usage))
        .route("/properties", get(file_properties))
        .route("/properties_multi", post(multi_file_properties))
        .route("/checksum", get(file_checksum))
        .route("/text", get(read_text_file).post(write_text_file))
        .route("/settings", get(get_folder_settings).post(update_folder_settings))
        .route("/regen-single-thumbnail", post(regen_single_thumbnail))
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub path: Option<String>,
}

#[derive(Serialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified_at: u64,
    pub thumbnail: Option<String>,
    pub children_count: Option<usize>,
}

async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let rel_path = query.path.unwrap_or_else(|| "".to_string());
    let mut base_path = get_storage_dir();
    base_path.push(rel_path.trim_start_matches('/'));

    if !base_path.exists() {
        return (StatusCode::NOT_FOUND, "Không tìm thấy thư mục").into_response();
    }

    let mut entries = match read_dir(&base_path).await {
        Ok(e) => e,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi đọc thư mục").into_response(),
    };

    let mut items = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().into_string().unwrap_or_default();
        let lower_name = name.to_lowercase();
        if lower_name == "thumbs.db" || lower_name == "desktop.ini" || lower_name == ".ds_store" || lower_name == ".trash" || lower_name == ".git" || lower_name == "$recycle.bin" || lower_name == "system volume information" {
            continue;
        }

        if rel_path.is_empty() && (lower_name == ".thumbnails" || lower_name == ".tmp") {
            continue;
        }

        let metadata = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };

        #[cfg(windows)]
        {
            if (metadata.file_attributes() & 2) != 0 {
                continue; // Bỏ qua file/thư mục có thuộc tính Hidden trên Windows
            }
        }
        #[cfg(not(windows))]
        {
            // Trên Linux/Mac, vẫn cho phép thư mục có dấu chấm (.) nếu người dùng cố tình tạo (như .Name).
            // Tuy nhiên, nếu bạn muốn ẩn tất cả dot-files trên Linux như mặc định, có thể bật lại:
            // if name.starts_with('.') && name != ".movies" && name != ".videos" { continue; }
        }

        let is_dir = metadata.is_dir();
        let size = metadata.file_size();
        let modified_at = metadata.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
        
        let mut children_count = None;
        if is_dir {
            if let Ok(mut sub_entries) = read_dir(entry.path()).await {
                let mut count = 0;
                while let Ok(Some(_)) = sub_entries.next_entry().await {
                    count += 1;
                }
                children_count = Some(count);
            }
        }

        let path = if rel_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", rel_path, name)
        };

        let thumbnail = if !is_dir && is_media(&name) {
            let thumb_name = get_thumbnail_name(&path);
            // Luôn trả về URL để Frontend render thẻ img. Nếu file không tồn tại, img onerror sẽ gửi request tạo lại.
            let encoded_thumb_name = urlencoding::encode(&thumb_name);
            Some(format!("/storages/.thumbnails/{}?v={}", encoded_thumb_name, modified_at))
        } else {
            None
        };

        items.push(FileItem {
            name,
            path,
            is_dir,
            size,
            modified_at,
            thumbnail,
            children_count,
        });
    }

    (StatusCode::OK, Json(items)).into_response()
}

fn default_playback_mode() -> String {
    "once".to_string()
}

fn default_video_speed() -> f64 {
    1.0
}

fn default_subtitle_mode() -> u8 {
    1
}

fn default_auto_skip() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FolderSettings {
    pub sort_by: String,
    pub sort_desc: bool,
    pub rename_enabled: bool,
    pub rename_template: String,
    pub use_file_time: bool,
    #[serde(default = "default_playback_mode")]
    pub playback_mode: String,
    #[serde(default = "default_video_speed")]
    pub video_speed: f64,
    #[serde(default = "default_subtitle_mode")]
    pub subtitle_mode: u8,
    #[serde(default)]
    pub show_remaining_time: bool,
    #[serde(default = "default_auto_skip")]
    pub auto_skip_enabled: bool,
}

#[derive(Deserialize)]
pub struct GetSettingsQuery {
    pub path: String,
}

async fn get_folder_settings(
    State(state): State<AppState>,
    Query(query): Query<GetSettingsQuery>,
) -> impl IntoResponse {
    let key = format!("settings_{}", query.path);
    if let Ok(Some(data)) = state.storage.get(&key).await {
        if let Ok(settings) = serde_json::from_str::<FolderSettings>(&data) {
            return (StatusCode::OK, Json(settings)).into_response();
        }
    }
    let default_settings = FolderSettings {
        sort_by: "name".to_string(),
        sort_desc: false,
        rename_enabled: false,
        rename_template: "".to_string(),
        use_file_time: true,
        playback_mode: "once".to_string(),
        video_speed: 1.0,
        subtitle_mode: 1,
        show_remaining_time: false,
        auto_skip_enabled: true,
    };
    (StatusCode::OK, Json(default_settings)).into_response()
}

#[derive(Deserialize)]
pub struct UpdateSettingsReq {
    pub path: String,
    pub settings: FolderSettings,
}

async fn update_folder_settings(
    State(state): State<AppState>,
    Json(payload): Json<UpdateSettingsReq>,
) -> impl IntoResponse {
    let key = format!("settings_{}", payload.path);
    if let Ok(json_str) = serde_json::to_string(&payload.settings) {
        let _ = state.storage.set(&key, &json_str).await;
        (StatusCode::OK, "Saved").into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error").into_response()
    }
}



#[derive(Deserialize)]
pub struct RegenSingleReq {
    pub path: String,
}

async fn regen_single_thumbnail(
    Json(req): Json<RegenSingleReq>,
) -> impl IntoResponse {
    let storage_dir = get_storage_dir();
    let thumbnails_dir = get_thumbnails_dir();
    
    let path_buf = PathBuf::from(&req.path);
    if let Some(file_name) = path_buf.file_name().and_then(|s| s.to_str()) {
        if is_media(file_name) {
            let full_path = storage_dir.join(req.path.trim_start_matches('/'));
            let thumb_path = thumbnails_dir.join(get_thumbnail_name(&req.path));
            
            if full_path.exists() && !thumb_path.exists() {
                let file_name_owned = file_name.to_string();
                tokio::spawn(async move {
                    tracing::info!(category = "System", "Tự động khôi phục thumbnail bị thiếu: {}", file_name_owned);
                    let _ = generate_thumbnail(&full_path, &thumb_path).await;
                });
                return (StatusCode::OK, "Đang khôi phục thumbnail").into_response();
            }
        }
    }
    
    (StatusCode::OK, "Không cần thiết").into_response()
}

pub fn is_media(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png") || lower.ends_with(".webp") || lower.ends_with(".gif") || lower.ends_with(".bmp") || lower.ends_with(".ico") || lower.ends_with(".mp4") || lower.ends_with(".mkv") || lower.ends_with(".avi") || lower.ends_with(".mov") || lower.ends_with(".webm") || lower.ends_with(".flv") || lower.ends_with(".wmv")
}

#[derive(Deserialize)]
pub struct CreateFolderReq {
    pub path: String,
    pub name: String,
}

async fn create_folder(
    State(state): State<AppState>,
    Json(payload): Json<CreateFolderReq>,
) -> impl IntoResponse {
    let mut path = get_storage_dir();
    path.push(payload.path.trim_start_matches('/'));
    path.push(payload.name);

    if path.exists() {
        return (StatusCode::BAD_REQUEST, "Thư mục đã tồn tại").into_response();
    }

    match std::fs::create_dir_all(&path) {
        Ok(_) => (StatusCode::CREATED, "Tạo thư mục thành công").into_response(),
        Err(e) => {
            error!(category = "FileBrowser", "Lỗi tạo thư mục: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi server").into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct DeleteReq {
    pub paths: Vec<String>,
    pub permanent: Option<bool>,
}

async fn delete_file(
    State(_state): State<AppState>,
    Json(payload): Json<DeleteReq>,
) -> impl IntoResponse {
    let permanent = payload.permanent.unwrap_or(false);

    for rel_path in payload.paths {
        if !permanent {
            // Chuyển vào thùng rác
            let _ = crate::infrastructure::trash::move_to_trash(&rel_path);
            continue;
        }

        let mut path = get_storage_dir();
        let trimmed_rel_path = rel_path.trim_start_matches('/');
        path.push(trimmed_rel_path);
        
        if !path.exists() {
            continue;
        }

        if path.is_dir() {
            let _ = std::fs::remove_dir_all(&path);
        } else {
            let _ = std::fs::remove_file(&path);
            
            // Xóa thumbnail tương ứng
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if is_media(name) {
                    let thumb_path = get_thumbnails_dir().join(get_thumbnail_name(trimmed_rel_path));
                    if thumb_path.exists() {
                        let _ = std::fs::remove_file(thumb_path);
                    }
                }
            }
        }
    }
    (StatusCode::OK, "Xóa thành công").into_response()
}

// --- TRASH ENDPOINTS ---
async fn list_trash() -> impl IntoResponse {
    let items = crate::infrastructure::trash::list_trash();
    (StatusCode::OK, Json(items)).into_response()
}

#[derive(Deserialize)]
pub struct RestoreReq {
    pub id: String,
}

async fn restore_trash(Json(payload): Json<RestoreReq>) -> impl IntoResponse {
    match crate::infrastructure::trash::restore_trash(&payload.id) {
        Ok(_) => (StatusCode::OK, "Khôi phục thành công").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[derive(Deserialize)]
pub struct EmptyTrashReq {
    pub ids: Option<Vec<String>>,
}

async fn empty_trash(Json(payload): Json<EmptyTrashReq>) -> impl IntoResponse {
    match crate::infrastructure::trash::empty_trash(payload.ids) {
        Ok(_) => (StatusCode::OK, "Đã dọn dẹp thùng rác").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[derive(Deserialize)]
pub struct RenameReq {
    pub old_path: String,
    pub new_name: String,
}

async fn rename_file(
    State(state): State<AppState>,
    Json(payload): Json<RenameReq>,
) -> impl IntoResponse {
    let mut old_path = get_storage_dir();
    old_path.push(payload.old_path.trim_start_matches('/'));

    if !old_path.exists() {
        return (StatusCode::NOT_FOUND, "Không tìm thấy file/thư mục").into_response();
    }

    let mut new_path = old_path.parent().unwrap_or(&get_storage_dir()).to_path_buf();
    
    // Smart Rename Logic
    let final_new_name = if payload.new_name.contains("yyyy-MM-dd") || payload.new_name.contains("HHhMM") {
        let now = chrono::Local::now();
        let formatted_date = now.format("%Y-%m-%d").to_string();
        let formatted_time = now.format("%Hh%M'").to_string();
        
        payload.new_name
            .replace("yyyy-MM-dd", &formatted_date)
            .replace("HHhMM'", &formatted_time)
            .replace("HHhMM", &formatted_time)
    } else {
        payload.new_name
    };

    new_path.push(&final_new_name);

    if new_path.exists() {
        return (StatusCode::BAD_REQUEST, "Tên mới đã tồn tại").into_response();
    }

    match std::fs::rename(&old_path, &new_path) {
        Ok(_) => {
            // Đổi tên thumbnail tương ứng
            if let Some(old_name) = old_path.file_name().and_then(|s| s.to_str()) {
                if is_media(old_name) {
                    let old_thumb = get_thumbnails_dir().join(get_thumbnail_name(&payload.old_path));
                    
                    let mut new_rel_path = std::path::PathBuf::from(&payload.old_path);
                    new_rel_path.set_file_name(&final_new_name);
                    let new_rel_path_str = new_rel_path.to_string_lossy().to_string().replace("\\", "/");
                    
                    let new_thumb = get_thumbnails_dir().join(get_thumbnail_name(&new_rel_path_str));
                    if old_thumb.exists() {
                        let _ = std::fs::rename(old_thumb, new_thumb);
                    }
                }
            }
            (StatusCode::OK, "Đổi tên thành công").into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi đổi tên").into_response(),
    }
}

fn apply_smart_rename(name: &str, template: Option<&String>, use_file_time: Option<bool>, file_path: Option<&std::path::Path>) -> String {
    if let Some(tpl) = template {
        if tpl.trim().is_empty() { return name.to_string(); }
        
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
        
        let now = if use_file_time.unwrap_or(false) {
            file_path.and_then(|p| p.metadata().ok())
                .and_then(|m| m.modified().ok())
                .map(|t| chrono::DateTime::<chrono::Local>::from(t))
                .unwrap_or_else(|| chrono::Local::now())
        } else {
            chrono::Local::now()
        };
        
        let formatted_date = now.format("%Y-%m-%d").to_string();
        let formatted_time = now.format("%Hh%M'").to_string();
        let formatted_time_alt = now.format("%Hh%M").to_string();
        
        let mut generated = tpl
            .replace("yyyy-MM-dd", &formatted_date)
            .replace("HHhMM'", &formatted_time)
            .replace("HHhMM", &formatted_time_alt)
            .replace("[type]", file_type);
            
        if !ext.is_empty() && generated.ends_with(&format!(".{}", ext)) {
            generated = generated.strip_suffix(&format!(".{}", ext)).unwrap().to_string();
        }
            
        let final_name = if ext.is_empty() {
            generated
        } else {
            format!("{}.{}", generated, ext)
        };
            
        if name == final_name {
            name.to_string()
        } else {
            final_name
        }
    } else {
        name.to_string()
    }
}

fn get_unique_path(mut dest_dir: std::path::PathBuf, final_name: &str, original_name: &str) -> std::path::PathBuf {
    dest_dir.push(final_name);
    if !dest_dir.exists() || final_name == original_name {
        return dest_dir;
    }
    
    let path_obj = std::path::Path::new(final_name);
    let stem = path_obj.file_stem().and_then(|s| s.to_str()).unwrap_or(final_name);
    let ext = path_obj.extension().and_then(|s| s.to_str()).unwrap_or("");
    
    let mut counter = 1;
    dest_dir.pop();
    loop {
        let new_name = if ext.is_empty() {
            format!("{} ({})", stem, counter)
        } else {
            format!("{} ({}).{}", stem, counter, ext)
        };
        dest_dir.push(&new_name);
        if !dest_dir.exists() {
            return dest_dir;
        }
        dest_dir.pop();
        counter += 1;
    }
}

#[derive(Deserialize)]
pub struct MoveReq {
    pub paths: Vec<String>,
    pub dest_path: String,
    pub smart_rename_template: Option<String>,
    pub use_file_time: Option<bool>,
    pub overwrite_paths: Option<Vec<String>>,
}

async fn move_file(
    State(state): State<AppState>,
    Json(payload): Json<MoveReq>,
) -> impl IntoResponse {
    let mut dest_dir = get_storage_dir();
    dest_dir.push(payload.dest_path.trim_start_matches('/'));

    if !dest_dir.exists() {
        return (StatusCode::NOT_FOUND, "Không tìm thấy thư mục đích").into_response();
    }

    for rel_path in payload.paths {
        let mut old_path = get_storage_dir();
        old_path.push(rel_path.trim_start_matches('/'));

        if old_path.exists() {
            let name_str = old_path.file_name().unwrap().to_string_lossy();
            let final_name = if old_path.is_dir() {
                name_str.to_string()
            } else {
                apply_smart_rename(&name_str, payload.smart_rename_template.as_ref(), payload.use_file_time, Some(&old_path))
            };
            let overwrite = payload.overwrite_paths.as_ref().map(|l| l.contains(&rel_path)).unwrap_or(false);
            
            let new_path = if overwrite {
                let mut dest = dest_dir.clone();
                dest.push(&name_str.to_string());
                dest
            } else {
                get_unique_path(dest_dir.clone(), &final_name, &name_str)
            };
            let _ = std::fs::rename(&old_path, &new_path);
        }
    }
    (StatusCode::OK, "Di chuyển thành công").into_response()
}

#[derive(Deserialize)]
pub struct CopyReq {
    pub paths: Vec<String>,
    pub dest_path: String,
    pub smart_rename_template: Option<String>,
    pub use_file_time: Option<bool>,
    pub overwrite_paths: Option<Vec<String>>,
}

async fn copy_file(
    State(state): State<AppState>,
    Json(payload): Json<CopyReq>,
) -> impl IntoResponse {
    let mut dest_dir = get_storage_dir();
    dest_dir.push(payload.dest_path.trim_start_matches('/'));

    if !dest_dir.exists() {
        return (StatusCode::NOT_FOUND, "Không tìm thấy thư mục đích").into_response();
    }

    for rel_path in payload.paths {
        let mut old_path = get_storage_dir();
        old_path.push(rel_path.trim_start_matches('/'));

        if old_path.exists() {
            let name_str = old_path.file_name().unwrap().to_string_lossy();
            let final_name = if old_path.is_dir() {
                name_str.to_string()
            } else {
                apply_smart_rename(&name_str, payload.smart_rename_template.as_ref(), payload.use_file_time, Some(&old_path))
            };
            let overwrite = payload.overwrite_paths.as_ref().map(|l| l.contains(&rel_path)).unwrap_or(false);
            
            let new_path = if overwrite {
                let mut dest = dest_dir.clone();
                dest.push(&name_str.to_string());
                dest
            } else {
                get_unique_path(dest_dir.clone(), &final_name, &name_str)
            };
            
            if old_path.is_dir() {
                let _ = copy_recursively(&old_path, &new_path);
            } else {
                let _ = std::fs::copy(&old_path, &new_path);
            }
        }
    }
    (StatusCode::OK, "Sao chép thành công").into_response()
}

fn copy_recursively(source: &std::path::Path, destination: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_recursively(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct UploadQuery {
    pub path: String,
    pub name: String,
    pub chunk: usize,
    pub total: usize,
    pub mtime: Option<u64>,
    pub offset: Option<u64>,
}

async fn upload_chunk(
    State(state): State<AppState>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut dest_path = get_storage_dir();
    dest_path.push(query.path.trim_start_matches('/'));
    dest_path.push(&query.name);

    // Tạo unique ID cơ bản từ path và name để tránh đụng độ nếu upload cùng tên file khác thư mục
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    query.path.hash(&mut hasher);
    query.name.hash(&mut hasher);
    let tmp_file_path = get_tmp_dir().join(format!("{}_{}", query.name, hasher.finish()));

    if query.chunk == 0 && tmp_file_path.exists() {
        let _ = std::fs::remove_file(&tmp_file_path);
    }

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if let Some(name) = field.name() {
            if name == "file" {
                let data = field.bytes().await.unwrap_or_default();
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&tmp_file_path)
                    .await
                    .unwrap();
                if let Some(offset) = query.offset {
                    let _ = file.seek(std::io::SeekFrom::Start(offset)).await;
                } else {
                    // Fallback to append if offset not provided
                    let _ = file.seek(std::io::SeekFrom::End(0)).await;
                }
                file.write_all(&data).await.unwrap();
            }
        }
    }

    if query.chunk + 1 >= query.total {
        if let Some(parent) = dest_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::rename(&tmp_file_path, &dest_path);

        if let Some(mtime_ms) = query.mtime {
            let sys_time = std::time::UNIX_EPOCH + std::time::Duration::from_millis(mtime_ms);
            if let Ok(f) = OpenOptions::new().write(true).open(&dest_path).await {
                let _ = f.into_std().await.set_modified(sys_time);
            }
        }

        if is_media(&query.name) {
            let full_rel_path = if query.path.is_empty() {
                query.name.clone()
            } else {
                format!("{}/{}", query.path, query.name)
            };
            let thumb_path = get_thumbnails_dir().join(get_thumbnail_name(&full_rel_path));
            let dest_path_clone = dest_path.clone();

            tokio::spawn(async move {
                let _ = generate_thumbnail(&dest_path_clone, &thumb_path).await;
            });
        }
        
        return (StatusCode::OK, "Upload hoàn tất").into_response();
    }

    (StatusCode::OK, format!("Chunk {} uploaded", query.chunk)).into_response()
}

#[derive(Deserialize)]
pub struct DownloadQuery {
    pub path: String,
}

async fn download_file(Query(query): Query<DownloadQuery>) -> impl IntoResponse {
    let mut path = get_storage_dir();
    path.push(query.path.trim_start_matches('/'));

    if !path.exists() || path.is_dir() {
        return (StatusCode::NOT_FOUND, "Không tìm thấy file").into_response();
    }

    let file = match TokioFile::open(&path).await {
        Ok(file) => file,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi mở file").into_response(),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    
    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
    let content_disposition = format!("attachment; filename=\"{}\"", file_name);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(body)
        .unwrap()
}

#[derive(Deserialize)]
pub struct ZipReq {
    pub paths: Vec<String>,
    pub output_name: String,
}

async fn zip_files(Json(payload): Json<ZipReq>) -> impl IntoResponse {
    let out_path = get_storage_dir().join(&payload.output_name);
    let file = std::fs::File::create(&out_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for rel_path in payload.paths {
        let mut target_path = get_storage_dir();
        target_path.push(rel_path.trim_start_matches('/'));
        
        if target_path.exists() {
            let name = target_path.file_name().unwrap().to_string_lossy().to_string();
            if target_path.is_file() {
                zip.start_file(name, options).unwrap();
                let mut f = std::fs::File::open(&target_path).unwrap();
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer).unwrap();
                zip.write_all(&buffer).unwrap();
            }
        }
    }
    zip.finish().unwrap();

    (StatusCode::OK, "Nén thành công").into_response()
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

async fn search_files(Query(query): Query<SearchQuery>) -> impl IntoResponse {
    let mut items = Vec::new();
    let base_path = get_storage_dir();
    search_recursive(&base_path, &query.q.to_lowercase(), &mut items, "").await;
    (StatusCode::OK, Json(items)).into_response()
}

use futures::future::BoxFuture;

fn search_recursive<'a>(
    dir: &'a PathBuf,
    query: &'a str,
    items: &'a mut Vec<FileItem>,
    rel_path: &'a str,
) -> BoxFuture<'a, ()> {
    Box::pin(async move {
        if let Ok(mut entries) = read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if rel_path.is_empty() && (name == ".thumbnails" || name == ".tmp") { continue; }
                
                let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
                let current_rel = if rel_path.is_empty() { name.clone() } else { format!("{}/{}", rel_path, name) };

                if name.to_lowercase().contains(query) {
                    let metadata = entry.metadata().await.unwrap();
                    items.push(FileItem {
                        name: name.clone(),
                        path: current_rel.clone(),
                        is_dir,
                        size: metadata.file_size(),
                        modified_at: metadata.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0),
                        thumbnail: None,
                        children_count: None,
                    });
                }

                if is_dir {
                    let next_dir = dir.join(&name);
                    search_recursive(&next_dir, query, items, &current_rel).await;
                }
            }
        }
    })
}

#[derive(Serialize)]
pub struct DiskUsageInfo {
    pub total_space: u64,
    pub used_space: u64,
}

async fn disk_usage() -> impl IntoResponse {
    use sysinfo::Disks;
    
    let disks = Disks::new_with_refreshed_list();
    
    let current_dir = std::env::current_dir().unwrap_or_default();
    let current_dir_str = current_dir.to_string_lossy().to_string();

    let mut total = 0;
    let mut used = 0;

    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy().to_string();
        if current_dir_str.starts_with(&mount) {
            total = disk.total_space();
            used = total - disk.available_space();
            break;
        }
    }

    // Get quota limit from env
    let limit_gb = std::env::var("STORAGE_LIMIT_GB").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    if limit_gb > 0 {
        let limit_bytes = limit_gb * 1024 * 1024 * 1024;
        total = limit_bytes;
        
        // Calculate actually used space in the storage path
        let storage_path = get_storage_dir();
        let mut actual_used = 0;
        let mut stack = vec![storage_path];
        while let Some(p) = stack.pop() {
            if let Ok(mut entries) = tokio::fs::read_dir(p).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(metadata) = entry.metadata().await {
                        if metadata.is_dir() {
                            stack.push(entry.path());
                        } else {
                            actual_used += metadata.file_size();
                        }
                    }
                }
            }
        }
        used = actual_used;
    }

    (StatusCode::OK, Json(DiskUsageInfo {
        total_space: total,
        used_space: used,
    })).into_response()
}

#[derive(Deserialize)]
pub struct PropertiesQuery {
    pub path: String,
}

#[derive(Serialize)]
pub struct FileProperties {
    pub file_type: String,
    pub path: String,
    pub contains: Option<String>,
    pub size: u64,
    pub allocated_size: u64,
    pub modified_at: u64,
    pub resolution: Option<String>,
    pub is_readable: bool,
    pub is_writable: bool,
    pub is_hidden: bool,
}

pub async fn file_properties(Query(query): Query<PropertiesQuery>) -> impl IntoResponse {
    let mut path = get_storage_dir();
    path.push(query.path.trim_start_matches("/"));

    if !path.exists() {
        return (StatusCode::NOT_FOUND, "Không tìm thấy file").into_response();
    }

    let metadata = match fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi đọc metadata").into_response(),
    };

    let is_dir = metadata.is_dir();
    let file_type = if is_dir { "Thư mục".to_string() } else { "Tập tin".to_string() };
    
    let mut contains = None;
    let mut size = metadata.len();
    
    if is_dir {
        let (fc, dc, sz) = compute_recursive_stats(&path);
        contains = Some(format!("{} Tập tin, {} Thư mục", fc, dc));
        size = sz;
    }
    
    let mut resolution = None;
    if !is_dir {
        if let Ok(dim) = image::image_dimensions(&path) {
            resolution = Some(format!("{}x{}", dim.0, dim.1));
        }
    }

    let modified_at = metadata.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);

    let is_hidden = {
        #[cfg(windows)]
        {
            (metadata.file_attributes() & 2) != 0
        }
        #[cfg(not(windows))]
        {
            path.file_name().and_then(|n| n.to_str()).map(|s| s.starts_with(".")).unwrap_or(false)
        }
    };

    let is_writable = !metadata.permissions().readonly();

    let props = FileProperties {
        file_type,
        path: query.path,
        contains,
        size,
        allocated_size: size,
        modified_at,
        resolution,
        is_readable: true,
        is_writable,
        is_hidden,
    };

    Json(props).into_response()
}

#[derive(Serialize)]
pub struct ChecksumResponse {
    pub md5: String,
    pub sha1: String,
}

pub async fn file_checksum(Query(query): Query<PropertiesQuery>) -> impl IntoResponse {
    let mut path = get_storage_dir();
    path.push(query.path.trim_start_matches("/"));

    if !path.exists() || path.is_dir() {
        return (StatusCode::NOT_FOUND, "Không tìm thấy file").into_response();
    }

    use md5::{Md5, Digest as Md5Digest};
    use sha1::{Sha1, Digest as Sha1Digest};
    use std::io::Read;
    
    let mut file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi mở file").into_response(),
    };
    
    let mut md5_hasher = Md5::new();
    let mut sha1_hasher = Sha1::new();
    let mut buffer = [0; 1024 * 64];
    
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                md5_hasher.update(&buffer[..n]);
                sha1_hasher.update(&buffer[..n]);
            },
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi đọc file").into_response(),
        }
    }
    
    let md5_bytes = md5_hasher.finalize();
    let md5_str = md5_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    
    let sha1_bytes = sha1_hasher.finalize();
    let sha1_str = sha1_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    Json(ChecksumResponse { md5: md5_str, sha1: sha1_str }).into_response()
}

fn compute_recursive_stats(base_path: &std::path::Path) -> (u64, u64, u64) {
    let mut file_count = 0;
    let mut dir_count = 0;
    let mut total_size = 0;

    for entry in walkdir::WalkDir::new(base_path).into_iter().filter_map(Result::ok) {
        if entry.path() == base_path {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if meta.is_dir() {
                dir_count += 1;
            } else {
                file_count += 1;
                total_size += meta.len();
            }
        }
    }
    (file_count, dir_count, total_size)
}

#[derive(serde::Deserialize)]
pub struct MultiPropertiesReq {
    pub paths: Vec<String>,
}

pub async fn multi_file_properties(Json(req): Json<MultiPropertiesReq>) -> impl IntoResponse {
    let mut total_file = 0;
    let mut total_dir = 0;
    let mut total_size = 0;
    
    let mut parent_path = String::new();
    
    for path_str in req.paths {
        let mut path = get_storage_dir();
        path.push(path_str.trim_start_matches("/"));
        
        if parent_path.is_empty() {
            let p = std::path::Path::new(&path_str);
            if let Some(parent) = p.parent() {
                parent_path = parent.to_string_lossy().replace("\\", "/");
                if parent_path.is_empty() {
                    parent_path = "/".to_string();
                } else if !parent_path.starts_with("/") {
                    parent_path = format!("/{}", parent_path);
                }
                if !parent_path.ends_with("/") {
                    parent_path.push('/');
                }
            }
        }
        
        if !path.exists() { continue; }
        if let Ok(meta) = std::fs::metadata(&path) {
            if meta.is_dir() {
                total_dir += 1;
                let (fc, dc, sz) = compute_recursive_stats(&path);
                total_file += fc;
                total_dir += dc;
                total_size += sz;
            } else {
                total_file += 1;
                total_size += meta.len();
            }
        }
    }
    
    if parent_path.is_empty() {
        parent_path = "/".to_string();
    }
    
    let props = FileProperties {
        file_type: "Nhiều tập tin".to_string(),
        path: parent_path,
        contains: Some(format!("{} Tập tin, {} Thư mục", total_file, total_dir)),
        size: total_size,
        allocated_size: total_size,
        modified_at: 0,
        resolution: None,
        is_readable: true,
        is_writable: true,
        is_hidden: false,
    };
    
    Json(props).into_response()
}

#[derive(Deserialize)]
pub struct TextFileQuery {
    pub path: String,
}

#[derive(Deserialize)]
pub struct TextFilePayload {
    pub path: String,
    pub content: String,
}

pub async fn read_text_file(
    State(_state): State<AppState>,
    Query(query): Query<TextFileQuery>,
) -> impl IntoResponse {
    let mut base_path = get_storage_dir();
    base_path.push(query.path.trim_start_matches('/'));

    if !base_path.exists() {
        return (StatusCode::NOT_FOUND, "File không tồn tại").into_response();
    }

    match tokio::fs::read_to_string(&base_path).await {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Lỗi đọc file: {}", e)).into_response(),
    }
}

pub async fn write_text_file(
    State(_state): State<AppState>,
    Json(payload): Json<TextFilePayload>,
) -> impl IntoResponse {
    let mut base_path = get_storage_dir();
    base_path.push(payload.path.trim_start_matches('/'));

    if let Some(parent) = base_path.parent() {
        if !parent.exists() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    match tokio::fs::write(&base_path, &payload.content).await {
        Ok(_) => (StatusCode::OK, "Đã lưu file thành công".to_string()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Lỗi lưu file: {}", e)).into_response(),
    }
}


