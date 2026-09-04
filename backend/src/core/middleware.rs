use axum::{
    extract::{State, Request},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use crate::core::{state::AppState, error::AppError};

/// Middleware chống Spam & DDoS
/// Giới hạn 200 Requests / Phút / 1 IP
pub async fn rate_limiter(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Bỏ qua rate limit cho API upload do cơ chế upload chunking gọi rất nhiều request liên tục
    if request.uri().path().ends_with("/upload") {
        return Ok(next.run(request).await);
    }

    // 1. Xác định IP của người dùng (Hỗ trợ proxy Cloudflare/Nginx)
    let ip = headers
        .get("cf-connecting-ip")
        .and_then(|h| h.to_str().ok())
        .or_else(|| headers.get("x-forwarded-for").and_then(|h| h.to_str().ok()))
        .unwrap_or("unknown_ip")
        .split(',')
        .next()
        .unwrap_or("unknown_ip")
        .trim();

    // 2. Tạo khóa định danh cho IP này
    let cache_key = format!("rate_limit:{}", ip);

    // 3. Tăng bộ đếm (Atomic Increment) với vòng đời 60 giây (1 phút)
    let count = state.cache.increment(&cache_key, 60).await;

    // 4. Kiểm tra giới hạn (Max 2000 request / phút)
    if count > 2000 {
        return Err(AppError::TooManyRequests);
    }

    // 5. Nếu an toàn, cho phép Request đi tiếp vào Router bên trong
    Ok(next.run(request).await)
}
