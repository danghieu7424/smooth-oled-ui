#![allow(dead_code)]
use axum::{
    extract::{State, Path},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use crate::core::{state::AppState, error::AppError, jspb::Jspb};
use tracing::info;

// Tuple Struct: Sẽ được serialize thành mảng [id, name, permissions] thay vì Object
#[derive(Serialize)]
pub struct JspbDemoUser(pub u64, pub String, pub Vec<String>);

#[derive(Deserialize)]
pub struct SetPayload {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct GetResponse {
    pub source: String,
    pub value: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/set", post(set_data))
        .route("/get/:key", get(get_data))
        .route("/jspb", get(get_jspb))
}

async fn set_data(
    State(state): State<AppState>,
    Json(payload): Json<SetPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!(category = "Demo", "Nhận request lưu dữ liệu vào Cache & Storage cho key: {}", payload.key);
    
    // Lưu vào Cache (Mini-Redis) với TTL 60 giây
    state.cache.set(&payload.key, &payload.value, Some(60)).await;
    
    // Lưu song song xuống Storage (TableEngine)
    // Dùng dấu ? để AppError tự động hứng mọi lỗi io::Error
    state.storage.set(&payload.key, &payload.value).await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Đã lưu thành công vào cả Cache (RAM) và Storage (Disk)"
    })))
}

async fn get_data(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    // BƯỚC 1: Đọc từ Cache (RAM siêu tốc)
    if let Some(val) = state.cache.get(&key).await {
        info!(category = "Demo", "Cache Hit! Lấy dữ liệu siêu tốc từ RAM.");
        return Ok(Json(serde_json::json!({
            "source": "Cache",
            "value": val
        })));
    }

    // BƯỚC 2: Nếu Cache không có (Cache Miss), đọc từ đĩa cứng (Storage)
    info!(category = "Demo", "Không có trong Cache (Cache Miss), đọc đĩa cứng...");
    
    // Dùng dấu ? để tự động báo lỗi I/O nếu có
    if let Some(val) = state.storage.get(&key).await? {
        // Đẩy lại vào Cache để lần sau đọc nhanh hơn
        state.cache.set(&key, &val, Some(60)).await;
        Ok(Json(serde_json::json!({
            "source": "Storage",
            "value": val
        })))
    } else {
        Err(AppError::Custom(axum::http::StatusCode::NOT_FOUND, "Không tìm thấy dữ liệu".to_string()))
    }
}

/****
 * [GET] /demo/jspb
 * Trả về định dạng Array-of-Arrays cực dị của Google để chống Hijacking và giảm Byte
 ****/
async fn get_jspb() -> Jspb<JspbDemoUser> {
    info!(category = "Demo", "Trả về dữ liệu định dạng JSPB Mảng (Array)");
    let user = JspbDemoUser(
        999,
        "SuperAdmin".to_string(),
        vec!["read".to_string(), "write".to_string(), "delete".to_string()]
    );
    Jspb(user)
}
