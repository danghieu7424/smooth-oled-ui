# TÀI LIỆU KIẾN TRÚC HỆ THỐNG - FACE ROBOT ESP32-S3
*(Tài liệu này đóng vai trò như một Single Source of Truth cho toàn bộ dự án)*

## 1. TỔNG QUAN KIẾN TRÚC (HYBRID FSD + ATOMIC)
Dự án được xây dựng dựa trên nguyên lý chia rẽ mối bận tâm (Separation of Concerns) để đảm bảo ESP32-S3 hoạt động ở hiệu suất tối đa (20+ FPS) trong khi vẫn chạy được thuật toán Trí Tuệ Nhân Tạo (AI).

Hệ thống được thiết kế theo cấu trúc Modular hóa C++ đa luồng (Multi-core) độc lập:

- **Core 0 (Não bộ - `AITask` trong `Comms.cpp`):** Đảm nhiệm việc giao tiếp với ESP32 Master qua UART, đọc lệnh thay đổi cảm xúc.
- **Core 1 (Khuôn mặt - `loop()` trong `face_rbot.ino`):** Đảm nhiệm việc gọi hàm nội suy tọa độ (`updateFaceLogic()` trong `Animation.cpp`) và dựng hình Vector (`renderToScreen()`). Tuyệt đối không chứa logic đọc/ghi Blocking.

Giao tiếp giữa 2 Core được thực hiện qua biến chia sẻ nguyên tử (Atomic Shared Variable) `volatile int targetEmotionCode` được định nghĩa chung trong `FaceGlobals.h`.

---

## 2. CẤU TRÚC MODULE (C++ MULTI-FILE ARCHITECTURE)
Để tránh phình to mã nguồn (Monolithic), dự án được chia thành các module với trách nhiệm rõ ràng:

### `FaceTypes.h`
Định nghĩa cấu trúc dữ liệu cơ bản (`struct FaceState`) và toàn bộ các biểu cảm tĩnh (ví dụ: `stateNormal`, `stateHappy`, `stateSleep`).

### `FaceGlobals.h` / `FaceGlobals.cpp`
Chứa các biến toàn cục (Global Variables) cần dùng chung giữa các Module:
- `currentFace`, `targetFace` (dùng cho Lerp Animation).
- `targetEmotionCode`, `lastEmotionCode`, `lastInteractionTime` (dùng cho State Machine).
- `susWeight`, `blinkFactor` (dùng cho ghi đè Animation).

### `Display.h` / `Display.cpp`
Tầng trừu tượng phần cứng (Hardware Abstraction Layer).
- Khởi tạo thư viện `LovyanGFX`, gán cấu hình cho SPI và màn hình ST7789.
- Chứa thuật toán vẽ cơ bản cốt lõi `drawGradientAsymmetricRect` (Concentric Layering).

### `Animation.h` / `Animation.cpp`
Tầng Logic Animation và VFX.
- `updateFaceLogic()`: Tính toán nội suy (Linear Interpolation - Lerp) để khung hình chuyển động mượt mà.
- `drawEye()`: Ghép các vòng Rect lại để tạo thành đôi mắt 3D nổi khối.
- `renderToScreen()`: Tổng hợp Mắt và Miệng, đồng thời thêm các dao động Toán học (Sóng Sin/Cos) để tạo hiệu ứng thở, ngáp, khóc, chóng mặt.

### `Comms.h` / `Comms.cpp`
Tầng Giao tiếp.
- Chứa `AITask` chạy độc lập trên Core 0, lắng nghe Serial và cập nhật trạng thái mục tiêu.

### `AI_Logic.h` / `AI_Logic.cpp`
Tầng Trí tuệ Nhân tạo (Tabular Q-Learning). Đóng gói thuật toán Bellman để máy học từ Cảm biến giả lập. (Mã chờ mở rộng cho tính năng Reinforcement Learning).

### `face_rbot.ino`
Điểm neo (Entry point) duy nhất:
- `#include` các Module trên.
- Khởi tạo `setup()` (gọi `tft.init()` và `xTaskCreatePinnedToCore`).
- Xử lý `loop()`:
  - Logic **Timeout Nhàn rỗi**: Rảnh quá 15s -> Ngó nghiêng xung quanh.
  - Logic **Timeout Ngủ gật**: Ngó nghiêng quá 10 phút (600s) -> Nhắm mắt đi ngủ.
  - Điều phối `updateFaceLogic()` và `renderToScreen()`.

---

## 3. NHỮNG THÀNH TỰU KỸ THUẬT CỐT LÕI (CORE IMPLEMENTATIONS)

