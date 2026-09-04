use serde::{Deserialize, Serialize};
use gloo_net::http::Request;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified_at: u64,
    pub thumbnail: Option<String>,
    pub children_count: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiskUsageInfo {
    pub total_space: u64,
    pub used_space: u64,
}

pub async fn fetch_files(path: &str) -> Result<Vec<FileItem>, String> {
    let url = if path.is_empty() {
        "/api/v1/files".to_string()
    } else {
        format!("/api/v1/files?path={}", path)
    };

    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<Vec<FileItem>>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(format!("Error fetching files: {}", resp.status()))
    }
}

pub async fn fetch_disk_usage() -> Result<DiskUsageInfo, String> {
    let resp = Request::get("/api/v1/files/disk")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<DiskUsageInfo>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(format!("Error fetching disk usage: {}", resp.status()))
    }
}

pub async fn create_folder(path: &str, name: &str) -> Result<(), String> {
    let req_body = serde_json::json!({
        "path": path,
        "name": name
    })
    .to_string();

    let resp = Request::post("/api/v1/files/folder")
        .header("Content-Type", "application/json")
        .body(req_body)
        .unwrap()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Error creating folder: {}", resp.status()))
    }
}

pub async fn upload_chunk(
    path: &str,
    name: &str,
    chunk: usize,
    total: usize,
    offset: u64,
    mtime: Option<f64>,
    file_data: web_sys::Blob,
) -> Result<(), String> {
    let mtime_str = if let Some(t) = mtime {
        format!("&mtime={}", t as u64)
    } else {
        "".to_string()
    };
    
    let url = format!(
        "/api/v1/files/upload?path={}&name={}&chunk={}&total={}&offset={}{}",
        js_sys::encode_uri_component(path).as_string().unwrap_or_default(),
        js_sys::encode_uri_component(name).as_string().unwrap_or_default(),
        chunk,
        total,
        offset,
        mtime_str
    );

    let form_data = web_sys::FormData::new().map_err(|_| "Lỗi FormData".to_string())?;
    form_data
        .append_with_blob("file", &file_data)
        .map_err(|_| "Lỗi đính kèm file".to_string())?;

    let resp = Request::post(&url)
        .body(form_data)
        .unwrap()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Error uploading chunk: {}", resp.status()))
    }
}

pub async fn move_files(paths: Vec<String>, dest_path: String, smart_rename_template: Option<String>, use_file_time: Option<bool>, overwrite_paths: Option<Vec<String>>) -> Result<(), String> {
    let req_body = serde_json::json!({
        "paths": paths,
        "dest_path": dest_path,
        "smart_rename_template": smart_rename_template,
        "use_file_time": use_file_time,
        "overwrite_paths": overwrite_paths
    })
    .to_string();

    let resp = Request::put("/api/v1/files/move")
        .header("Content-Type", "application/json")
        .body(req_body)
        .unwrap()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Error moving files: {}", resp.status()))
    }
}

pub async fn delete_files(paths: Vec<String>, permanent: bool) -> Result<(), String> {
    let req_body = serde_json::json!({
        "paths": paths,
        "permanent": permanent
    })
    .to_string();

    let resp = Request::delete("/api/v1/files")
        .header("Content-Type", "application/json")
        .body(req_body)
        .unwrap()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Error deleting files: {}", resp.status()))
    }
}

// --- TRASH APIs ---

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct TrashItem {
    pub id: String,
    pub original_path: String,
    pub original_name: String,
    pub deleted_at: String, // Serialize directly or as string
    pub size: u64,
    pub is_dir: bool,
}

pub async fn get_trash() -> Result<Vec<TrashItem>, String> {
    let resp = Request::get("/api/v1/files/trash")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        let items: Vec<TrashItem> = resp.json().await.unwrap_or_default();
        Ok(items)
    } else {
        Err(format!("Lỗi khi lấy danh sách thùng rác: {}", resp.status()))
    }
}

pub async fn restore_trash(id: String) -> Result<(), String> {
    let req_body = serde_json::json!({ "id": id }).to_string();
    
    let resp = Request::post("/api/v1/files/trash/restore")
        .header("Content-Type", "application/json")
        .body(req_body)
        .unwrap()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Lỗi khi khôi phục: {}", resp.status()))
    }
}

pub async fn empty_trash(ids: Option<Vec<String>>) -> Result<(), String> {
    let req_body = serde_json::json!({ "ids": ids }).to_string();
    
    let resp = Request::delete("/api/v1/files/trash")
        .header("Content-Type", "application/json")
        .body(req_body)
        .unwrap()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Lỗi khi dọn thùng rác: {}", resp.status()))
    }
}


