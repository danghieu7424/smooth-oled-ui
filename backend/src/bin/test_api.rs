use reqwest::Client;
use serde_json::json;
use colored::Colorize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Để màu sắc hiển thị tốt trên Windows Terminal
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let client = Client::new();
    let base_url = "http://localhost:5000/api/v1/users"; // Cổng 5000 theo cấu hình main.rs

    println!("\n{}", "🚀 BẮT ĐẦU CHẠY SCRIPT TEST API TỰ ĐỘNG (CRUD)".bright_cyan().bold());
    println!("{}", "=================================================".bright_cyan());

    // 1. Test GET (Lấy danh sách)
    println!("\n{}", "[1] Đang gọi GET /api/v1/users...".bright_yellow());
    let res = client.get(base_url).send().await?;
    println!("Status: {}", res.status());
    println!("Body: {}", res.text().await?);

    // 2. Test POST (Tạo mới)
    println!("\n{}", "[2] Đang gọi POST /api/v1/users...".bright_yellow());
    let res = client.post(base_url)
        .json(&json!({
            "name": "Super Admin",
            "email": "admin@example.com"
        }))
        .send().await?;
    println!("Status: {}", res.status());
    println!("Body: {}", res.text().await?);

    // 3. Test GET One (Lấy chi tiết)
    println!("\n{}", "[3] Đang gọi GET /api/v1/users/123...".bright_yellow());
    let res = client.get(format!("{}/123", base_url)).send().await?;
    println!("Status: {}", res.status());
    println!("Body: {}", res.text().await?);

    // 4. Test Error 404 (Bẫy lỗi)
    println!("\n{}", "[4] Đang gọi GET /api/v1/users/0 (Test Error 404)...".bright_yellow());
    let res = client.get(format!("{}/0", base_url)).send().await?;
    println!("Status: {}", res.status().as_u16().to_string().bright_red());
    println!("Body: {}", res.text().await?);

    // 5. Test PATCH (Cập nhật một phần)
    println!("\n{}", "[5] Đang gọi PATCH /api/v1/users/123...".bright_yellow());
    let res = client.patch(format!("{}/123", base_url))
        .json(&json!({ "name": "Admin Pro Max" }))
        .send().await?;
    println!("Status: {}", res.status());
    println!("Body: {}", res.text().await?);

    // 6. Test DELETE (Xóa)
    println!("\n{}", "[6] Đang gọi DELETE /api/v1/users/123...".bright_yellow());
    let res = client.delete(format!("{}/123", base_url)).send().await?;
    println!("Status: {}", res.status());
    println!("Body: {}", res.text().await?);

    println!("\n{}", "✅ HOÀN TẤT KỊCH BẢN TEST!".bright_green().bold());
    
    Ok(())
}
