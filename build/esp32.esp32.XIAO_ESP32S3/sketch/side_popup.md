#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\side_popup.md"
[HW-ELEC]

Hiệu ứng trong hình là **Side Popup / Circular Reveal Menu** (thường thấy trong TxtViewer / NWatch). Giao diện kết hợp 3 kỹ thuật chính:

1. **Hiệu ứng đẩy lùi và mở rộng (Slide & Reveal Animation):** Menu chính trượt nhẹ sang trái, đồng thời một hình tròn hoặc cánh cung màu đen mở rộng dần từ cạnh phải (bán kính $R$ tăng dần theo hàm Lerp) để che một phần icon/nền phía dưới.
2. **Cuộn danh sách vô hạn theo trục dọc (Vertical Scroll with Camera Lerp):** Khi danh sách dài hơn chiều cao màn hình, danh sách chữ trượt dọc theo biến vị trí `cam_y`.
3. **Thanh bôi đen đảo màu cục bộ (Elastic XOR Selection):** Áp dụng chế độ `setDrawColor(2)` để đảo màu chữ tại item được chọn ngay trên nền cánh cung.

---

### Mã Nguồn Triển Khai Hoàn Chỉnh (ESP32: SDA = 47, SCL = 48)

```cpp
#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>

#define I2C_SDA_PIN 47
#define I2C_SCL_PIN 48

U8G2_SSD1306_128X64_NONAME_F_HW_I2C u8g2(U8G2_R0, /* reset=*/ U8X8_PIN_NONE);

// Icon đồng hồ báo thức 24x24 (XBM format) đại diện cho Menu cha bên dưới
static const unsigned char icon_alarm_24[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x18, 0x00, 0x18, 0x3c, 0x00, 0x3c, 0x66, 0x00, 0x66,
  0xc3, 0x81, 0xc3, 0x81, 0xff, 0x81, 0x00, 0xff, 0x00, 0x00, 0x18, 0x00,
  0x00, 0x18, 0x00, 0x00, 0x18, 0x00, 0x00, 0x18, 0x00, 0x00, 0x18, 0x00,
  0x00, 0xff, 0x00, 0x81, 0xff, 0x81, 0xc3, 0x81, 0xc3, 0x66, 0x00, 0x66,
  0x3c, 0x00, 0x3c, 0x18, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

// Danh sách Watchface/Chức năng trong Sub-menu
const char* list_items[] = {
    "Normal",
    "CodeRain",
    "TerminalSim",
    "SimClock",
    "Cube3D",
    "Snow",
    "Galaxy"
};
const int TOTAL_ITEMS = 7;
int selected_idx = 0;

// Trạng thái vật lý nội suy (Lerp)
float parent_x = 32.0f;       // Tọa độ X của menu/icon cha (bị đẩy lùi về trái)
float arc_radius = 0.0f;      // Bán kính hình tròn che nền
float list_cam_y = 0.0f;      // Vị trí cuộn danh sách dọc
float cursor_w = 40.0f;       // Chiều dài thanh bôi đen

const float TARGET_PARENT_X = 18.0f;
const float TARGET_ARC_RADIUS = 52.0f;
const float LERP_FACTOR = 0.20f;
const int LINE_SPACING = 11;

void setup() {
    Wire.begin(I2C_SDA_PIN, I2C_SCL_PIN);
    Wire.setClock(400000);

    u8g2.begin();
    u8g2.setFont(u8g2_font_6x10_tf);
}

void update_physics() {
    // 1. Hiệu ứng mở cánh menu sang bên trái
    parent_x += (TARGET_PARENT_X - parent_x) * LERP_FACTOR;
    arc_radius += (TARGET_ARC_RADIUS - arc_radius) * LERP_FACTOR;

    // 2. Điểm đích cuộn sao cho item đang chọn luôn nằm ở vùng hiển thị giữa (Y ~ 26)
    float target_cam_y = (float)(selected_idx * LINE_SPACING);
    list_cam_y += (target_cam_y - list_cam_y) * LERP_FACTOR;

    // 3. Nội suy chiều rộng thanh focus theo độ dài ký tự
    int target_w = u8g2.getStrWidth(list_items[selected_idx]) + 4;
    cursor_w += (target_w - cursor_w) * LERP_FACTOR;
}

void draw_side_list_menu() {
    u8g2.clearBuffer();

    // --- LỚP 1: Giao diện cha bên dưới (Đang bị đẩy sang trái) ---
    u8g2.setDrawColor(1);
    u8g2.drawStr((int)parent_x - 4, 10, "MAIN MENU");
    u8g2.drawXBMP((int)parent_x, 24, 24, 24, icon_alarm_24);

    // --- LỚP 2: Cánh cung / Mặt nạ che nền (Circular Mask) ---
    // Vẽ hình tròn đặc màu đen (Color 0) để cắt và đè lên icon cha
    u8g2.setDrawColor(0);
    u8g2.drawDisc(78, 32, (int)arc_radius);

    // Vẽ đường viền vòng cung màu trắng (Color 1) bao quanh mép menu
    u8g2.setDrawColor(1);
    u8g2.drawCircle(78, 32, (int)arc_radius, U8G2_DRAW_ALL);

    // --- LỚP 3: Danh sách chữ cuộn dọc (Scroll List) ---
    int base_y = 28; // Vị trí dòng được chọn trên màn hình
    for (int i = 0; i < TOTAL_ITEMS; i++) {
        int item_y = base_y + (i * LINE_SPACING) - (int)list_cam_y;

        // Chỉ vẽ text nếu nằm trong giới hạn hiển thị dọc của màn hình
        if (item_y > 10 && item_y < 64) {
            u8g2.drawStr(42, item_y, list_items[i]);
        }
    }

    // --- LỚP 4: Thanh bôi đen đảo màu (XOR Mode) ---
    // Đặt thanh đảo màu cố định tại vị trí của mục được focus
    u8g2.setDrawColor(2);
    u8g2.drawBox(40, base_y - 8, (int)cursor_w, 10);

    // Trả chế độ màu về mặc định
    u8g2.setDrawColor(1);

    u8g2.sendBuffer();
}

void loop() {
    static uint32_t last_frame_time = 0;
    static uint32_t last_input_time = 0;
    uint32_t now = millis();

    // Tự động chuyển mục sau mỗi 1.8 giây để minh họa hiệu ứng cuộn
    if (now - last_input_time >= 1800) {
        last_input_time = now;
        selected_idx = (selected_idx + 1) % TOTAL_ITEMS;
    }

    // Duy trì chu kỳ render ~60 FPS (16ms)
    if (now - last_frame_time >= 16) {
        last_frame_time = now;
        update_physics();
        draw_side_list_menu();
    }
}

```