#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\list.md"
[HW-ELEC]

Hiệu ứng đảo màu từng nửa con chữ khi thanh chọn trượt qua (như trong hình ảnh đính kèm) được thực hiện nhờ chế độ vẽ **XOR / Invert Mode** (`u8g2.setDrawColor(2)`).

Cơ chế hoạt động:

1. **Lớp nền (Background & Text):** Vẽ toàn bộ các chuỗi text thông thường với màu trắng/đen chuẩn.
2. **Khung chọn nội suy (Interpolated Elastic Box):** Vị trí $Y$ và chiều dài $Width$ của thanh bôi đen được tính toán trôi mượt bằng **Lerp**.
3. **Phép đảo màu (XOR Draw):** Vẽ một khối chữ nhật đè lên buffer tại tọa độ $(X, Y, Width, Height)$ ở chế độ `setDrawColor(2)`. Bất kỳ pixel chữ nào bị hình chữ nhật này trùm lên sẽ tự động bị lật ngược bit (sáng $\to$ tối, tối $\to$ sáng), tạo ra hiện tượng chia cắt nửa trên - nửa dưới chính xác tuyệt đối mà không cần tính toán cắt chữ phức tạp.

---

### Mã Nguồn Hoàn Chỉnh (ESP32: SDA = 47, SCL = 48)

```cpp
#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>

#define I2C_SDA_PIN 47
#define I2C_SCL_PIN 48

// Khởi tạo SSD1306 Hardware I2C Full Buffer
U8G2_SSD1306_128X64_NONAME_F_HW_I2C u8g2(U8G2_R0, /* reset=*/ U8X8_PIN_NONE);

// Danh sách các mục trong Menu
const char* menu_items[] = {
    "ScreenOff",
    "PowerOff",
    "change mod"
};
const int TOTAL_MENU_ITEMS = 3;
int current_selection = 0;

// Thông số layout menu
const int MENU_BOX_X = 32;
const int MENU_BOX_Y = 16;
const int MENU_BOX_W = 64;
const int MENU_BOX_H = 36;
const int ITEM_START_Y = 26; // Tọa độ baseline dòng đầu tiên
const int LINE_HEIGHT = 10;  // Khoảng cách giữa các dòng

// Biến nội suy chuyển động (Lerp Physics)
float cursor_y = (float)(ITEM_START_Y - 9);
float cursor_w = 56.0f;
float target_cursor_y = (float)(ITEM_START_Y - 9);
float target_cursor_w = 56.0f;

const float LERP_FACTOR = 0.22f; // Tốc độ co giãn và trượt

void setup() {
    // Khởi tạo I2C trên ESP32 chân 47, 48 và nâng tốc độ lên 400kHz
    Wire.begin(I2C_SDA_PIN, I2C_SCL_PIN);
    Wire.setClock(400000);

    u8g2.begin();
    u8g2.setFont(u8g2_font_6x10_tf);
}

void update_menu_physics() {
    // 1. Tính toán đích đến Y và Width theo item đang chọn
    target_cursor_y = (float)(ITEM_START_Y + (current_selection * LINE_HEIGHT) - 8);
    
    // Chiều rộng thanh bôi đen = độ dài chữ thực tế + lề (padding 4px)
    int text_pixel_width = u8g2.getStrWidth(menu_items[current_selection]);
    target_cursor_w = (float)(text_pixel_width + 4);

    // 2. Nội suy vị trí Y và chiều dài Width (Elastic effect)
    cursor_y += (target_cursor_y - cursor_y) * LERP_FACTOR;
    cursor_w += (target_cursor_w - cursor_w) * LERP_FACTOR;
}

void draw_popup_menu() {
    u8g2.clearBuffer();

    // --- Giả lập màn hình nền (Mặt đồng hồ phía sau popup) ---
    u8g2.setDrawColor(1);
    u8g2.setFont(u8g2_font_courB18_tf);
    u8g2.drawStr(5, 42, "12");
    u8g2.drawStr(100, 42, "18");
    u8g2.setFont(u8g2_font_5x7_tf);
    u8g2.drawStr(24, 8, "Tue 01 Oct 2024");
    u8g2.drawStr(88, 60, "33.1C");

    // --- Vẽ Hộp thoại Menu ---
    // 1. Xóa vùng nền popup (vẽ đè box đen để che đồng hồ phía sau)
    u8g2.setDrawColor(0);
    u8g2.drawBox(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H);

    // 2. Vẽ khung viền hộp thoại
    u8g2.setDrawColor(1);
    u8g2.drawFrame(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H);

    // 3. Vẽ toàn bộ Text của Menu (vẽ chữ trắng thông thường)
    u8g2.setFont(u8g2_font_6x10_tf);
    for (int i = 0; i < TOTAL_MENU_ITEMS; i++) {
        u8g2.drawStr(MENU_BOX_X + 4, ITEM_START_Y + (i * LINE_HEIGHT), menu_items[i]);
    }

    // 4. KỸ THUẬT ĐẢO MÀU (XOR / Invert):
    // Đổi DrawColor thành 2 (XOR Mode) và vẽ hình chữ nhật trượt đè lên
    u8g2.setDrawColor(2);
    u8g2.drawBox(MENU_BOX_X + 2, (int)cursor_y, (int)cursor_w, 9);

    // Trả DrawColor về mặc định sau khi hoàn tất
    u8g2.setDrawColor(1);

    // Đẩy Framebuffer ra màn hình SSD1306
    u8g2.sendBuffer();
}

void loop() {
    static uint32_t last_frame_time = 0;
    static uint32_t last_input_time = 0;
    uint32_t now = millis();

    // Giả lập thao tác chuyển mục chọn sau mỗi 1.5 giây để kiểm tra animation
    if (now - last_input_time >= 1500) {
        last_input_time = now;
        current_selection = (current_selection + 1) % TOTAL_MENU_ITEMS;
    }

    // Vòng lặp Render cố định tốc độ ~60 FPS (16ms)
    if (now - last_frame_time >= 16) {
        last_frame_time = now;
        update_menu_physics();
        draw_popup_menu();
    }
}

```