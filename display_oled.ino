/****
 * [UI-ATOM] Thành phần Carousel Menu (Trượt ngang quán tính)
 * Input: Các thông số trạng thái hiện tại (Index, CamX)
 * Output: Khởi tạo phần cứng màn hình và I2C Bus.
 ****/
#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>

// Khởi tạo SSD1306 I2C chế độ Full Buffer
// Căn cứ theo quyết định kinh doanh: Chế độ Full Buffer cần 1024 Bytes RAM nhưng tối ưu tốc độ xé hình tốt nhất.
U8G2_SSD1306_128X64_NONAME_F_HW_I2C u8g2(U8G2_R0, /* reset=*/ U8X8_PIN_NONE);

// Các Icon XBM (Atom Icons)
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

// Biến trạng thái vật lý Lerp
float cam_x = 0.0f;
float target_cam_x = 0.0f;
// Tại sao (Why): Hệ số 0.18 là chuẩn đàn hồi của NWatch để mắt người cảm giác mượt nhất.
const float LERP_SPEED = 0.18f; 
const int ITEM_SPACING = 40; 

/****
 * [LOGIC] Tính toán nội suy vị trí camera (Physics Engine)
 * Input: Không có
 * Output: Cập nhật biến cam_x để cuộn trượt mượt.
 ****/
void update_physics() {
    target_cam_x = (float)(current_index * ITEM_SPACING);
    cam_x += (target_cam_x - cam_x) * LERP_SPEED;
}

/****
 * [DRAW] Lệnh render màn hình toàn khung
 * Input: Không có
 * Output: Vẽ frame mới nhất lên u8g2.
 ****/
void draw_carousel_menu() {
    u8g2.clearBuffer();

    u8g2.drawStr(30, 10, "< MAIN MENU >");

    int screen_center_x = 64;
    int screen_center_y = 32;

    for (int i = 0; i < TOTAL_ITEMS; i++) {
        int item_real_x = screen_center_x + (i * ITEM_SPACING) - (int)cam_x - 8; 
        int item_real_y = screen_center_y - 8;

        if (item_real_x > -20 && item_real_x < 128) {
            u8g2.drawXBMP(item_real_x, item_real_y, 16, 16, menu_items[i].icon);
        }
    }

    // SCSS Glassmorphism Box Fake (Vẽ viền khung chọn tĩnh)
    u8g2.drawFrame(52, 20, 24, 24);
    u8g2.drawBox(50, 22, 2, 20); 
    u8g2.drawBox(76, 22, 2, 20);

    const char* label = menu_items[current_index].title;
    int str_width = u8g2.getStrWidth(label);
    u8g2.drawStr((128 - str_width) / 2, 58, label);

    u8g2.sendBuffer();
}

void setup() {
  Wire.begin(21, 22);
  // Tại sao (Why): Đặt I2C lên 400kHz (Fast Mode) bắt buộc để render đạt 60 FPS không bị nghẽn SPI.
  Wire.setClock(400000); 
  u8g2.begin();
  u8g2.setFont(u8g2_font_6x10_tf);
}

void loop() {
  static uint32_t last_tick = 0;
  static uint32_t last_switch = 0;
  uint32_t now = millis();

  // Đổi trang sau mỗi 2s để demo
  if (now - last_switch > 2000) {
      last_switch = now;
      current_index = (current_index + 1) % TOTAL_ITEMS;
  }

  // Cố định render ~60 FPS (16ms)
  // Tại sao (Why): Không dùng hàm delay() để CPU rảnh xử lý ngắt và tránh văng Watchdog.
  if (now - last_tick >= 16) {
      last_tick = now;
      update_physics();
      draw_carousel_menu();
  }
}
