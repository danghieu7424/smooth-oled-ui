#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\render_oled.md"

### 2. Xuất Framebuffer qua Cổng Nối tiếp (UART Streamer)

Nếu bắt buộc phải chạy code trên vi điều khiển thực tế (ESP32) để lấy dữ liệu từ cảm biến thực:

* **Nguyên lý:** Mỗi chu kỳ sau khi vẽ xong mảng `buffer[1024]` (128x64 bit), MCU vừa gửi dữ liệu tới OLED qua I2C/SPI, vừa đẩy 1024 bytes này qua cổng UART (tốc độ cao $921600\text{ baud}$ hoặc $2000000\text{ baud}$) về máy tính.
* **Xử lý trên PC:** Viết một script Python ngắn dùng thư viện `Pygame` hoặc `OpenCV` để đọc 1024 bytes từ cổng COM, tái tạo lại ảnh đen trắng và lưu trực tiếp thành file video `.mp4` hoặc chuỗi `.png` lossless.

```python
# Script Python mẫu nhận UART và ghi video lossless (PC side)
import serial, cv2, numpy as np

ser = serial.Serial('COM3', 921600)
out = cv2.VideoWriter('oled_capture.mp4', cv2.VideoWriter_fourcc(*'mp4v'), 60, (512, 256))

while True:
    data = ser.read(1024)
    if len(data) == 1024:
        # Chuyển 1024 bytes 1-bit thành ma trận ảnh 128x64
        bits = np.unpackbits(np.frombuffer(data, dtype=np.uint8))
        frame = (bits.reshape((64, 128)) * 255).astype(np.uint8)
        # Phóng to 4x (512x256) với bộ lọc Nearest Neighbor để giữ cạnh pixel sắc nét
        frame_scaled = cv2.resize(frame, (512, 256), interpolation=cv2.INTER_NEAREST)
        frame_bgr = cv2.cvtColor(frame_scaled, cv2.COLOR_GRAY2BGR)
        out.write(frame_bgr)

```

