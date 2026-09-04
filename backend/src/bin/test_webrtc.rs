use colored::Colorize;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    println!("\n{}", "🚀 BẮT ĐẦU TEST WEBRTC SIGNALING SERVER (P2P)".bright_cyan().bold());
    println!("{}", "=======================================================".bright_cyan());

    // --- PEER A (Alice) ---
    let (alice_ws, _) = connect_async("ws://localhost:5000/api/v1/webrtc/Alice").await?;
    let (mut alice_sender, mut alice_receiver) = alice_ws.split();
    println!("{} Alice đã kết nối Trạm Signaling", "📡 [Alice]".bright_green());

    // --- PEER B (Bob) ---
    let (bob_ws, _) = connect_async("ws://localhost:5000/api/v1/webrtc/Bob").await?;
    let (mut bob_sender, mut bob_receiver) = bob_ws.split();
    println!("{} Bob đã kết nối Trạm Signaling", "📡 [Bob]".bright_blue());

    // Luồng nghe của Alice
    let alice_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = alice_receiver.next().await {
            let text_str = text.to_string();
            println!("{} Nhận: {}", "📬 [Alice Nhận]".bright_magenta(), text_str.bright_white());
        }
    });

    // Luồng nghe của Bob
    let bob_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = bob_receiver.next().await {
            let text_str = text.to_string();
            println!("{} Nhận: {}", "📬 [Bob Nhận]".bright_cyan(), text_str.bright_white());
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 1. Alice gửi Offer cho Bob
    let offer = r#"{"type":"offer","sdp":"v=0\r\no=alice...","target_id":"Bob","sender_id":"Alice"}"#;
    println!("\n{} Đang gửi SDP Offer cho Bob...", "📤 [Alice Gửi]".bright_green());
    alice_sender.send(Message::Text(offer.to_string().into())).await?;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 2. Bob gửi Answer cho Alice
    let answer = r#"{"type":"answer","sdp":"v=0\r\no=bob...","target_id":"Alice","sender_id":"Bob"}"#;
    println!("\n{} Đang gửi SDP Answer cho Alice...", "📤 [Bob Gửi]".bright_blue());
    bob_sender.send(Message::Text(answer.to_string().into())).await?;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 3. Trao đổi ICE Candidates
    let ice_a = r#"{"type":"ice_candidate","candidate":"candidate:1 1 UDP...","target_id":"Bob","sender_id":"Alice"}"#;
    println!("\n{} Đang gửi ICE Candidate cho Bob...", "📤 [Alice Gửi]".bright_green());
    alice_sender.send(Message::Text(ice_a.to_string().into())).await?;

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Đóng kết nối
    println!("\n{}", "👋 Kết thúc kịch bản kết nối!".bright_yellow());
    let _ = alice_sender.send(Message::Close(None)).await;
    let _ = bob_sender.send(Message::Close(None)).await;

    let _ = alice_task.await;
    let _ = bob_task.await;

    println!("\n{}", "✅ HOÀN TẤT BÀI KIỂM TRA P2P SIGNALING!".bright_cyan().bold());
    Ok(())
}
