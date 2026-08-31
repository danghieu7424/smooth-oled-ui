#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\architecture.md"
[HW-ELEC]

SDA: **47**

SCL: **48**


Qua phân tích video, hệ thống UI này được xây dựng trên nền tảng **NWatch** kết hợp các hiệu ứng nội suy mượt mà từ mã nguồn mở **TxtViewer** (phổ biến trong cộng đồng DIY Smartwatch).

---

### 1. Phân tích Các Kỹ thuật Cốt lõi Tạo nên Độ Mượt

1. **Menu trượt ngang (Carousel Icon Menu):**
* Các icon $16\times16$ hoặc $24\times24$ được định vị theo một trục tọa độ toàn cục ($X$).
* Tọa độ hiển thị của camera/khung nhìn được kéo bám theo icon được chọn bằng thuật toán quán tính/Lerp:

$$X_{current} = X_{current} + (X_{target} - X_{current}) \times K \quad (0.1 \le K \le 0.3)$$




2. **Khung chọn co giãn (Elastic Selection Box):**
* Trong danh sách dọc hoặc menu icon, khung viền focus không nhảy tức thì mà đồng thời thay đổi cả $X, Y, \text{Width}, \text{Height}$ theo công thức nội suy về kích thước và vị trí đích.


3. **Hiệu ứng chuyển cảnh (Window Transitions):**
* Khi mở/đóng sub-menu, giao diện áp dụng hiệu ứng mở rộng/thu nhỏ hình chữ nhật (Zoom Box Wipe) hoặc trượt dọc đè lên màn hình trước đó.


4. **Cơ chế Vẽ 1 Lớp (Single Full Framebuffer):**
* Toàn bộ phép toán vẽ (Icon, Text, Border) thực thi trên mảng RAM $128 \times 64 \text{ bits} = 1024\text{ Bytes}$. Không can thiệp ghi lẻ tẻ vào SSD1306, chỉ đẩy toàn bộ buffer qua SPI/I2C ở cuối frame.



---

### 2. Sơ đồ Khối Phần cứng & Giao tiếp

```
  +-------------------------------------------------------------+
  |                        ESP32 / STM32                        |
  |                                                             |
  |  +------------------+     +-------------------------------+ |
  |  |  Physics Engine  | --> |     Full Frame Buffer (RAM)   | |
  |  |  (Lerp / Easing) |     |  128 x 64 bits = 1024 Bytes   | |
  |  +------------------+     +-------------------------------+ |
  +-------------------------------------------|-----------------+
                                              | SPI (SCK/MOSI) @ 10MHz
                                              | (hoặc I2C @ 400kHz - 800kHz)
                                              v
                               +-----------------------------+
                               |     OLED Module (SSD1306)   |
                               |    128x64 0.96" / 1.3"      |
                               +-----------------------------+

```

---

### 3. Mã Nguồn Mẫu (ESP32 / Arduino C++ - Chuẩn NWatch/TxtViewer Carousel)

Đoạn code dưới đây triển khai menu trượt ngang biểu tượng động với thuật toán nội suy Lerp hoàn chỉnh, có thể nạp trực tiếp qua Arduino IDE / PlatformIO:

