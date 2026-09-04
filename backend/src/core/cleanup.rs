use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;
use tracing::{info, warn};
use crate::routes::files::{get_thumbnail_name, get_thumbnails_dir, is_media};

pub fn cleanup_orphaned_thumbnails(storage_path: &Path) {
    info!(category = "System", "Bắt đầu quét dọn dẹp các thumbnail rác...");
    let thumbnails_dir = get_thumbnails_dir();
    
    if !thumbnails_dir.exists() {
        return;
    }

    // 1. Quét toàn bộ storage để lấy danh sách hash hợp lệ
    let mut valid_hashes = HashSet::new();
    
    for entry in WalkDir::new(storage_path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".thumbnails" && name != ".tmp" && name != ".git"
        })
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_string_lossy();
            if is_media(&file_name) {
                // Tính toán relative path từ storage root
                if let Ok(rel_path) = entry.path().strip_prefix(storage_path) {
                    let rel_path_str = rel_path.to_string_lossy().replace("\\", "/");
                    let thumb_name = get_thumbnail_name(&rel_path_str);
                    valid_hashes.insert(thumb_name);
                }
            }
        }
    }

    // 2. Quét thư mục .thumbnails để đối chiếu
    let mut deleted_count = 0;
    if let Ok(entries) = std::fs::read_dir(&thumbnails_dir) {
        for entry in entries.filter_map(Result::ok) {
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                let thumb_name = entry.file_name().to_string_lossy().to_string();
                if thumb_name.ends_with(".jpg") {
                    if !valid_hashes.contains(&thumb_name) {
                        let path_to_delete = entry.path();
                        if std::fs::remove_file(&path_to_delete).is_ok() {
                            deleted_count += 1;
                        } else {
                            warn!(category = "System", "Không thể xóa thumbnail rác: {:?}", path_to_delete);
                        }
                    }
                }
            }
        }
    }

    if deleted_count > 0 {
        info!(category = "System", "Đã dọn dẹp thành công {} thumbnail rác.", deleted_count);
    } else {
        info!(category = "System", "Không có thumbnail rác nào cần dọn dẹp.");
    }
}
