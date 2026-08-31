# Smooth OLED UI (ESP32 + U8g2)

Một dự án thiết kế giao diện động (Dynamic UI) chất lượng cao cho màn hình OLED 128x64 sử dụng vi điều khiển ESP32-S3. Giao diện được thiết kế theo kiến trúc **Hybrid FSD + Atomic**, tập trung tối ưu hóa các hiệu ứng chuyển động vật lý bằng thuật toán Nội suy tuyến tính (LERP) đảm bảo tần số quét mượt mà 60FPS.

## 🚀 Tính năng nổi bật

Chương trình hoạt động dưới dạng một State Machine (Máy trạng thái) liên hoàn gồm 3 hiệu ứng phô diễn kỹ thuật:

1. **Carousel Menu (Trượt ngang):** Menu Icon 24x24 cuộn ngang với quán tính lò xo.
2. **List Popup XOR (Danh sách Dropdown):** Menu thả xuống với thanh bôi đen (Cursor) đảo màu thông minh không cần vẽ lại text (XOR Invert).
3. **Circular Reveal Side Popup (Menu cánh cung):** Giao diện chính lùi về bên trái, nhường chỗ cho một mặt nạ hình tròn mở rộng từ lề phải. Các danh sách chữ tự động cuộn dọc và uốn cong bám sát theo bán kính viền tròn thông qua tính toán định lý Pytago trực tiếp theo thời gian thực.

## 🛠️ Yêu cầu Phần cứng

- **Vi điều khiển:** ESP32-S3 (hoặc các dòng ESP32 hỗ trợ I2C cấu hình tùy chọn).
- **Màn hình:** OLED SSD1306 (Độ phân giải 128x64).
- **Pinout I2C Mặc định:** `SDA = 47`, `SCL = 48` (Có thể tùy chỉnh trong file `.ino`).

## ⚙️ Môi trường Phát triển & Build

Dự án được xây dựng bằng Arduino C++ nhưng sử dụng hệ thống Build riêng qua CLI để tối ưu hóa thời gian biên dịch và tránh lỗi chiếm dụng cổng Serial (Lock Port).

### 1. Công cụ cần thiết
- [Arduino CLI](https://arduino.github.io/arduino-cli/): Biên dịch (Compile) mã nguồn C/C++.
- Thư viện `U8g2` của oliver (Cài bằng Arduino IDE hoặc qua lệnh `arduino-cli`).
- [espflash](https://github.com/esp-rs/espflash): Công cụ nạp Firmware cực nhanh viết bằng Rust. (Có thể cài qua lệnh `cargo binstall espflash`).

### 2. Hướng dẫn Biên dịch và Nạp Code
Bạn không cần mở Arduino IDE. Chỉ cần mở Terminal (PowerShell) và chạy script đã được viết sẵn:

```powershell
.\build_flash.ps1
```

Script này sẽ tự động dọn dẹp các tiến trình Serial bị kẹt, biên dịch gia tăng (Incremental Build) dự án thông qua Arduino CLI và Flash vào vi điều khiển bằng `espflash`.

## 📐 Kiến trúc Phần mềm (Hybrid FSD)

- **State Machine:** Chia tách logic Render thành từng Atom độc lập (Carousel, List, Side Popup) giúp mã nguồn sạch và dễ bảo trì.
- **Tách biệt Logic và Render:** Mỗi Atom bao gồm một hàm `update_physics()` để tính toán vị trí, gia tốc độc lập với hàm `draw_menu()` chịu trách nhiệm đẩy Pixel ra màn hình.
- **Glassmorphism & Masking:** Ứng dụng kỹ thuật xếp lớp (Z-Index bằng thứ tự vẽ) kết hợp XOR `setDrawColor(2)` để tạo cảm giác các khối Menu nổi đè lên nhau.