```cpp
#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>

// Khởi tạo SSD1306 I2C chế độ Full Buffer
U8G2_SSD1306_128X64_NONAME_F_HW_I2C u8g2(U8G2_R0, /* reset=*/ U8X8_PIN_NONE);

// Khai báo Icon mẫu 16x16 (XBM Format)
static const unsigned char icon_alarm[] U8X8_PROGMEM = {
  0x00, 0x00, 0x81, 0x81, 0xc3, 0xc3, 0x66, 0x66, 0x3c, 0x3c, 0x18, 0x18, 
  0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3c, 0x3c, 0x66, 0x66, 0xc3, 0xc3, 
  0x81, 0x81, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

static const unsigned char icon_settings[] U8X8_PROGMEM = {
  0x00, 0x00, 0x81, 0x81, 0x42, 0x42, 0x24, 0x24, 0x18, 0x18, 0x7e, 0x7e, 
  0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7e, 0x7e, 0x18, 0x18, 0x24, 0x24, 
  0x42, 0x42, 0x81, 0x81, 0x00, 0x00, 0x00, 0x00
};

static const unsigned char icon_battery[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x3c, 0x3c, 0x7e, 0x7e, 0xff, 0xff, 0xff, 0xff, 
  0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7e, 0x7e, 
  0x3c, 0x3c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

struct MenuItem {
    const char* title;
    const unsigned char* icon;
};

const MenuItem menu_items[] = {
    {"Alarms", icon_alarm},
    {"Settings", icon_settings},
    {"Battery", icon_battery},
    {"Exit", icon_alarm}
};

const int TOTAL_ITEMS = 4;
int current_index = 0;

// Biến trạng thái nội suy camera
float cam_x = 0.0f;
float target_cam_x = 0.0f;
const float LERP_SPEED = 0.18f; // Trọng số mượt mà (0.1 - 0.25)

const int ITEM_SPACING = 40; // Khoảng cách giữa tâm các icon

void setup() {
    Wire.begin(21, 22);
    Wire.setClock(400000); // 400kHz Fast I2C
    u8g2.begin();
    u8g2.setFont(u8g2_font_6x10_tf);
}

void update_physics() {
    // Điểm đích của camera sao cho item được chọn luôn nằm ở giữa màn hình (x = 64)
    target_cam_x = (float)(current_index * ITEM_SPACING);
    // Tính Lerp
    cam_x += (target_cam_x - cam_x) * LERP_SPEED;
}

void draw_carousel_menu() {
    u8g2.clearBuffer();

    // 1. Tiêu đề menu trên cùng
    u8g2.drawStr(30, 10, "< MAIN MENU >");

    // 2. Vẽ danh sách Icon trượt
    int screen_center_x = 64;
    int screen_center_y = 32;

    for (int i = 0; i < TOTAL_ITEMS; i++) {
        // Tọa độ thực tế của từng icon trên màn hình
        int item_real_x = screen_center_x + (i * ITEM_SPACING) - (int)cam_x - 8; // -8 để căn giữa icon 16x16
        int item_real_y = screen_center_y - 8;

        // Chỉ vẽ nếu icon nằm trong vùng nhìn thấy của màn hình
        if (item_real_x > -20 && item_real_x < 128) {
            u8g2.drawXBMP(item_real_x, item_real_y, 16, 16, menu_items[i].icon);
        }
    }

    // 3. Khung cố định ở giữa để làm tiêu điểm (Focus Box dạng NWatch)
    u8g2.drawFrame(52, 20, 24, 24);
    u8g2.drawBox(50, 22, 2, 20); // Điểm nhấn góc focus
    u8g2.drawBox(76, 22, 2, 20);

    // 4. Nhãn tên mục bên dưới
    const char* label = menu_items[current_index].title;
    int str_width = u8g2.getStrWidth(label);
    u8g2.drawStr((128 - str_width) / 2, 58, label);

    u8g2.sendBuffer();
}

void loop() {
    static uint32_t last_tick = 0;
    static uint32_t last_switch = 0;
    uint32_t now = millis();

    // Tự động chuyển trang sau mỗi 2 giây để demo hiệu ứng
    if (now - last_switch > 2000) {
        last_switch = now;
        current_index = (current_index + 1) % TOTAL_ITEMS;
    }

    // Cố định chu kỳ render ~60 FPS (16ms)
    if (now - last_tick >= 16) {
        last_tick = now;
        update_physics();
        draw_carousel_menu();
    }
}

```

---

### 4. Quy tắc Tối ưu Khi Tự Phát Triển UI Tương tự

* **Không dùng hàm `delay()`:** Mọi luồng tính toán chuyển động và đọc nút bấm phải dựa trên biến đo thời gian `millis()` hoặc ngắt phần cứng (Hardware Interrupts).
* **Kiểu dữ liệu số thực (`float`):** Dùng `float` cho các biến vị trí ($X, Y$) trong quá trình tính Lerp để triệt tiêu hiện tượng giật bậc thang khi tốc độ trôi giảm dần, sau đó ép kiểu `(int)` khi truyền vào hàm vẽ của U8g2.
* **Tăng tốc xung giao tiếp:**
* SSD1306 SPI: Đặt SPI clock lên $8\text{ MHz} - 10\text{ MHz}$.
* SSD1306 I2C: Đặt `Wire.setClock(400000)` hoặc `800000` (Fast Mode).