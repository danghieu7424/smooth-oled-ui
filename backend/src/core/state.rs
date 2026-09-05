/****
 * Module: Core State
 * Chức năng: Lưu trữ trạng thái toàn cục của ứng dụng (Shared State) dùng chung cho tất cả các request/route.
 * Biến đầu vào: DB Pool (Optional), Gemini Semaphore, Http Client, Crypto Keys...
 ****/

use std::sync::Arc;
use tokio::sync::Semaphore;
use dashmap::DashMap;

#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub gemini_semaphore: Arc<Semaphore>,
    pub http_client: reqwest::Client,
    pub server_sk_bytes: [u8; 32],
    pub cache: crate::infrastructure::cache::LocalCache,
    pub storage: crate::infrastructure::storage::TableEngine,
    pub sse_tx: tokio::sync::broadcast::Sender<(String, String)>,
    pub forwarder_token: String,
    pub ws_sessions: Arc<DashMap<String, tokio::sync::mpsc::Sender<String>>>,
}
