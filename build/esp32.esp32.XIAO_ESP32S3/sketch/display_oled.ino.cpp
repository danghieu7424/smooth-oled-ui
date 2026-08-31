#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
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

// Các Icon XBM 24x24 (Atom Icons - Thiết kế lớn cân đối)
// 1. Icon Home (Ngôi nhà) 24x24
static const unsigned char icon_home[] U8X8_PROGMEM = {
  0x00, 0x08, 0x00, 0x00, 0x1c, 0x00, 0x00, 0x3e, 0x00, 0x00, 0x7f, 0x00,
  0x80, 0xff, 0x00, 0xc0, 0xff, 0x01, 0xe0, 0xff, 0x03, 0xf0, 0xff, 0x07,
  0xf8, 0xff, 0x0f, 0xfc, 0xff, 0x1f, 0xfe, 0xff, 0x3f, 0x3f, 0xff, 0x7f,
  0x0e, 0x00, 0x38, 0x0e, 0x00, 0x38, 0x0e, 0x00, 0x38, 0x0e, 0x00, 0x38,
  0x0e, 0x00, 0x38, 0x0e, 0x00, 0x38, 0x0e, 0x00, 0x38, 0x0e, 0x00, 0x38,
  0x0e, 0x00, 0x38, 0xfe, 0xff, 0x3f, 0xfe, 0xff, 0x3f, 0x00, 0x00, 0x00
};
// 2. Icon Brightness (Mặt trời/Độ sáng) 24x24
static const unsigned char icon_brightness[] U8X8_PROGMEM = {
  0x00, 0x08, 0x00, 0x00, 0x08, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
  0x04, 0x00, 0x10, 0x08, 0xc0, 0x08, 0x10, 0xf0, 0x04, 0x20, 0xf8, 0x02,
  0xc0, 0x7f, 0x01, 0xc0, 0x3f, 0x01, 0x80, 0x3f, 0x00, 0x80, 0x3f, 0x00,
  0x80, 0x3f, 0x00, 0x80, 0x3f, 0x00, 0xc0, 0x3f, 0x01, 0xc0, 0x7f, 0x01,
  0x20, 0xf8, 0x02, 0x10, 0xf0, 0x04, 0x08, 0xc0, 0x08, 0x04, 0x00, 0x10,
  0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x08, 0x00, 0x00, 0x08, 0x00
};
// 3. Icon Settings (Bánh răng) 24x24
static const unsigned char icon_settings[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x80, 0x18, 0x00, 0x80, 0x3c, 0x00,
  0x80, 0x7e, 0x00, 0x08, 0x7e, 0x08, 0x1c, 0xff, 0x1c, 0x3e, 0xff, 0x3e,
  0x7f, 0xff, 0x7f, 0x7f, 0xc3, 0x7f, 0xff, 0x81, 0x7f, 0xff, 0x00, 0x7f,
  0xff, 0x00, 0x7f, 0xff, 0x81, 0x7f, 0x7f, 0xc3, 0x7f, 0x7f, 0xff, 0x7f,
  0x3e, 0xff, 0x3e, 0x1c, 0xff, 0x1c, 0x08, 0x7e, 0x08, 0x80, 0x7e, 0x00,
  0x80, 0x3c, 0x00, 0x80, 0x18, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00
};
// 4. Icon About (Chữ i thông tin) 24x24
static const unsigned char icon_about[] U8X8_PROGMEM = {
  0x00, 0xf8, 0x00, 0x00, 0xfc, 0x01, 0x00, 0xfe, 0x03, 0x00, 0xff, 0x07,
  0x80, 0x8f, 0x0f, 0xc0, 0x07, 0x1f, 0xe0, 0x03, 0x3f, 0xe0, 0x03, 0x3f,
  0xf0, 0x01, 0x7e, 0xf0, 0xc1, 0x7f, 0xf8, 0xe0, 0xff, 0xf8, 0xe0, 0xff,
  0xf8, 0xe0, 0xff, 0xf8, 0xe0, 0xff, 0xf0, 0xc1, 0x7f, 0xf0, 0x01, 0x7e,
  0xe0, 0x03, 0x3f, 0xe0, 0x03, 0x3f, 0xc0, 0x07, 0x1f, 0x80, 0x8f, 0x0f,
  0x00, 0xff, 0x07, 0x00, 0xfe, 0x03, 0x00, 0xfc, 0x01, 0x00, 0xf8, 0x00
};

