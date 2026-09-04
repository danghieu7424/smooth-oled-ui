use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::core::state::AppState;

/****
 * Module: WebRTC Signaling Router
 * Chức năng: Điều phối các bản tin SDP (Offer/Answer) và ICE Candidates 
 * giữa 2 Client muốn gọi Video/Voice cho nhau (Peer-to-Peer).
 *
 * TỐI ƯU HÓA: Thay vì ném vào kênh Broadcast toàn cục, ta sử dụng DashMap
 * (ws_sessions) để chuyển tin nhắn theo mô hình Point-to-Point (Điểm - Điểm).
 * Tránh đánh thức hàng ngàn connection không liên quan!
 ****/

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")] // Dùng enum tag để phân biệt loại tin nhắn trong JSON
pub enum RtcMessage {
    #[serde(rename = "offer")]
    Offer { sdp: String, target_id: String, sender_id: String },
    
    #[serde(rename = "answer")]
    Answer { sdp: String, target_id: String, sender_id: String },
    
    #[serde(rename = "ice_candidate")]
    IceCandidate { candidate: String, target_id: String, sender_id: String },
}

pub fn router() -> Router<AppState> {
    // Client gọi: ws://localhost:5000/api/v1/webrtc/{user_id_cua_minh}
    Router::new().route("/:user_id", get(signaling_handler))
}

async fn signaling_handler(
    ws: WebSocketUpgrade,
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, user_id, state))
}

async fn handle_socket(socket: WebSocket, user_id: String, state: AppState) {
    info!(category = "WebRTC", "Trạm Signaling: Peer [{}] đã kết nối", user_id);

    let (mut sender, mut receiver) = socket.split();
    
    // Tạo 1 ống hút tin nhắn riêng (Point-to-Point) cho Client này
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    // Lưu ống hút vào Kho chứa dùng chung (DashMap cực nhanh)
    state.ws_sessions.insert(user_id.clone(), tx);

    let state_clone = state.clone();
    let user_id_clone = user_id.clone();

    // -----------------------------------------------------
    // LUỒNG 1: Nhận tin nhắn TỪ CÁC PEER KHÁC chuyển tới
    // -----------------------------------------------------
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            // Ném thẳng xuống Client
            if sender.send(Message::Text(msg.clone())).await.is_err() {
                warn!(category = "WebRTC", "Lỗi gửi tin xuống Peer {}", user_id_clone);
                break;
            }
        }
    });

    // -----------------------------------------------------
    // LUỒNG 2: Nghe tin nhắn TỪ CLIENT NÀY và điều phối đi
    // -----------------------------------------------------
    let user_id_clone2 = user_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Thử Parse xem đúng chuẩn WebRTC không
                if let Ok(rtc_msg) = serde_json::from_str::<RtcMessage>(&text) {
                    let target = match &rtc_msg {
                        RtcMessage::Offer { target_id, .. } => target_id,
                        RtcMessage::Answer { target_id, .. } => target_id,
                        RtcMessage::IceCandidate { target_id, .. } => target_id,
                    };

                    // TÌM ĐÍCH ĐẾN BẰNG DASHMAP (O(1) - Lock-free Sharded)
                    if let Some(target_tx) = state_clone.ws_sessions.get(target) {
                        let _ = target_tx.send(text).await;
                        info!(category = "WebRTC", "[Định tuyến] {} -> {}", user_id_clone2, target);
                    } else {
                        warn!(category = "WebRTC", "Bị rơi tin! Không tìm thấy Peer đích: {}", target);
                    }
                }
            }
        }
    });

    // Kéo 1 trong 2 chết thì chết chùm (Bảo vệ RAM)
    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    };

    // Khi thoát, phải tự tay xóa khỏi danh bạ DashMap
    state.ws_sessions.remove(&user_id);
    info!(category = "WebRTC", "Peer [{}] đã thoát khỏi Trạm Signaling", user_id);
}
