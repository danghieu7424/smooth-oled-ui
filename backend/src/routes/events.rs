use axum::{
    extract::State,
    response::sse::{Event, Sse, KeepAlive},
    routing::{get, post},
    Json, Router,
};
use axum::http::StatusCode;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{info, warn};

use crate::core::{error::AppError, state::AppState};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BroadcastPayload {
    pub topic: String,
    pub message: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        // [GET] /api/v1/events -> Mở luồng SSE (Client subscribe vào đây)
        .route("/", get(sse_handler))
        // [POST] /api/v1/events/broadcast -> API nội bộ để nhồi Data vào luồng SSE
        .route("/broadcast", post(broadcast_handler))
}

/****
 * [GET] Thiết lập kết nối Server-Sent Events (SSE)
 * Giữ kết nối mở liên tục (Long-lived Connection). Có Keep-Alive chống rớt mạng.
 ****/
async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    info!(category = "Realtime", "Một Client mới vừa mở kết nối SSE...");

    // Lấy một thẻ (receiver) từ kênh Broadcast chung
    let rx = state.sse_tx.subscribe();

    // Biến Receiver thành một Async Stream liên tục
    let stream = BroadcastStream::new(rx)
        .filter_map(|msg| async move {
            match msg {
                Ok((topic, data)) => {
                    // Đóng gói data thành SSE Event chuẩn HTML5
                    Some(Ok(Event::default().event(topic).data(data)))
                }
                Err(e) => {
                    warn!(category = "Warning", "Lỗi luồng SSE Broadcast: {}", e);
                    None
                }
            }
        });

    // Trả về luồng SSE, thiết lập KeepAlive bắn "ping" mỗi 15 giây
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

/****
 * [POST] Phát thanh (Broadcast) dữ liệu
 * Bất kỳ ai gọi API này, hệ thống sẽ đẩy tin nhắn tới TẤT CẢ các Client đang mở SSE.
 ****/
async fn broadcast_handler(
    State(state): State<AppState>,
    Json(payload): Json<BroadcastPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    info!(category = "Realtime", "Phát sóng thông báo mới lên kênh SSE: [{}]", payload.topic);
    
    // Đẩy vào kênh Tx. Bỏ qua lỗi nếu không có ai đang nghe (Rx = 0).
    let _ = state.sse_tx.send((payload.topic.clone(), payload.message.clone()));

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "message": "Đã phát sóng tín hiệu Real-time",
            "delivered_topic": payload.topic
        }))
    ))
}
