#![allow(dead_code)]
#![allow(unused_imports)]
use axum::{
    extract::{Path, State},
    routing::{get, post, put, patch, delete},
    Json, Router,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::core::{error::AppError, state::AppState};

#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct CreateUserPayload {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateUserPayload {
    pub name: Option<String>,
    pub email: Option<String>,
}

pub fn router() -> Router<AppState> {
    // Nhóm tất cả các endpoints lại
    Router::new()
        // API thao tác trên Tập hợp (Collection)
        .route("/", get(list_users).post(create_user))
        // API thao tác trên Thực thể (Entity) qua ID
        .route("/:id", get(get_user).put(replace_user).patch(update_user).delete(delete_user))
}

/****
 * [GET] Lấy danh sách Users (Read All)
 * Không yêu cầu ID. Thường dùng chung với Pagination, Filter.
 ****/
async fn list_users(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!(category = "Users", "Lấy danh sách người dùng...");
    Ok(Json(serde_json::json!({
        "status": "success",
        "data": [
            { "id": "1", "name": "John Doe", "email": "john@example.com" },
            { "id": "2", "name": "Jane Doe", "email": "jane@example.com" }
        ],
        "meta": { "total": 2, "page": 1 }
    })))
}

/****
 * [POST] Tạo mới một User (Create)
 * Dùng CreateUserPayload yêu cầu điền đủ tất cả các trường. Trả về mã 201 Created.
 ****/
async fn create_user(
    State(_state): State<AppState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    info!(category = "Users", "Tạo mới người dùng: {}", payload.name);
    // TODO: Gắn logic lưu Database tại đây
    
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "success",
            "message": "Đã tạo người dùng thành công",
            "data": { 
                "id": crate::helpers::suid::suid(), 
                "name": payload.name, 
                "email": payload.email 
            }
        }))
    ))
}

/****
 * [GET] Lấy chi tiết một User theo ID (Read One)
 * Nếu không có, ném AppError để hệ thống tự trả về Format JSON cực chuẩn với mã 404.
 ****/
async fn get_user(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!(category = "Users", "Lấy chi tiết người dùng: {}", id);
    
    // Giả lập lỗi
    if id == "0" {
        return Err(AppError::Custom(StatusCode::NOT_FOUND, "Không tìm thấy người dùng này trong hệ thống".to_string()));
    }

    Ok(Json(serde_json::json!({
        "status": "success",
        "data": { "id": id, "name": "User Demo", "email": "demo@example.com" }
    })))
}

/****
 * [PUT] Thay thế toàn bộ dữ liệu một User (Update Full)
 * Yêu cầu gửi toàn bộ Object (giống POST). Dữ liệu nào thiếu sẽ bị xóa.
 ****/
async fn replace_user(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!(category = "Users", "Thay thế toàn bộ thông tin người dùng: {}", id);
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Đã ghi đè toàn bộ thông tin",
        "data": { "id": id, "name": payload.name, "email": payload.email }
    })))
}

/****
 * [PATCH] Cập nhật một phần dữ liệu User (Update Partial)
 * Dùng UpdateUserPayload với Option<T>. Trường nào có thì mới sửa, không thì bỏ qua.
 ****/
async fn update_user(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!(category = "Users", "Cập nhật một phần thông tin người dùng: {}", id);
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Đã cập nhật một phần dữ liệu",
        "updated_fields": payload
    })))
}

/****
 * [DELETE] Xóa một User (Delete)
 ****/
async fn delete_user(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!(category = "Users", "Xóa người dùng: {}", id);
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Đã xóa người dùng {}", id)
    })))
}
