use reqwest::Client;
use colored::Colorize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    println!("\n{}", "🚀 BẮT ĐẦU TEST BẢO MẬT JSPB (GOOGLE ANTI-HIJACKING)".bright_cyan().bold());
    println!("{}", "=======================================================".bright_cyan());

    let url = "http://localhost:5000/demo/jspb";
    let client = Client::new();

    println!("{} GET {}", "📡 Requesting:".bright_yellow(), url);
    let res = client.get(url).send().await?;
    
    println!("{} HTTP {}", "✅ Status:".bright_green(), res.status());
    
    // Lấy chuỗi thô (Raw Body)
    let raw_body = res.text().await?;
    
    println!("\n{}", "📦 RAW BODY (CHƯA QUA XỬ LÝ):".bright_magenta().bold());
    println!("{}", raw_body.bright_black());

    println!("\n{}", "🛠️  PHÂN TÍCH BẢO MẬT:".bright_yellow().bold());
    
    // 1. Kiểm tra tiền tố chống Hijacking
    if raw_body.starts_with(")]}',\n") {
        println!("{} Tiền tố `)]}}',\\n` đã chặn đứng thành công các thẻ <script> từ chối nhận diện đây là JSON!", "🛡️  [Pass]".bright_green());
        
        // 2. Bóc tách JSON an toàn
        let json_part = raw_body.replace(")]}',\n", "");
        println!("\n{} Dữ liệu sau khi cắt tiền tố an toàn:", "🔍 [Decode]".bright_cyan());
        println!("{}", json_part.bright_white());

        // 3. Phân tích Array-of-Arrays (JSPB Serialization)
        if let Ok(json_arr) = serde_json::from_str::<serde_json::Value>(&json_part) {
            if json_arr.is_array() {
                println!("\n{} Dữ liệu là MẢNG (Array-of-Arrays) giống Google thay vì Object! Tiết kiệm băng thông cực mạnh.", "🗜️  [Pass]".bright_green());
                
                let arr = json_arr.as_array().unwrap();
                println!("    - ID: {}", arr[0]);
                println!("    - Tên: {}", arr[1]);
                println!("    - Quyền hạn: {}", arr[2]);
            } else {
                println!("❌ Dữ liệu không phải là Mảng!");
            }
        }
    } else {
        println!("{} Không tìm thấy tiền tố chống Hijacking của Google!", "❌ [Fail]".bright_red());
    }

    println!("\n{}", "✅ HOÀN TẤT BÀI KIỂM TRA!".bright_green().bold());
    Ok(())
}
