use axum::{
    extract::{Request, DefaultBodyLimit},
    response::{Response, IntoResponse},
    routing::{Router},
    http::{Method, HeaderValue},
};
use axum::http::header::{self, HeaderName};
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::TraceLayer,
};
use dotenvy::dotenv;
use std::fs;
use std::path::Path;
use tokio::signal;
use tracing::{info, warn};
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

mod core;
mod infrastructure;
mod helpers;
mod auth;
mod routes;
mod utils;

use rust_embed::RustEmbed;
use axum::http::{StatusCode, Uri};
use axum::response::Html;
use mime_guess::from_path;

#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
struct FrontendAssets;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }
    
    if path == "package.json" {
        return (
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            include_bytes!("../../package.json").to_vec(),
        ).into_response();
    }

    match FrontendAssets::get(&path) {
        Some(content) => {
            let mime = from_path(&path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            ).into_response()
        }
        None => {
            if let Some(index_content) = FrontendAssets::get("index.html") {
                Html(index_content.data).into_response()
            } else {
                (StatusCode::NOT_FOUND, "404 Not Found").into_response()
            }
        }
    }
}

static FAVICON: &[u8] = include_bytes!("../static/favicon.ico");

async fn favicon_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/x-icon")],
        FAVICON,
    )
}

#[tokio::main]
async fn main() {
    // Đảm bảo thư mục gốc luôn là thư mục chứa file chạy (để tính năng khởi động cùng hệ thống hoạt động đúng do Windows hay gọi từ System32)
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let _ = std::env::set_current_dir(parent);
        }
    }

    dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let cmd = args[1].as_str();
        if matches!(cmd, "cli" | "setup" | "--setup" | "-c" | "start" | "--start" | "-s" | "stop" | "--stop" | "-k" | "status" | "--status" | "-t" | "logs" | "--logs" | "-l") {
            utils::cli::handle_cli(args).await;
            return;
        }
    }

    colored::control::set_override(true);
    
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let is_prod = std::env::var("APP_ENV").unwrap_or_default() == "production";

    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,hyper=warn,reqwest=warn,sqlx=warn,tower=warn,h2=warn"));

    if is_prod {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(non_blocking_writer)
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .compact()
                    .with_ansi(false)
                    .with_writer(non_blocking_writer)
            )
            .with(crate::core::logger::ColorTerminalLayer)
            .init();
    }

    let port_str = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());


    let local_cache = crate::infrastructure::cache::LocalCache::init(50_000).await;
    let table_engine = crate::infrastructure::storage::TableEngine::new("storages/demo.rdb").await.expect("Không tạo được TableEngine");

    let server_private_key_hex = std::env::var("SERVER_PRIVATE_KEY_HEX")
        .expect("Thảm họa: Chưa set SERVER_PRIVATE_KEY_HEX trong file .env");

    let server_private_key_bytes: [u8; 32] = hex::decode(&server_private_key_hex)
        .expect("SERVER_PRIVATE_KEY_HEX không phải là chuỗi Hex hợp lệ")
        .try_into()
        .expect("SERVER_PRIVATE_KEY_HEX phải có độ dài chính xác là 32 bytes (64 ký tự hex)");

    let (sse_tx, _) = tokio::sync::broadcast::channel(1000);

    let app_state = std::sync::Arc::new(crate::core::state::AppState {
        gemini_semaphore: std::sync::Arc::new(tokio::sync::Semaphore::new(3)),
        http_client: reqwest::Client::new(),
        server_sk_bytes: server_private_key_bytes,
        cache: local_cache,
        storage: table_engine,
        sse_tx,
        forwarder_token: std::env::var("FORWARDER_TOKEN").unwrap_or_else(|_| "abc".to_string()),
        ws_sessions: std::sync::Arc::new(dashmap::DashMap::new()),
    });
    
    let allowed_origins_str = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:8080,http://localhost:5000".to_string());
        
    let allowed_origins: Vec<HeaderValue> = allowed_origins_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(|s| {
            s.parse::<HeaderValue>().map_err(|e| {
                tracing::warn!(category = "Warning", origin = s, error = %e, "Bỏ qua cấu hình CORS origin không hợp lệ trong .env");
            }).ok()
        })
        .collect();

    let allowed_headers = [
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
        header::ACCEPT,
        HeaderName::from_static("x-requested-with"),
        HeaderName::from_static("sec-fetch-dest"),
        HeaderName::from_static("sec-fetch-mode"),
        HeaderName::from_static("cache-control"),
        HeaderName::from_static("x-auth-token"),
        HeaderName::from_static("sec-fetch-site"),
    ];

    let cors_layer = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(allowed_headers)
        .allow_credentials(true);

    let storage_path = std::env::var("STORAGE_PATH").unwrap_or_else(|_| "storages".to_string());
    if !Path::new(&storage_path).exists() {
        fs::create_dir_all(&storage_path).expect("Không tạo được folder storages");
        info!(category = "System", "Đã tạo thư mục lưu trữ: {}", storage_path);
    }
    // Đảm bảo tạo thư mục tmp
    let _ = fs::create_dir_all(Path::new(&storage_path).join(".tmp"));

    let spa_storages = ServeDir::new(&storage_path); 

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let req_id = crate::helpers::suid::generate_random_hex()[..8].to_string();
            let method = request.method().to_string();
            let uri = request.uri().to_string();
            tracing::info_span!("HTTP", id = %req_id, method = %method, uri = %uri)
        })
        .on_response(|response: &Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
            if response.headers().contains_key("x-stream-log-handled") {
                return;
            }
            let status = response.status();
            let status_code = status.as_u16();
            let reason_str = status.canonical_reason().unwrap_or("Unknown");
            let category = if status.is_server_error() || status.is_client_error() { "Error" } else { "Complete" };
            info!(
                category = category,
                status = status_code, 
                reason = reason_str,
                latency_ms = latency.as_millis(), 
                "Phản hồi HTTP"
            );
        });

    let app = Router::new()
        .route("/favicon.ico", axum::routing::get(favicon_handler))
        .nest("/api/projects", crate::routes::projects::router())
        .nest("/api/auth", crate::routes::auth::router())
        .nest_service("/storages", spa_storages)
        .fallback(static_handler)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(cors_layer)
        .layer(trace_layer)
        .with_state(app_state.clone());

    let _addr = format!("0.0.0.0:{}", port_str);
    let listener = tokio::net::TcpListener::bind(&_addr).await.unwrap();
    
    crate::infrastructure::log_compressor::start_log_compressor_task().await;
    
    info!(category = "System", "Server đang chạy tại http://{}", _addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(app_state))
        .await
        .unwrap();
}

async fn shutdown_signal(_state: std::sync::Arc<crate::core::state::AppState>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Không thể cài đặt Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Không thể cài đặt SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    warn!(category = "Warning", "Đã nhận tín hiệu tắt máy! Tiến hành đóng an toàn (Graceful Shutdown)...");
    

    info!(category = "System", "Hoàn tất dọn dẹp. Tạm biệt!");
}