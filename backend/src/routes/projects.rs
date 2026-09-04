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
        .route("/:id", get(get_project))
        .route("/:id/firmware", axum::routing::post(upload_firmware))
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

async fn list_projects(State(state): State<Arc<AppState>>) -> Json<Vec<Project>> {
    let projects = state.storage.execute_query(|conn| {
        let mut stmt = conn.prepare("
            SELECT p.id, p.user_id, p.project_id, p.name, p.created_at, 
                   (SELECT version FROM firmwares WHERE project_id = p.project_id ORDER BY id DESC LIMIT 1) as version
            FROM projects p
        ")?;
        
        let iter = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                user_id: row.get(1)?,
                project_id: row.get(2)?,
                name: row.get(3)?,
                created_at: row.get(4)?,
                version: row.get(5).unwrap_or(None),
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
    pub name: String,
    pub active_devices: i64,
    pub latest_version: String,
    pub firmwares: Vec<serde_json::Value>,
}

async fn get_project(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> Json<ProjectDetailResponse> {
    let id_clone = id.clone();
    let detail = state.storage.execute_query(move |conn| {
        let mut p_stmt = conn.prepare("SELECT id, project_id, name FROM projects WHERE project_id = ?1")?;
        let (p_id, p_project_id, p_name): (i64, String, String) = p_stmt.query_row([&id_clone], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;

        let active_devices: i64 = conn.query_row("SELECT COUNT(*) FROM devices WHERE project_id = ?1", [&id_clone], |row| row.get(0)).unwrap_or(0);
        
        let mut fw_stmt = conn.prepare("
            SELECT id, version, file_path, notes, created_at, 
                   (SELECT COUNT(*) FROM devices WHERE project_id = ?1 AND current_version = firmwares.version) as devices_count
            FROM firmwares WHERE project_id = ?1 ORDER BY id DESC
        ")?;

        let fw_iter = fw_stmt.query_map([&id_clone, &id_clone], |row| {
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
        for fw in fw_iter {
            if let Ok(f) = fw { firmwares.push(f); }
        }

        let latest_version = firmwares.first().and_then(|v| v.get("version").and_then(|s| s.as_str())).unwrap_or("Chưa có").to_string();

        Ok(ProjectDetailResponse {
            id: p_id,
            project_id: p_project_id,
            name: p_name,
            active_devices,
            latest_version,
            firmwares,
        })
    }).await.unwrap_or_else(|_| ProjectDetailResponse {
        id: 0,
        project_id: id,
        name: "Không tìm thấy".to_string(),
        active_devices: 0,
        latest_version: "N/A".to_string(),
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

    let final_project_id = format!("{}-{}", user_id, payload.project_id);
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
    mut multipart: axum::extract::Multipart,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let mut version = String::new();
    let mut file_data = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if let Some(name) = field.name() {
            if name == "version" {
                version = field.text().await.unwrap_or_default();
            } else if name == "file" {
                file_data = field.bytes().await.unwrap_or_default().to_vec();
            }
        }
    }

    if version.is_empty() || file_data.is_empty() {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    // Save firmware file (mock path for now, in real app save to storage)
    let file_path = format!("storages/firmwares/{}_{}.bin", project_id, version);
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&file_path, &file_data).map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let res = state.storage.execute_query({
        let p_id = project_id.clone();
        let ver = version.clone();
        let path = file_path.clone();
        move |conn| {
            conn.execute(
                "INSERT INTO firmwares (project_id, version, file_path) VALUES (?1, ?2, ?3)",
                rusqlite::params![p_id, ver, path],
            )
        }
    }).await;

    match res {
        Ok(_) => Ok(Json(serde_json::json!({"status": "success", "version": version}))),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}
