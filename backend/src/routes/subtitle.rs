/****
 * Module: Subtitle API
 * Chức năng: Đọc file .msub (TOML format), trích xuất ngôn ngữ VTT tương ứng trả về cho Trình duyệt
 * Đầu vào: file path (rel path) và lang (optional, default: vi)
 ****/

use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::fs;

#[derive(Deserialize)]
pub struct SubtitleQuery {
    pub file: String,
    pub lang: Option<String>,
}

#[derive(Deserialize)]
struct SubtitleTrack {
    content: String,
}

pub async fn get_subtitle(Query(query): Query<SubtitleQuery>) -> impl IntoResponse {
    let storage_dir = crate::routes::files::get_storage_dir();
    
    // Bảo mật: Ngăn chặn Path Traversal
    // // Căn cứ: Yêu cầu Sparing Partner - Tuân thủ nguyên tắc Bảo mật (INTEGRITY)
    if query.file.contains("..") {
        return (StatusCode::BAD_REQUEST, "Invalid file path").into_response();
    }
    
    let file_path = storage_dir.join(&query.file);
    
    let file_content = match fs::read_to_string(&file_path).await {
        Ok(c) => c,
        Err(_) => return (StatusCode::NOT_FOUND, "Subtitle file not found").into_response(),
    };
    
    let parsed: HashMap<String, SubtitleTrack> = match toml::from_str(&file_content) {
        Ok(p) => p,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Invalid .msub format").into_response(),
    };
    
    let lang = query.lang.unwrap_or_else(|| "vi".to_string());
    
    // Ưu tiên: Ngôn ngữ user chọn -> Tiếng Việt -> Bất kỳ ngôn ngữ nào có sẵn
    let track = parsed.get(&lang).or_else(|| parsed.get("vi")).or_else(|| parsed.values().next());
    
    match track {
        Some(t) => {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/vtt; charset=utf-8")],
                t.content.clone(),
            ).into_response()
        },
        None => (StatusCode::NOT_FOUND, "Language not found in .msub").into_response(),
    }
}
