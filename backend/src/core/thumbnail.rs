use std::path::Path;
use std::process::Command;
use tracing::{info, warn, error};

/****
 * Module: Thumbnail Generator
 * Sinh ảnh thu nhỏ (thumbnail) tối đa 240px cho các tệp hình ảnh và video.
 * Các tệp này được lưu tại `.thumbnails/`.
 ****/
pub async fn generate_thumbnail(file_path: &Path, thumb_path: &Path) -> Result<(), String> {
    if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
        let ext = ext.to_lowercase();
        if ext == "jpg" || ext == "jpeg" || ext == "png" || ext == "webp" || ext == "gif" || ext == "bmp" || ext == "ico" {
            generate_image_thumbnail(file_path, thumb_path)
        } else if ext == "mp4" || ext == "mkv" || ext == "avi" || ext == "mov" || ext == "webm" || ext == "flv" || ext == "wmv" {
            generate_video_thumbnail(file_path, thumb_path).await
        } else {
            Err("Định dạng không được hỗ trợ".into())
        }
    } else {
        Err("Không thể lấy đuôi mở rộng".into())
    }
}

fn generate_image_thumbnail(file_path: &Path, thumb_path: &Path) -> Result<(), String> {
    match image::open(file_path) {
        Ok(img) => {
            let thumbnail = img.thumbnail(240, 240);
            match thumbnail.save(thumb_path) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Lỗi lưu ảnh thu nhỏ: {}", e)),
            }
        }
        Err(e) => Err(format!("Lỗi mở ảnh: {}", e)),
    }
}

async fn generate_video_thumbnail(file_path: &Path, thumb_path: &Path) -> Result<(), String> {
    // Dùng ffmpeg lấy frame tại giây thứ 1
    // ffmpeg -i input.mp4 -ss 00:00:01.000 -vframes 1 -vf scale="240:-1" output.jpg
    let ffmpeg_cmd = std::env::var("FFMPEG_PATH").unwrap_or_else(|_| "ffmpeg".to_string());
    let output = Command::new(ffmpeg_cmd)
        .arg("-y") // Override if exists
        .arg("-i")
        .arg(file_path.to_string_lossy().as_ref())
        .arg("-ss")
        .arg("00:00:01.000")
        .arg("-vframes")
        .arg("1")
        .arg("-vf")
        .arg("scale=240:-1") // 240 width, auto height
        .arg(thumb_path.to_string_lossy().as_ref())
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(())
            } else {
                Err(format!("ffmpeg lỗi: {}", String::from_utf8_lossy(&out.stderr)))
            }
        }
        Err(e) => {
            warn!(category = "System", "Không tìm thấy lệnh ffmpeg, tính năng ảnh thu nhỏ cho video sẽ bị bỏ qua. Lỗi: {}", e);
            Err("ffmpeg không khả dụng".into())
        }
    }
}
