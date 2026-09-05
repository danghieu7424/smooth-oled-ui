use axum::{
    extract::{State, Path},
    Json, Router, routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::core::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub user_id: i64,
    pub project_id: String,
    pub name: String,
    pub created_at: String,
    pub version: Option<String>,
    pub is_starred: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Stats {
    pub total_projects: i64,
    pub total_devices: i64,
    pub total_updates: i64,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/stats", get(get_stats))
        .route("/:id", get(get_project).delete(delete_project))
        .route("/:id/star", axum::routing::patch(toggle_star))
        .route("/:id/firmware", axum::routing::post(upload_firmware))
        .route("/:id/firmware/:version", axum::routing::delete(delete_firmware))
        .route("/", get(list_projects).post(create_project))
}

async fn get_stats(State(state): State<Arc<AppState>>) -> Json<Stats> {
    let stats = state.storage.execute_query(|conn| {
        let total_projects: i64 = conn.query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0)).unwrap_or(0);
        let total_devices: i64 = conn.query_row("SELECT COUNT(*) FROM devices", [], |row| row.get(0)).unwrap_or(0);
        
        Ok(Stats {
            total_projects,
            total_devices,
            total_updates: 0, // Placeholder
        })
    }).await.unwrap_or(Stats { total_projects: 0, total_devices: 0, total_updates: 0 });
    
    Json(stats)
}

