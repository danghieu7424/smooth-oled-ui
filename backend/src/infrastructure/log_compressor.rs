// src/infrastructure/log_compressor.rs
#![allow(dead_code)]
use std::path::PathBuf;
use std::time::{SystemTime, Duration};
use tokio::fs;
use tokio::io::{BufReader, BufWriter};
use async_compression::tokio::write::GzipEncoder;
use tracing::{info, error, instrument};

#[instrument(skip_all)]
pub async fn start_log_compressor_task() {
    // Cài đặt đồng hồ báo thức: 12 giờ (để check thường xuyên hơn)
    let mut interval = tokio::time::interval(Duration::from_secs(12 * 3600));
    
    // Tách hẳn một luồng ngầm chạy vĩnh viễn cùng vòng đời Server
    tokio::spawn(async move {
        loop {
            // Ngủ chờ đến lần báo thức tiếp theo
            interval.tick().await;
            
            info!(category = "System", "Bắt đầu quét, nén file log > 7 ngày và xóa file > 30 ngày...");
            if let Err(e) = manage_old_logs().await {
                error!(category = "Error", error_detail = ?e, "Lỗi trong quá trình dọn dẹp log");
            }
        }
    });
}

async fn manage_old_logs() -> std::io::Result<()> {
    let log_dir = "logs";
    
    // Nếu chưa có thư mục log, bỏ qua
    if !tokio::fs::try_exists(log_dir).await.unwrap_or(false) {
        return Ok(());
    }

    let mut entries = fs::read_dir(log_dir).await?;
    let now = SystemTime::now();
    let seven_days_ago = now - Duration::from_secs(7 * 24 * 3600);
    let thirty_days_ago = now - Duration::from_secs(30 * 24 * 3600);

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        // Chỉ xử lý file (bỏ qua folder)
        if path.is_dir() {
            continue;
        }

        let is_gz = path.extension().and_then(|e| e.to_str()) == Some("gz");
        let metadata = entry.metadata().await?;
        
        if let Ok(modified_time) = metadata.modified() {
            let path_clone = path.clone();
            let file_name = path_clone.file_name().unwrap_or_default().to_string_lossy().to_string();
            
            if is_gz {
                // Xóa VĨNH VIỄN các file .gz cũ hơn 30 ngày để chống đầy đĩa
                if modified_time < thirty_days_ago {
                    info!(category = "System", file = %file_name, "File nén đã quá 30 ngày. Đang xóa vĩnh viễn...");
                    if let Err(e) = fs::remove_file(&path).await {
                        error!(category = "Error", file = %file_name, "Không thể xóa file rác: {}", e);
                    } else {
                        info!(category = "Complete", file = %file_name, "Đã xóa vĩnh viễn log cũ!");
                    }
                }
            } else {
                // Nén các file .log / .txt cũ hơn 7 ngày (Sử dụng 100% Async I/O)
                if modified_time < seven_days_ago {
                    info!(category = "System", file = %file_name, "Phát hiện log cũ, chuẩn bị nén Async...");
                    
                    if compress_file_async(&path_clone).await.is_ok() {
                        let _ = fs::remove_file(&path).await; 
                        info!(category = "Complete", file = %file_name, "Đã nén thành .gz và xóa bản gốc");
                    }
                }
            }
        }
    }
    Ok(())
}

// Hàm này chạy trên Tokio Runtime bằng 100% Async I/O, sử dụng Buffer 64KB
async fn compress_file_async(path: &PathBuf) -> std::io::Result<()> {
    let input_file = fs::File::open(path).await?;
    let mut reader = BufReader::with_capacity(64 * 1024, input_file); // Đệm 64KB để đọc nhanh
    
    let gz_path = format!("{}.gz", path.display());
    let output_file = fs::File::create(gz_path).await?;
    let writer = BufWriter::with_capacity(64 * 1024, output_file); // Đệm 64KB để ghi nhanh
    
    // GzipEncoder tự động nén dữ liệu đẩy qua nó
    let mut encoder = GzipEncoder::new(writer);
    
    // Stream dữ liệu qua bộ nén (Không dùng CPU Block)
    tokio::io::copy(&mut reader, &mut encoder).await?;
    
    use tokio::io::AsyncWriteExt;
    encoder.shutdown().await?; // Kết thúc file nén
    
    Ok(())
}