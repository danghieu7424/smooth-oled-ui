use colored::Colorize;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    println!("\n{}", "🚀 BẮT ĐẦU CHẠY SCRIPT TEST WEBSOCKET (WS)".bright_cyan().bold());
    println!("{}", "=================================================".bright_cyan());

    let ws_url = "ws://localhost:5000/api/v1/ws";
    
    println!("{}", "📡 Đang mở kết nối WebSocket...".bright_yellow());
    
    let (ws_stream, response) = match connect_async(ws_url).await {
        Ok((stream, res)) => (stream, res),
        Err(e) => {
            println!("{} {}", "❌ Lỗi kết nối WS (Server đã chạy chưa?):".red(), e);
            return Ok(());
        }
    };
    
    println!("{} HTTP Status: {}", "✅ Đã kết nối WS thành công!".bright_green(), response.status());

    let (mut sender, mut receiver) = ws_stream.split();

    // 1. Luồng chạy ngầm: Lắng nghe bất kỳ tin nhắn nào từ Server ném xuống
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let text_str = text.to_string();
                    println!("{} {}", "⚡ [CLIENT NHẬN TỪ SERVER]:".bright_magenta().bold(), text_str.bright_white());
                }
                Ok(Message::Close(_)) => {
                    println!("{}", "👋 Server chủ động đóng kết nối".yellow());
                    break;
                }
                _ => {} // Bỏ qua Ping/Pong
            }
        }
    });

    // 2. Chờ 1 chút rồi bắn thử vài tin nhắn Chat lên Server
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    
    let chat_msgs = vec![
        "Xin chào Server, tôi là WebSocket Client đây!",
        "Hãy phản hồi lại tin nhắn này nhé!",
    ];

    for msg in chat_msgs {
        println!("{} {}", "📤 [CLIENT GỬI LÊN SERVER]:".bright_cyan().bold(), msg);
        sender.send(Message::Text(msg.to_string().into())).await?;
        
        // Chờ 1 giây để xem Server có dội lại (Echo) kịp không
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    // 3. Đợi thêm 2 giây. 
    // Trong lúc này nếu ai đó gọi API POST /api/v1/events/broadcast, 
    // luồng WS này cũng sẽ chộp được luôn vì Server đã hợp nhất Kênh!
    println!("\n{}", "⏳ Chờ 3 giây để bắt Event Broadcast từ SSE (nếu có)...".bright_black());
    
    // Gửi thử 1 tín hiệu qua HTTP REST (Giả lập hệ thống khác gọi hàm Broadcast)
    let http_client = reqwest::Client::new();
    let _ = http_client.post("http://localhost:5000/api/v1/events/broadcast")
        .json(&serde_json::json!({
            "topic": "ws_integration",
            "message": "Tin nhắn này gửi từ REST API nhưng sẽ chui vào ống WebSocket!"
        }))
        .send().await;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // 4. Đóng kết nối đàng hoàng (Close Handshake)
    println!("\n{}", "👋 Đóng kết nối WebSocket...".bright_yellow());
    let _ = sender.send(Message::Close(None)).await;

    let _ = recv_task.await; // Chờ luồng nghe kết thúc hẳn

    println!("\n{}", "✅ HOÀN TẤT KỊCH BẢN TEST WEBSOCKET!".bright_green().bold());
    Ok(())
}