pub async fn copy_files(paths: Vec<String>, dest_path: String, smart_rename_template: Option<String>, use_file_time: Option<bool>, overwrite_paths: Option<Vec<String>>) -> Result<(), String> {
    let req_body = serde_json::json!({
        "paths": paths,
        "dest_path": dest_path,
        "smart_rename_template": smart_rename_template,
        "use_file_time": use_file_time,
        "overwrite_paths": overwrite_paths
    })
    .to_string();

    let resp = Request::post("/api/v1/files/copy")
        .header("Content-Type", "application/json")
        .body(req_body)
        .unwrap()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Error copying files: {}", resp.status()))
    }
}


pub async fn rename_file(old_path: String, new_name: String) -> Result<(), String> {
    let req_body = serde_json::json!({
        "old_path": old_path,
        "new_name": new_name
    })
    .to_string();

    let resp = Request::put("/api/v1/files/rename")
        .header("Content-Type", "application/json")
        .body(req_body)
        .unwrap()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Error renaming file: {}", resp.status()))
    }
}


#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
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

#[derive(serde::Deserialize, Clone)]
pub struct ChecksumResponse {
    pub md5: String,
    pub sha1: String,
}

pub async fn get_file_properties(path: &str) -> Result<FileProperties, String> {
    let url = format!("/api/v1/files/properties?path={}", js_sys::encode_uri_component(path));
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<FileProperties>().await.map_err(|e| e.to_string())
    } else {
        Err(resp.text().await.unwrap_or_else(|_| "Lỗi lấy thuộc tính".to_string()))
    }
}

pub async fn get_file_checksum(path: &str) -> Result<ChecksumResponse, String> {
    let url = format!("/api/v1/files/checksum?path={}", js_sys::encode_uri_component(path));
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;

    if resp.ok() {
        let data = resp.json::<ChecksumResponse>().await.map_err(|e| e.to_string())?;
        Ok(data)
    } else {
        Err(resp.text().await.unwrap_or_else(|_| "Lỗi lấy checksum".to_string()))
    }
}


#[derive(serde::Serialize)]
pub struct ZipReq {
    pub paths: Vec<String>,
    pub output_name: String,
}

pub async fn zip_files(paths: Vec<String>, output_name: String) -> Result<(), String> {
    let req = ZipReq { paths, output_name };
    let json = serde_json::to_string(&req).unwrap();
    let resp = Request::post("/api/v1/files/zip")
        .header("Content-Type", "application/json")
        .body(json).map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(resp.text().await.unwrap_or_else(|_| "Lỗi nén zip".to_string()))
    }
}


#[derive(serde::Serialize)]
pub struct MultiPropertiesReq {
    pub paths: Vec<String>,
}

pub async fn get_multi_file_properties(paths: Vec<String>) -> Result<FileProperties, String> {
    let req = MultiPropertiesReq { paths };
    let json = serde_json::to_string(&req).unwrap();
    let resp = Request::post("/api/v1/files/properties_multi")
        .header("Content-Type", "application/json")
        .body(json).map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<FileProperties>().await.map_err(|e| e.to_string())
    } else {
        Err(resp.text().await.unwrap_or_else(|_| "Lỗi lấy thuộc tính nhiều file".to_string()))
    }
}

pub async fn read_text_file(path: &str) -> Result<String, String> {
    let url = format!("/api/v1/files/text?path={}", js_sys::encode_uri_component(path));
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.text().await.map_err(|e| e.to_string())
    } else {
        Err(resp.text().await.unwrap_or_else(|_| "Lỗi đọc file".to_string()))
    }
}

#[derive(serde::Serialize)]
struct WriteTextFileReq {
    path: String,
    content: String,
}

pub async fn write_text_file(path: &str, content: &str) -> Result<(), String> {
    let req = WriteTextFileReq {
        path: path.to_string(),
        content: content.to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let resp = Request::post("/api/v1/files/text")
        .header("Content-Type", "application/json")
        .body(json).map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(resp.text().await.unwrap_or_else(|_| "Lỗi lưu file".to_string()))
    }
}




pub async fn regen_single_thumbnail(path: &str) -> Result<(), String> {
    let req_body = serde_json::json!({
        "path": path
    });
    let json = serde_json::to_string(&req_body).unwrap();
    let resp = gloo_net::http::Request::post("/api/v1/files/regen-single-thumbnail")
        .header("Content-Type", "application/json")
        .body(json).unwrap()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(resp.text().await.unwrap_or_default())
    }
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
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

pub async fn fetch_folder_settings(path: &str) -> Result<FolderSettings, String> {
    let url = format!("/api/v1/files/settings?path={}", js_sys::encode_uri_component(path));
    let resp = gloo_net::http::Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json::<FolderSettings>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Error fetching settings: {}", resp.status()))
    }
}

pub async fn save_folder_settings(path: &str, settings: FolderSettings) -> Result<(), String> {
    let req_body = serde_json::json!({
        "path": path,
        "settings": settings
    });
    let json = serde_json::to_string(&req_body).unwrap();
    let resp = gloo_net::http::Request::post("/api/v1/files/settings")
        .header("Content-Type", "application/json")
        .body(json)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Error saving settings: {}", resp.status()))
    }
}
