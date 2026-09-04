use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error, warn};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrashItem {
    pub id: String,
    pub original_path: String,
    pub original_name: String,
    pub deleted_at: DateTime<Utc>,
    pub size: u64,
    pub is_dir: bool,
}

pub fn get_trash_dir() -> PathBuf {
    crate::routes::files::get_storage_dir().join(".trash")
}

pub fn get_trash_files_dir() -> PathBuf {
    get_trash_dir().join("files")
}

pub fn get_trash_info_dir() -> PathBuf {
    get_trash_dir().join("info")
}

pub fn init_trash_dirs() {
    let _ = fs::create_dir_all(get_trash_files_dir());
    let _ = fs::create_dir_all(get_trash_info_dir());
}

pub fn move_to_trash(rel_path: &str) -> Result<(), String> {
    init_trash_dirs();
    
    let storage_dir = crate::routes::files::get_storage_dir();
    let mut original_full_path = storage_dir.clone();
    original_full_path.push(rel_path.trim_start_matches('/'));

    if !original_full_path.exists() {
        return Err("File not found".to_string());
    }

    let original_name = original_full_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let is_dir = original_full_path.is_dir();
    let size = if is_dir {
        0 // Có thể tính toán folder size sau nếu cần
    } else {
        original_full_path.metadata().map(|m| m.len()).unwrap_or(0)
    };

    let id = Uuid::new_v4().to_string();
    
    let trash_file_path = get_trash_files_dir().join(&id);
    let trash_info_path = get_trash_info_dir().join(format!("{}.json", id));

    // Đổi tên (di chuyển) file vào thùng rác
    if let Err(e) = fs::rename(&original_full_path, &trash_file_path) {
        error!("Lỗi khi di chuyển file vào thùng rác: {}", e);
        return Err(e.to_string());
    }

    let trash_item = TrashItem {
        id,
        original_path: rel_path.to_string(),
        original_name,
        deleted_at: Utc::now(),
        size,
        is_dir,
    };

    // Lưu metadata
    if let Ok(json) = serde_json::to_string(&trash_item) {
        let _ = fs::write(trash_info_path, json);
    }

    // Xóa thumbnail nếu là file
    if !trash_item.is_dir {
        let thumb_path = crate::routes::files::get_thumbnails_dir().join(crate::routes::files::get_thumbnail_name(&trash_item.original_path));
        if thumb_path.exists() {
            let _ = std::fs::remove_file(thumb_path);
        }
    }

    Ok(())
}

pub fn list_trash() -> Vec<TrashItem> {
    init_trash_dirs();
    let mut items = Vec::new();
    
    if let Ok(entries) = fs::read_dir(get_trash_info_dir()) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(item) = serde_json::from_str::<TrashItem>(&content) {
                        items.push(item);
                    }
                }
            }
        }
    }
    
    // Sort by newest deleted first
    items.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));
    items
}

pub fn restore_trash(id: &str) -> Result<(), String> {
    let info_path = get_trash_info_dir().join(format!("{}.json", id));
    let file_path = get_trash_files_dir().join(id);

    if !info_path.exists() || !file_path.exists() {
        return Err("Không tìm thấy file trong thùng rác".to_string());
    }

    let content = fs::read_to_string(&info_path).map_err(|e| e.to_string())?;
    let item: TrashItem = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let storage_dir = crate::routes::files::get_storage_dir();
    let mut restore_path = storage_dir.clone();
    restore_path.push(item.original_path.trim_start_matches('/'));

    // Đảm bảo thư mục cha tồn tại
    if let Some(parent) = restore_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Nếu đã có file trùng tên tại đích, tạo tên mới
    if restore_path.exists() {
        let ext = restore_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let stem = restore_path.file_stem().and_then(|s| s.to_str()).unwrap_or("restored");
        
        let mut counter = 1;
        loop {
            let mut new_name = format!("{} ({})", stem, counter);
            if !ext.is_empty() {
                new_name = format!("{}.{}", new_name, ext);
            }
            let mut new_path = restore_path.parent().unwrap().to_path_buf();
            new_path.push(&new_name);
            
            if !new_path.exists() {
                restore_path = new_path;
                break;
            }
            counter += 1;
        }
    }

    if let Err(e) = fs::rename(&file_path, &restore_path) {
        return Err(e.to_string());
    }

    let _ = fs::remove_file(info_path);

    Ok(())
}

pub fn empty_trash(ids: Option<Vec<String>>) -> Result<(), String> {
    if let Some(id_list) = ids {
        // Chỉ xóa các ID được chỉ định
        for id in id_list {
            let info_path = get_trash_info_dir().join(format!("{}.json", id));
            let file_path = get_trash_files_dir().join(&id);
            
            let _ = fs::remove_file(info_path);
            
            if file_path.is_dir() {
                let _ = fs::remove_dir_all(file_path);
            } else {
                let _ = fs::remove_file(file_path);
            }
        }
    } else {
        // Xóa toàn bộ
        let _ = fs::remove_dir_all(get_trash_dir());
        init_trash_dirs();
    }
    
    Ok(())
}
