use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use futures_util::stream::StreamExt;
use colored::Colorize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    println!("\n{}", "🚀 BẮT ĐẦU CHẠY SCRIPT TEST REAL-TIME (SSE)".bright_cyan().bold());
    println!("{}", "=================================================".bright_cyan());

    let sse_url = "http://localhost:5000/api/v1/events";
    let broadcast_url = "http://localhost:5000/api/v1/events/broadcast";

    // 1. MỞ LUỒNG LẮNG NGHE (CLIENT) CHẠY NGẦM
    let client = Client::new();
    println!("{}", "📡 Đang mở kết nối Server-Sent Events (SSE)...".bright_yellow());
    
    let response = client.get(sse_url).send().await?;
    if !response.status().is_success() {
        println!("{}", format!("❌ Lỗi kết nối SSE: {}", response.status()).red());
        return Ok(());
    }

    println!("{}", "✅ Đã kết nối SSE thành công! Đang lắng nghe tín hiệu...".bright_green());

    // Tách việc lắng nghe ra một luồng riêng để không chặn luồng chính
    let listener_handle = tokio::spawn(async move {
        let mut byte_stream = response.bytes_stream();
        while let Some(item) = byte_stream.next().await {
            match item {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
                    for line in lines {
                        // Bỏ qua các tín hiệu Keep-Alive rỗng
                        if line.starts_with("event:") || line.starts_with("data:") {
                            println!("{} {}", "⚡ [CLIENT NHẬN]:".bright_magenta().bold(), line.bright_white());
                        }
                    }
                }
                Err(e) => {
                    println!("{} {}", "❌ Lỗi đọc stream:".red(), e);
                    break;
                }
            }
        }
    });

    // Chờ 2 giây để chắc chắn kết nối đã ổn định
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 2. PHÁT SÓNG THÔNG BÁO (BROADCAST) TỪ LUỒNG CHÍNH
    println!("\n{}", "📢 Bắt đầu phát sóng tín hiệu (Broadcast)...".bright_yellow());
    
    let messages = vec![
        ("system_alert", "Hệ thống chuẩn bị bảo trì trong 5 phút nữa!"),
        ("price_update", "Giá Bitcoin vừa vượt mốc 100,000 USD!"),
    ];

    for (topic, msg) in messages {
        println!("{} Gửi: [{}] {}", "📤 [SERVER ĐẨY]:".bright_cyan().bold(), topic, msg);
        let _ = client.post(broadcast_url)
            .json(&json!({ "topic": topic, "message": msg }))
            .send().await?;
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // Chờ thêm 2 giây để luồng ngầm kịp in ra màn hình các tin nhắn nhận được
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Hủy luồng nghe (Vì SSE là vô tận)
    listener_handle.abort();

    println!("\n{}", "✅ HOÀN TẤT KỊCH BẢN TEST REAL-TIME!".bright_green().bold());
    
    Ok(())
}
