#![allow(dead_code)]
use rusqlite::{Connection, Result, params};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};
use std::path::Path;

#[derive(Clone)]
pub struct TableEngine {
    conn: Arc<Mutex<Connection>>,
}

impl TableEngine {
    pub async fn new<P: AsRef<Path>>(db_path: P) -> std::io::Result<Self> {
        let path_str = db_path.as_ref().to_string_lossy().to_string();
        
        let conn = tokio::task::spawn_blocking(move || -> Result<Connection> {
            let conn = Connection::open(&path_str)?;
            
            // Khởi tạo bảng cho OTA System
            conn.execute(
                "CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    email TEXT UNIQUE NOT NULL,
                    google_id TEXT UNIQUE NOT NULL,
                    is_verified BOOLEAN DEFAULT 0,
                    role TEXT DEFAULT 'user',
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS projects (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id INTEGER NOT NULL,
                    project_id TEXT UNIQUE NOT NULL,
                    token TEXT NOT NULL,
                    name TEXT NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY(user_id) REFERENCES users(id)
                )",
                [],
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS firmwares (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id TEXT NOT NULL,
                    version TEXT NOT NULL,
                    file_path TEXT NOT NULL,
                    notes TEXT,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY(project_id) REFERENCES projects(project_id)
                )",
                [],
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS devices (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id TEXT NOT NULL,
                    mac_address TEXT NOT NULL,
                    current_version TEXT NOT NULL,
                    last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(project_id, mac_address)
                )",
                [],
            )?;

            Ok(conn)
        }).await.unwrap().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        info!(category = "Database", "Đã khởi tạo SQLite thành công tại: {}", db_path.as_ref().display());

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    // Các hàm phụ trợ thực thi query qua spawn_blocking
    pub async fn execute_query<F, T>(&self, func: F) -> std::io::Result<T>
    where
        F: FnOnce(&Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn_clone = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_clone.blocking_lock();
            func(&conn)
        })
        .await
        .unwrap()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}