### 3.1. Đồ họa Đa lớp Đồng tâm (Concentric Layering)
* **Vấn đề:** Không thể dùng hàm vẽ Sprite mờ lề (Anti-aliasing) hoặc Easing Alpha của LovyanGFX vì ngốn RAM và giảm FPS trầm trọng.
* **Giải pháp:** Sử dụng thuật toán **Concentric Layering (Các lớp đồng tâm)** thông qua hàm tự viết `drawGradientAsymmetricRect()`. 
* **Cách hoạt động:** Dùng Scanline Rasterization kết hợp công thức phương trình Elip `x = r - r * sqrt(1 - (dy/r)^2)` để vẽ các đường `drawFastHLine`. Hình ảnh có cảm giác 3D nổi khối và viền siêu mượt do sự hòa trộn màu gradient theo chiều dọc (VGradient).

### 3.2. Hiệu ứng Hoạt họa Động (Dynamic VFX)
Thay vì sử dụng Sprite chuyển động truyền thống tốn kém bộ nhớ, các hiệu ứng sinh động được tính toán bằng Toán học (Sóng Sin/Cos) ngay trong quá trình Render:
* **Ngủ (Sleep):** Mí mắt từ từ sụp xuống thông qua phép nội suy bậc nhất (Linear Interpolation) dựa theo % thời gian kết hợp với sóng Sin tần số siêu nhỏ (`0.008`) để giả lập 1-2 lần chép miệng thèm ăn trong khi đang ngái ngủ.
* **Chóng mặt (Dizzy):** Cộng thêm đồ thị Hình Sin/Cos (`sin/cos`) vào `offsetX/offsetY` để khiến con mắt xoay mòng mòng liên tục.
* **Nháy mắt (Wink):** Ghi đè chỉ số `blinkFactor` riêng biệt cho từng mắt tại thời điểm vẽ (Mắt trái khép tịt `0.05f`, mắt phải giữ nguyên).

### 3.3. Sửa Lỗi Hoán Đổi Byte (Endianness Mismatch) của SPI DMA
* **Vấn đề:** Khi định nghĩa màu Xanh Lá (Green), màn hình lại hiển thị màu Xanh Dương (Blue) hoặc Tím (Magenta).
* **Nguyên nhân:** CPU ESP32 sử dụng Little-Endian, nhưng chuẩn SPI của màn hình LCD lại là Big-Endian. Khi thư viện DMA đẩy 16-bit màu (RGB565) qua dây cáp, Byte thấp và Byte cao bị đảo ngược. Điều này khiến dải màu Xanh Lá (nằm ở giữa 6-bit) bị cắt đôi và phân bổ lộn xộn vào kênh Đỏ (Red) và Xanh Dương (Blue).
* **Giải pháp RALL (Phải Tuân Thủ):** 
  - Khai báo mọi màu sắc gốc dưới dạng **HEX RGB888** (Ví dụ: `0x00DC00` cho Xanh Lá).
  - Đưa màu vào hàm `lerpColor()`. Bước cuối cùng của hàm này **BẮT BUỘC** phải có thao tác hoán đổi bit (Bitwise Swap).

---

## 4. QUY TẮC CỨNG DÀNH CHO BẤT KỲ AI ĐỌC/SỬA MÃ NGUỒN (GUARDRAILS)

Nếu bạn là tôi trong tương lai hoặc một AI Sparing Partner khác, bạn **BẮT BUỘC PHẢI TUÂN THỦ** các nguyên tắc sau:

1. **KHÔNG BLOCKING TRÊN CORE 1:** Mọi thao tác chờ đợi (delay, kết nối WiFi) phải đưa vào Task của Core 0 (`Comms.cpp`). Core 1 (`loop()`) chỉ được quyền điều phối Animation.
2. **KIẾN TRÚC MÔ ĐUN:** Không được nhét thêm logic vào file `face_rbot.ino`. Nó chỉ là file điều phối (Entry point). Khi thêm tính năng, phải xác định xem nó thuộc về `Display` (Hiển thị), `Animation` (Chuyển động), hay `AI` (Tính toán).
3. **MẮT XÍCH MÀU SẮC:** Khi đổi màu giao diện, phải tuân thủ chuẩn RGB888 và thay đổi ở mảng định nghĩa màu cục bộ. Không được hardcode màu sắc rác trong các thân hàm vẽ.
4. **COMMENT THEO NGUYÊN TẮC "WHY":** Mọi hàm quan trọng phải có Block Comment giải thích tại sao lại viết như vậy, phục vụ nghiệp vụ gì. Không giải thích cú pháp C++.
5. **THÁI ĐỘ MÃ NGUỒN:** Cấm sử dụng dấu ba chấm `...` khi cung cấp đoạn mã để sửa đổi. Sửa phần nào thì cung cấp trọn vẹn Logic khối (Block) đó.

> *Tài liệu này được tạo ra để lưu giữ tư duy kiến trúc ban đầu. Hãy trân trọng và đừng phá vỡ nó.*