struct MenuItem {
    const char* title;
    const unsigned char* icon;
};

// Căn cứ theo yêu cầu UI: Cập nhật 4 state mới
const MenuItem menu_items[] = {
    {"Home", icon_home},
    {"Brightness", icon_brightness},
    {"Settings", icon_settings},
    {"About", icon_about}
};

const int TOTAL_ITEMS = 4;
int current_index = 0;

// Biến trạng thái vật lý Lerp
float cam_x = 0.0f;
float target_cam_x = 0.0f;
// Tại sao (Why): Hệ số 0.18 là chuẩn đàn hồi của NWatch để mắt người cảm giác mượt nhất.
const float LERP_SPEED = 0.18f; 
// Tăng Spacing vì Icon đã lớn hơn (từ 40 lên 45)
const int ITEM_SPACING = 45; 

// --- [STATE MACHINE] ---
// Tại sao (Why): Phân tách trạng thái để chạy độc lập 2 Atom trên cùng 1 vòng lặp
enum AppState {
    STATE_CAROUSEL,
    STATE_POPUP
};
AppState app_state = STATE_CAROUSEL;

// --- [POPUP LIST MENU ATOM] ---
const char* list_items[] = {
    "ScreenOff",
    "PowerOff",
    "change mod"
};
const int TOTAL_LIST_ITEMS = 3;
int current_list_selection = 0;

const int MENU_BOX_X = 32;
const int MENU_BOX_Y = 16;
const int MENU_BOX_W = 64;
const int MENU_BOX_H = 36;
const int ITEM_START_Y = 26; 
const int LINE_HEIGHT = 10;  

float cursor_y = (float)(ITEM_START_Y - 8);
float cursor_w = 56.0f;
float target_cursor_y = (float)(ITEM_START_Y - 8);
float target_cursor_w = 56.0f;
const float LIST_LERP_FACTOR = 0.22f;

#line 106 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
void update_list_physics();
#line 114 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
void draw_popup_menu();
#line 155 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
void update_physics();
#line 165 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
void draw_carousel_menu();
#line 195 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
void setup();
#line 207 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
void loop();
#line 106 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
void update_list_physics() {
    target_cursor_y = (float)(ITEM_START_Y + (current_list_selection * LINE_HEIGHT) - 8);
    int text_pixel_width = u8g2.getStrWidth(list_items[current_list_selection]);
    target_cursor_w = (float)(text_pixel_width + 4);
    cursor_y += (target_cursor_y - cursor_y) * LIST_LERP_FACTOR;
    cursor_w += (target_cursor_w - cursor_w) * LIST_LERP_FACTOR;
}

