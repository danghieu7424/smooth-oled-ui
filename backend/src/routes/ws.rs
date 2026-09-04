use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use tracing::{info, warn};

use crate::core::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // [GET] /api/v1/ws -> Yêu cầu nâng cấp (Upgrade) lên WebSocket
        .route("/", get(ws_handler))
}

/****
 * [GET] Hàm mồi (Handshake)
 * Bắt lấy yêu cầu nâng cấp HTTP và chuyển giao cho hàm xử lý WebSocket thực thụ.
 ****/
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    info!(category = "Realtime", "Có yêu cầu mở kết nối WebSockets...");
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/****
 * Logic Xử lý Cốt lõi của 1 Client WebSocket
 * Được thiết kế theo mô hình Hợp nhất Kênh (Event Bus) chạy 2 luồng song song bằng tokio::select!
 ****/
async fn handle_socket(socket: WebSocket, state: AppState) {
    info!(category = "Realtime", "WebSockets đã kết nối thành công!");

    // Tách ổ cắm (Socket) làm 2 nửa: Một nửa để Gửi (Sender), một nửa để Nhận (Receiver)
    let (mut sender, mut receiver) = socket.split();

    // Mở một máy thu (Receiver) vào kênh Broadcast của hệ thống (chung với SSE)
    let mut rx = state.sse_tx.subscribe();

    // Luồng 1: Lắng nghe Client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    info!(category = "Realtime", "Client WS nói: {}", text);
                    
                    // Tuyệt chiêu: Client này nhắn tin, ta ném thẳng tin nhắn đó vào Kênh Broadcast chung.
                    // Việc này sẽ làm TẤT CẢ SSE Client và WS Client khác đều nhận được!
                    let _ = state.sse_tx.send(("ws_chat".to_string(), text));
                }
                Message::Close(c) => {
                    if let Some(cf) = c {
                        info!(category = "Realtime", "Client đóng kết nối WS: {} - {}", cf.code, cf.reason);
                    } else {
                        info!(category = "Realtime", "Client đóng kết nối WS không rõ lý do.");
                    }
                    break;
                }
                _ => {} // Bỏ qua Ping/Pong/Binary trong phiên bản Demo này
            }
        }
    });

    // Luồng 2: Lắng nghe Kênh Broadcast (SSE_TX)
    let mut send_task = tokio::spawn(async move {
        // Bất cứ khi nào Kênh chung có tín hiệu (Từ SSE, hoặc từ WS khác ném vào)
        while let Ok((topic, data)) = rx.recv().await {
            let payload = format!("[Tín hiệu Mạng: {}] {}", topic, data);
            
            // Bắn thẳng xuống Client này
            if sender.send(Message::Text(payload)).await.is_err() {
                warn!(category = "Realtime", "Lỗi gửi tin nhắn WS, có thể Client đã ngắt kết nối.");
                break;
            }
        }
    });

    // Trình quản lý vòng đời: Nếu 1 trong 2 luồng chết (Ví dụ Client tắt trình duyệt -> Luồng 1 chết)
    // Thì luồng còn lại cũng phải bị ép chết theo (abort) để dọn rác RAM.
    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    };

    info!(category = "Realtime", "WebSockets đã đóng hoàn toàn và giải phóng RAM.");
}