async fn list_projects(
    jar: axum_extra::extract::cookie::CookieJar,
    State(state): State<Arc<AppState>>
) -> Json<Vec<Project>> {
    let mut user_id = 0;
    if let Some(cookie) = jar.get("auth_token") {
        if let Ok(token_data) = crate::auth::token::verify_user_token(cookie.value()) {
            user_id = token_data.claims.sub;
        }
    }
    
    let projects = state.storage.execute_query(move |conn| {
        let mut stmt = conn.prepare("
            SELECT p.id, p.user_id, p.project_id, p.name, p.created_at, 
                   (SELECT version FROM firmwares WHERE project_id = p.project_id ORDER BY id DESC LIMIT 1) as version,
                   p.is_starred
            FROM projects p
            WHERE p.user_id = ?1
        ")?;
        
        let iter = stmt.query_map(rusqlite::params![user_id], |row| {
            Ok(Project {
                id: row.get(0)?,
                user_id: row.get(1)?,
                project_id: row.get(2)?,
                name: row.get(3)?,
                created_at: row.get(4)?,
                version: row.get(5).unwrap_or(None),
                is_starred: row.get(6).unwrap_or(false),
            })
        })?;
        
        let mut res = Vec::new();
        for item in iter {
            if let Ok(p) = item {
                res.push(p);
            }
        }
        Ok(res)
    }).await.unwrap_or_default();
    
    Json(projects)
}

#[derive(Serialize)]
pub struct ProjectDetailResponse {
    pub id: i64,
    pub project_id: String,
    pub user_suid: String,
    pub name: String,
    pub active_devices: i64,
    pub latest_version: String,
    pub token: String,
    pub firmwares: Vec<serde_json::Value>,
}

async fn get_project(
    jar: axum_extra::extract::cookie::CookieJar,
    Path(id): Path<String>, 
    State(state): State<Arc<AppState>>
) -> Json<ProjectDetailResponse> {
    let mut user_id = 0;
    let mut user_suid = String::new();
    if let Some(cookie) = jar.get("auth_token") {
        if let Ok(token_data) = crate::auth::token::verify_user_token(cookie.value()) {
            user_id = token_data.claims.sub;
            user_suid = token_data.claims.suid;
        }
    }
    let id_clone = id.clone();
    let user_suid_clone = user_suid.clone();
    let detail = state.storage.execute_query(move |conn| {
        let mut p_stmt = conn.prepare("SELECT id, project_id, name, token FROM projects WHERE project_id = ?1 AND user_id = ?2")?;
        let (p_id, p_project_id, p_name, p_token): (i64, String, String, String) = p_stmt.query_row(rusqlite::params![id_clone, user_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;

        let mut d_stmt = conn.prepare("SELECT COUNT(*) FROM devices WHERE project_id = ?1")?;
        let active_devices: i64 = d_stmt.query_row([&p_project_id], |row| row.get(0)).unwrap_or(0);

        let mut f_stmt = conn.prepare("
            SELECT id, version, file_path, notes, created_at, 
                   (SELECT COUNT(*) FROM devices WHERE project_id = ?1 AND current_version = firmwares.version) as devices_count
            FROM firmwares WHERE project_id = ?1 ORDER BY id DESC
        ")?;
        let f_iter = f_stmt.query_map([&p_project_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "version": row.get::<_, String>(1)?,
                "file_path": row.get::<_, String>(2)?,
                "notes": row.get::<_, Option<String>>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "devices_count": row.get::<_, i64>(5)?
            }))
        })?;
        
        let mut firmwares = Vec::new();
        for f in f_iter {
            if let Ok(fv) = f { firmwares.push(fv); }
        }

        let latest_version = firmwares.first()
            .and_then(|v| v["version"].as_str())
            .unwrap_or("N/A")
            .to_string();

        Ok(ProjectDetailResponse {
            id: p_id,
            project_id: p_project_id,
            user_suid: user_suid_clone,
            name: p_name,
            active_devices,
            latest_version,
            token: p_token,
            firmwares,
        })
    }).await.unwrap_or_else(|_| ProjectDetailResponse {
        id: 0,
        project_id: "".to_string(),
        user_suid: "".to_string(),
        name: "Không tìm thấy".to_string(),
        active_devices: 0,
        latest_version: "N/A".to_string(),
        token: "".to_string(),
        firmwares: vec![],
    });

    Json(detail)
}

#[derive(serde::Deserialize)]
pub struct CreateProjectReq {
    pub name: String,
    pub project_id: String,
    pub description: Option<String>,
}

async fn create_project(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::cookie::CookieJar,
    axum::extract::Json(payload): axum::extract::Json<CreateProjectReq>
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    
    let mut user_id = 1;
    if let Some(cookie) = jar.get("auth_token") {
        if let Ok(token_data) = crate::auth::token::verify_user_token(cookie.value()) {
            user_id = token_data.claims.sub;
        }
    }

    let final_project_id = payload.project_id;
    let token = crate::helpers::suid::generate_random_hex();
    
    let res = state.storage.execute_query({
        let name = payload.name.clone();
        let description = payload.description.clone();
        let final_id_clone = final_project_id.clone();
        move |conn| {
            conn.execute(
                "INSERT INTO projects (user_id, project_id, token, name, description) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![user_id, final_id_clone, token, name, description],
            )
        }
    }).await;

    match res {
        Ok(_) => Ok(Json(serde_json::json!({"status": "success", "project_id": final_project_id}))),
        Err(_) => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}

async fn upload_firmware(
    Path(project_id): Path<String>,
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::cookie::CookieJar,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let mut user_id = 1;
    let mut user_suid = "0".to_string();
    if let Some(cookie) = jar.get("auth_token") {
        if let Ok(token_data) = crate::auth::token::verify_user_token(cookie.value()) {
            user_id = token_data.claims.sub;
            user_suid = token_data.claims.suid;
        }
    }

    let is_owner: bool = state.storage.execute_query({
        let p_id = project_id.clone();
        move |conn| {
            let res: i64 = conn.query_row("SELECT 1 FROM projects WHERE project_id = ?1 AND user_id = ?2", rusqlite::params![p_id, user_id], |row| row.get(0)).unwrap_or(0);
            Ok(res == 1)
        }
    }).await.unwrap_or(false);
    
    if !is_owner {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }

    let mut version = String::new();
    let mut file_data = Vec::new();
    let mut notes = String::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if let Some(name) = field.name() {
            if name == "version" {
                version = field.text().await.unwrap_or_default();
            } else if name == "file" {
                file_data = field.bytes().await.unwrap_or_default().to_vec();
            } else if name == "notes" {
                notes = field.text().await.unwrap_or_default();
            }
        }
    }

    if version.is_empty() || file_data.is_empty() {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    // Save firmware file matching the draft_plan.md structure: storages/projects/user_suid/project_id/firmware_version.bin
    let file_path = format!("storages/projects/{}/{}/firmware_{}.bin", user_suid, project_id, version);
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&file_path, &file_data).map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let res = state.storage.execute_query({
        let p_id = project_id.clone();
        let ver = version.clone();
        let path = file_path.clone();
        let n = notes.clone();
        move |conn| {
            // Extract pure version for the ESP to check against
            let core_version = ver.split("_V")
                .last()
                .unwrap_or(&ver)
                .split("_v")
                .last()
                .unwrap_or(&ver)
                .trim()
                .to_string();
                
            conn.execute(
                "INSERT INTO firmwares (project_id, version, core_version, file_path, notes) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![p_id, ver, core_version, path, n],
            )
        }
    }).await;

    match res {
        Ok(_) => Ok(Json(serde_json::json!({"status": "success", "version": version}))),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn toggle_star(
    jar: axum_extra::extract::cookie::CookieJar,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let mut user_id = 0;
    if let Some(cookie) = jar.get("auth_token") {
        if let Ok(token_data) = crate::auth::token::verify_user_token(cookie.value()) {
            user_id = token_data.claims.sub;
        }
    }
    let res = state.storage.execute_query(move |conn| {
        let current_state: bool = conn.query_row(
            "SELECT is_starred FROM projects WHERE project_id = ?1 AND user_id = ?2",
            rusqlite::params![id, user_id],
            |row| row.get(0),
        ).unwrap_or(false);
        
        let new_state = !current_state;
        conn.execute(
            "UPDATE projects SET is_starred = ?1 WHERE project_id = ?2 AND user_id = ?3",
            rusqlite::params![new_state, id, user_id],
        )?;
        Ok(new_state)
    }).await;

    match res {
        Ok(state) => Ok(Json(serde_json::json!({"status": "success", "is_starred": state}))),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_firmware(
    jar: axum_extra::extract::cookie::CookieJar,
    Path((project_id, version)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let mut user_suid = String::new();
    if let Some(cookie) = jar.get("auth_token") {
        if let Ok(token_data) = crate::auth::token::verify_user_token(cookie.value()) {
            user_suid = token_data.claims.suid;
        }
    }
    
    let p_id_clone = project_id.clone();
    let v_clone = version.clone();
    
    // Fetch file_path from DB to safely delete it
    let file_path: Option<String> = state.storage.execute_query(move |conn| {
        use rusqlite::OptionalExtension;
        conn.query_row(
            "SELECT file_path FROM firmwares WHERE project_id = ?1 AND version = ?2",
            rusqlite::params![p_id_clone, v_clone],
            |row| row.get(0)
        ).optional()
    }).await.unwrap_or(None);

    if let Some(path) = file_path {
        let _ = std::fs::remove_file(&path);
    }

    let res = state.storage.execute_query(move |conn| {
        conn.execute(
            "DELETE FROM firmwares WHERE project_id = ?1 AND version = ?2",
            rusqlite::params![project_id, version]
        )
    }).await;
    
    match res {
        Ok(_) => Ok(Json(serde_json::json!({"status": "success"}))),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn download_latest_firmware(
    Path(suid_pid): Path<String>,
    headers: axum::http::HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    // suid_pid format: "007Rlq30Q2vU-esp32-tool"
    let parts: Vec<&str> = suid_pid.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }
    let project_id = parts[1].to_string();

    let latest_fw: Option<(String, String)> = state.storage.execute_query({
        move |conn| {
            use rusqlite::OptionalExtension;
            conn.query_row(
                "SELECT file_path, core_version, version FROM firmwares WHERE project_id = ?1 ORDER BY id DESC LIMIT 1",
                rusqlite::params![project_id],
                |row| {
                    let path: String = row.get(0)?;
                    let core_version: Option<String> = row.get(1)?;
                    let version: String = row.get(2)?;
                    
                    let final_version = core_version.unwrap_or_else(|| {
                        version.split("_V").last().unwrap_or(&version).split("_v").last().unwrap_or(&version).trim().to_string()
                    });
                    Ok((path, final_version))
                }
            ).optional()
        }
    }).await.unwrap_or(None);

    if let Some((path, latest_core_version)) = latest_fw {
        // Check if ESP32 sent its current version
        let device_version = headers.get("x-ESP32-version")
            .or_else(|| headers.get("x-ESP8266-version"))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .trim()
            .trim_start_matches(|c| c == 'v' || c == 'V');
            
        // latest_version bây giờ chính là core_version được lấy trực tiếp từ DB
        let normalized_latest = latest_core_version.trim().trim_start_matches(|c| c == 'v' || c == 'V');
            
        if device_version.eq_ignore_ascii_case(normalized_latest) {
            return Ok(axum::response::Response::builder()
                .status(axum::http::StatusCode::NOT_MODIFIED)
                .body(axum::body::Body::empty())
                .unwrap());
        }
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(file) = tokio::fs::File::open(&path).await {
                let filename = std::path::Path::new(&path).file_name().and_then(|n| n.to_str()).unwrap_or("firmware.bin");
                let stream = tokio_util::io::ReaderStream::new(file);
                let resp = axum::response::Response::builder()
                    .header(axum::http::header::CONTENT_TYPE, "application/octet-stream")
                    .header(axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename))
                    .header(axum::http::header::CONTENT_LENGTH, metadata.len().to_string())
                    .header("x-ESP32-version", normalized_latest)
                    .body(axum::body::Body::from_stream(stream))
                    .unwrap();
                return Ok(resp);
            }
        }
    }
    
    Err(axum::http::StatusCode::NOT_FOUND)
}

async fn delete_project(
    jar: axum_extra::extract::cookie::CookieJar,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let mut user_id = 0;
    let mut user_suid = String::new();
    if let Some(cookie) = jar.get("auth_token") {
        if let Ok(token_data) = crate::auth::token::verify_user_token(cookie.value()) {
            user_id = token_data.claims.sub;
            user_suid = token_data.claims.suid;
        }
    }

    let p_id_clone = id.clone();
    let res = state.storage.execute_query(move |conn| {
        // Validate ownership first
        let exists: i64 = conn.query_row(
            "SELECT 1 FROM projects WHERE project_id = ?1 AND user_id = ?2",
            rusqlite::params![id, user_id],
            |row| row.get(0),
        ).unwrap_or(0);
        
        if exists == 1 {
            conn.execute("DELETE FROM devices WHERE project_id = ?1", [&id])?;
            conn.execute("DELETE FROM firmwares WHERE project_id = ?1", [&id])?;
            conn.execute("DELETE FROM projects WHERE project_id = ?1 AND user_id = ?2", rusqlite::params![id, user_id])?;
        }
        Ok(exists)
    }).await;

    match res {
        Ok(exists) => {
            if exists == 1 && !user_suid.is_empty() {
                let dir_path = format!("storages/projects/{}/{}", user_suid, p_id_clone);
                let _ = std::fs::remove_dir_all(&dir_path);
            }
            Ok(Json(serde_json::json!({"status": "success"})))
        },
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}
