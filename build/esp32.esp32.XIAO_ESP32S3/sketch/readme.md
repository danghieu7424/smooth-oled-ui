#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\readme.md"
# Smooth OLED UI (ESP32 + U8g2)

![Smooth OLED UI Demo](data/oled_capture.gif?v=2)

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

## 📚 Hướng dẫn tích hợp SmoothOLED (C++)

Thư viện được thiết kế theo hướng đối tượng, đóng gói toàn bộ logic vật lý (Lerp) và render thành class `SmoothOLED`, giúp bạn dễ dàng tích hợp vào project ESP32 bất kỳ.

### 1. Khởi tạo & Cấu hình (`setup`)
```cpp
#include <U8g2lib.h>
#include <Wire.h>
#include "SmoothOLED.h"

// 1. Khai báo màn hình U8g2
U8G2_SSD1306_128X64_NONAME_F_HW_I2C u8g2(U8G2_R0, U8X8_PIN_NONE, 48, 47);

// 2. Truyền con trỏ u8g2 và cổng Serial vào class UI (Truyền &Serial để kích hoạt tính năng PC Viewer)
SmoothOLED ui(&u8g2, &Serial); 

void setup() {
    Serial.begin(921600);
    u8g2.begin();
    
    // 3. Gán dữ liệu cho các Menu
    ui.setCarouselItems(menu_items, TOTAL_ITEMS, "< MAIN MENU >");
    ui.setPopupListItems(popup_items, TOTAL_POPUP_ITEMS);
    ui.setSidePopupItems(side_items, TOTAL_SIDE_ITEMS);

    // 4. Cấu hình tính năng mở rộng
    ui.enableAutoDemo(false); // Bật/Tắt chế độ tự động chạy Demo
    ui.enablePCViewer(true);  // Bật xuất khung hình ra Serial (Nên TẮT tính năng này khi Build Release để tối ưu hiệu năng)
    
    // 5. Khởi động UI
    ui.begin();
}
```

### 2. Vòng lặp chính & Xử lý Sự kiện (`loop`)
```cpp
void loop() {
    // 1. Gọi hàm update() liên tục (hệ thống tự giới hạn Frame Limit 60FPS bên trong)
    ui.update();

    // 2. Chuyển tiếp các tín hiệu điều khiển (từ Nút bấm vật lý hoặc Serial) vào thư viện
    if (Serial.available() > 0) {
        char c = Serial.read();
        if (c == 'U') ui.up();        // Chuyển mục lên / sang trái
        if (c == 'D') ui.down();      // Chuyển mục xuống / sang phải
        if (c == 'P') ui.openPopup(); // Mở Menu Popup (dạng Dropdown)
        if (c == 'S') ui.openSideList();// Mở Menu Side Popup (dạng Cánh cung)
        if (c == 'C') ui.closeOverlay();// Trở về Menu chính (Đóng các Popup)
        if (c == 'E') ui.select();    // Chọn mục (Enter)
    }
}
```

---

## 🎮 Công cụ Giả lập & Ghi hình (PC Viewer)

Hai tệp Script Python đính kèm đóng vai trò như một "Màn hình OLED Ảo" trên PC. Chúng nhận dữ liệu pixel thô từ cổng UART thảy ra từ ESP32 và render bằng OpenCV.
Điểm đặc biệt là cả 2 script đều được tích hợp **Bộ lắng nghe bàn phím ngầm (Global Keylogger)**, cho phép bạn điều khiển màn hình ESP32 trực tiếp bằng bàn phím máy tính/Bluetooth!

**Yêu cầu thư viện:**
```bash
pip install pyserial opencv-python numpy keyboard
```

### 1. Giám sát trực tiếp (`view.py`)
Dành cho quá trình phát triển (Debug). Tạo ra một cửa sổ mô phỏng màn hình OLED để tương tác mà không cần cúi xuống nhìn màn hình vật lý.
- **Lệnh chạy:** `py view.py`
- **Điều khiển:** Phím Mũi tên (lên/xuống/trái/phải), Home, End, Esc, Enter.

### 2. Trình Quay Video 60FPS (`render.py`)
Chức năng điều khiển y hệt `view.py`, nhưng có thêm khả năng âm thầm lưu lại mọi khung hình bạn đang thao tác để kết xuất thành video chất lượng cao (`oled_capture.mp4`).
- **Lệnh chạy:** `py render.py`
- **Điều khiển:** Sử dụng phím điều hướng tương tự như trên.
- **Kết thúc ghi hình:** Bấm phím `x` trên bàn phím máy tính hoặc bấm tắt cửa sổ. Script sẽ lập tức dừng thu và lưu file MP4 thành công!