void draw_popup_menu() {
    u8g2.clearBuffer();

    // Giả lập mặt đồng hồ tĩnh phía sau
    u8g2.setDrawColor(1);
    u8g2.setFont(u8g2_font_courB18_tf);
    u8g2.drawStr(5, 42, "12");
    u8g2.drawStr(100, 42, "18");
    u8g2.setFont(u8g2_font_5x7_tf);
    u8g2.drawStr(24, 8, "Tue 01 Oct 2024");
    u8g2.drawStr(88, 60, "33.1C");

    // Xóa nền đen để đè hộp thoại
    u8g2.setDrawColor(0);
    u8g2.drawBox(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H);

    // Vẽ viền hộp thoại
    u8g2.setDrawColor(1);
    u8g2.drawFrame(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H);

    // Vẽ toàn bộ danh sách text
    u8g2.setFont(u8g2_font_6x10_tf);
    for (int i = 0; i < TOTAL_LIST_ITEMS; i++) {
        u8g2.drawStr(MENU_BOX_X + 4, ITEM_START_Y + (i * LINE_HEIGHT), list_items[i]);
    }

    // Đảo màu XOR (Invert Mode)
    // Tại sao (Why): Đè box XOR màu 2 lên chữ để tạo hiệu ứng chia cắt màu sắc mà không cần tính toán tọa độ font phức tạp
    u8g2.setDrawColor(2);
    // Tăng chiều cao hộp thoại từ 9 lên 11 để bao trọn các ký tự có đuôi (g, y, p...)
    u8g2.drawBox(MENU_BOX_X + 2, (int)cursor_y, (int)cursor_w, 11);
    u8g2.setDrawColor(1); // Reset lại chế độ màu

    u8g2.sendBuffer();
}

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
        // Căn chỉnh hệ tọa độ cho ảnh 24x24 (Dịch về tâm 12 thay vì 8 như bản cũ 16x16)
        int item_real_x = screen_center_x + (i * ITEM_SPACING) - (int)cam_x - 12; 
        int item_real_y = screen_center_y - 14;

        if (item_real_x > -30 && item_real_x < 128) {
            u8g2.drawXBMP(item_real_x, item_real_y, 24, 24, menu_items[i].icon);
        }
    }

    // SCSS Glassmorphism Box Fake (Vẽ viền khung chọn tĩnh to hơn để bọc icon 24x24)
    u8g2.drawFrame(48, 14, 32, 32);
    u8g2.drawBox(46, 16, 2, 28); 
    u8g2.drawBox(80, 16, 2, 28);

    const char* label = menu_items[current_index].title;
    int str_width = u8g2.getStrWidth(label);
    u8g2.drawStr((128 - str_width) / 2, 58, label);

    u8g2.sendBuffer();
}

void setup() {
  Wire.begin(47, 48);
  // Tại sao (Why): Đặt I2C lên 400kHz (Fast Mode) bắt buộc để băng thông SPI/I2C đủ xả 1024 Bytes trong 16ms (đạt 60 FPS).
  Wire.setClock(400000); 
  u8g2.begin();
  
  // Tại sao (Why): Màn hình OLED SSD1306 hỗ trợ dải tương phản (0-255). Giảm độ sáng giúp chống cháy điểm ảnh (Burn-in) và giảm chói vào ban đêm.
  u8g2.setContrast(20); // Mức độ sáng 20/255 (khoảng 10%). Bạn có thể đổi số này.

  u8g2.setFont(u8g2_font_6x10_tf);
}

void loop() {
  static uint32_t last_tick = 0;
  static uint32_t last_switch = 0;
  uint32_t now = millis();

  // Máy trạng thái (State Machine)
  if (app_state == STATE_CAROUSEL) {
      // Đổi trang sau mỗi 2s để demo Carousel
      if (now - last_switch > 2000) {
          last_switch = now;
          current_index++;
          // Chuyển sang tính năng List Popup sau khi trượt hết 1 vòng
          if (current_index >= TOTAL_ITEMS) {
              current_index = 0;
              app_state = STATE_POPUP;
          }
      }

      if (now - last_tick >= 16) {
          last_tick = now;
          update_physics();
          draw_carousel_menu();
      }

  } else if (app_state == STATE_POPUP) {
      // Đổi dòng sau mỗi 1.5s để demo Popup List XOR
      if (now - last_switch > 1500) {
          last_switch = now;
          current_list_selection++;
          // Quay lại Carousel sau khi trượt hết list
          if (current_list_selection >= TOTAL_LIST_ITEMS) {
              current_list_selection = 0;
              app_state = STATE_CAROUSEL;
          }
      }

      if (now - last_tick >= 16) {
          last_tick = now;
          update_list_physics();
          draw_popup_menu();
      }
  }
}

