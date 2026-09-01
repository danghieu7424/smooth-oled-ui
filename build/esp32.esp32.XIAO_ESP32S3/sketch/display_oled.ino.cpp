#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>
#include <Preferences.h>
#include <WiFi.h>
#include "SmoothOLED.h"

// Khởi tạo Preferences lưu trữ vào Flash
Preferences prefs;

// Khởi tạo màn hình
U8G2_SSD1306_128X64_NONAME_F_HW_I2C u8g2(U8G2_R0, /* reset=*/ U8X8_PIN_NONE);

// Truyền tham chiếu màn hình và UART (để Stream) vào lõi thư viện
SmoothOLED ui(&u8g2, &Serial);

// =======================================================================
// [DATA] Danh sách Icon (XBM 24x24)
// =======================================================================
static const unsigned char icon_home[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x3c, 0x00, 
  0x00, 0x7e, 0x00, 0x00, 0xe7, 0x00, 0x80, 0xc3, 0x01, 0xc0, 0x81, 0x03, 
  0xe0, 0x00, 0x07, 0x70, 0x00, 0x0e, 0x38, 0x00, 0x1c, 0x3c, 0x00, 0x3c, 
  0x30, 0x00, 0x0c, 0x30, 0x7e, 0x0c, 0x30, 0xff, 0x0c, 0x30, 0xc3, 0x0c, 
  0x30, 0xc3, 0x0c, 0x30, 0xc3, 0x0c, 0x30, 0xc3, 0x0c, 0x30, 0xc3, 0x0c, 
  0xf0, 0xff, 0x0f, 0xe0, 0xff, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

static const unsigned char icon_brightness[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x3c, 0x00, 
  0x00, 0x7e, 0x00, 0xc0, 0xe7, 0x03, 0xe0, 0xc3, 0x07, 0x60, 0x00, 0x06, 
  0x60, 0x3c, 0x06, 0x70, 0x7e, 0x0e, 0x38, 0xff, 0x1c, 0x1c, 0xff, 0x38, 
  0x1c, 0xff, 0x38, 0x38, 0xff, 0x1c, 0x70, 0x7e, 0x0e, 0x60, 0x3c, 0x06, 
  0x60, 0x00, 0x06, 0xe0, 0xc3, 0x07, 0xc0, 0xe7, 0x03, 0x00, 0x7e, 0x00, 
  0x00, 0x3c, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

static const unsigned char icon_settings[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x7e, 0x00, 
  0x38, 0x66, 0x1c, 0xfc, 0xe7, 0x3f, 0xfc, 0xc3, 0x3f, 0x1c, 0x00, 0x38, 
  0x18, 0x3c, 0x18, 0x30, 0x7e, 0x0c, 0x30, 0xe7, 0x0c, 0x30, 0xc3, 0x0c, 
  0x30, 0xc3, 0x0c, 0x30, 0xe7, 0x0c, 0x30, 0x7e, 0x0c, 0x18, 0x3c, 0x18, 
  0x1c, 0x00, 0x38, 0xfc, 0xc3, 0x3f, 0xfc, 0xe7, 0x3f, 0x38, 0x66, 0x1c, 
  0x00, 0x7e, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

static const unsigned char icon_about[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3c, 0x00, 0x80, 0xe7, 0x01, 
  0xe0, 0x00, 0x07, 0x30, 0x00, 0x0c, 0x10, 0x00, 0x08, 0x18, 0x18, 0x18, 
  0x08, 0x18, 0x10, 0x08, 0x00, 0x10, 0x0c, 0x00, 0x30, 0x04, 0x18, 0x20, 
  0x04, 0x18, 0x20, 0x0c, 0x18, 0x30, 0x08, 0x18, 0x10, 0x08, 0x18, 0x18, 
  0x18, 0x18, 0x18, 0x10, 0x00, 0x08, 0x30, 0x00, 0x0c, 0xe0, 0x00, 0x07, 
  0x80, 0xe7, 0x01, 0x00, 0x3c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

// [MỚI] Icon WiFi - Vẽ trên khung 24x24 px
static const unsigned char icon_wifi[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
  0x80, 0xff, 0x01, 0xe0, 0xff, 0x07, 0xf0, 0x00, 0x0f, 0x38, 0x00, 0x1c, 
  0x1c, 0xff, 0x38, 0xc0, 0xff, 0x03, 0xe0, 0x81, 0x07, 0x70, 0x00, 0x0e, 
  0x00, 0x7e, 0x00, 0x80, 0xff, 0x01, 0x80, 0xc3, 0x01, 0x00, 0x00, 0x00, 
  0x00, 0x18, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x18, 0x00, 
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};
// [MỚI] Icon esp now - Vẽ trên khung 24x24 px
static const unsigned char icon_esp_now[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x02, 0x60, 0x00, 0x06, 
  0x30, 0x00, 0x0c, 0x10, 0x42, 0x08, 0x18, 0x81, 0x18, 0x18, 0x99, 0x18, 
  0x18, 0x99, 0x18, 0x18, 0x81, 0x18, 0x10, 0x42, 0x08, 0x30, 0x18, 0x0c, 
  0x60, 0x18, 0x06, 0x40, 0x18, 0x02, 0x00, 0x18, 0x00, 0x00, 0x18, 0x00, 
  0x00, 0x18, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x3c, 0x00, 
  0x00, 0x7e, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

// --- KHAI BÁO CÁC HÀM XỬ LÝ SỰ KIỆN (CALLBACKS) ---
void open_settings_menu();
void open_brightness_slider();
void on_brightness_change(int val);
void on_enter_wifi();
void on_wifi_password_submit(const char* pwd);

const MenuItem menu_items[] = {
    {"Home", icon_home, nullptr},
    {"Settings", icon_settings, open_settings_menu},
    {"About", icon_about, nullptr}
};
const int TOTAL_MAIN_ITEMS = 3;

const MenuItem settings_items[] = {
    {"WiFi", icon_wifi, on_enter_wifi},
    {"ESP NOW", icon_esp_now, nullptr},
    {"Brightness", icon_brightness, open_brightness_slider}
};
const int TOTAL_SETTINGS_ITEMS = 3;

const char* popup_items[] = {
    "ScreenOff",
    "PowerOff",
    "change mod",
    "smooth screen ui"
};
const int TOTAL_POPUP_ITEMS = 4;

const char* side_items[] = {
    "Normal",
    "CodeRain",
    "TerminalSim",
    "SimClock",
    "Cube3D",
    "Snow",
    "Galaxy"
};
const int TOTAL_SIDE_ITEMS = 7;


// =======================================================================
// [SETUP & LOOP]
// =======================================================================

// Quản lý trạng thái Menu
enum MenuLevel { LEVEL_MAIN, LEVEL_SETTINGS, LEVEL_WIFI };
MenuLevel current_level = LEVEL_MAIN;
int current_brightness = 20; // Độ sáng hiện tại
int saved_brightness = 20; // Độ sáng đã lưu trong Flash

// Quản lý WiFi
#define MAX_WIFI_NETWORKS 15
char wifi_ssid[MAX_WIFI_NETWORKS][32];
char wifi_raw_ssid[MAX_WIFI_NETWORKS][32];
const char* wifi_ssid_ptrs[MAX_WIFI_NETWORKS];
int wifi_count = 0;
bool is_scanning_wifi = false;

// --- CÀI ĐẶT CÁC HÀM XỬ LÝ SỰ KIỆN ---

void on_wifi_selected(int idx);

#line 185 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
void setup();
#line 213 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
void loop();
#line 138 "D:\\all_projects\\rust\\rust\\display_oled\\display_oled.ino"
void open_settings_menu() {
  current_level = LEVEL_SETTINGS;
  ui.setCarouselItems(settings_items, TOTAL_SETTINGS_ITEMS, "< SETTINGS >");
}

void open_brightness_slider() {
  // Khi mở thanh gạt, ta đăng ký hàm `on_brightness_change` làm Callback!
  ui.openSlider("Brightness", current_brightness, 255, on_brightness_change);
}

void on_brightness_change(int val) {
  current_brightness = val;
  u8g2.setContrast(current_brightness); // Lệnh phần cứng đổi độ sáng OLED trực tiếp
}

void on_wifi_selected(int idx) {
  // Chặn mở mật khẩu nếu đang Scanning, không có mạng, hoặc lỗi
  if (idx >= 0 && idx < wifi_count && strncmp(wifi_ssid[0], "Scanning", 8) != 0 && strncmp(wifi_ssid[0], "No networks", 10) != 0 && strncmp(wifi_ssid[0], "Scan Failed", 11) != 0) {
      char title[32];
      snprintf(title, sizeof(title), "PWD: %s", wifi_raw_ssid[idx]);
      ui.openTextInput(title, on_wifi_password_submit);
  }
}

void on_enter_wifi() {
  current_level = LEVEL_WIFI;
  is_scanning_wifi = true;
  WiFi.mode(WIFI_STA);
  WiFi.disconnect();
  WiFi.scanNetworks(true); // Quét bất đồng bộ (Async)
  
  // Khởi tạo UI hiển thị tạm thời "Scanning..."
  wifi_count = 1;
  strncpy(wifi_ssid[0], "Scanning...", 31);
  wifi_ssid_ptrs[0] = wifi_ssid[0];
  ui.openFullList("WIFI NETWORKS", wifi_ssid_ptrs, wifi_count, on_wifi_selected);
}

void on_wifi_password_submit(const char* pwd) {
  int idx = ui.getFullListSelectedIndex();
  if (idx >= 0 && idx < wifi_count) {
      String ssid = wifi_raw_ssid[idx];
      Serial.printf("\n[WiFi] Connecting to %s with password: %s\n", ssid.c_str(), pwd);
      WiFi.begin(ssid.c_str(), pwd);
  }
}

void setup() {
  Serial.begin(921600);
  
  // Nạp cấu hình từ Flash
  prefs.begin("oled-ui", false);
  saved_brightness = prefs.getInt("brightness", 20); // Nếu chưa lưu, mặc định là 20
  current_brightness = saved_brightness;

  Wire.begin(47, 48);
  Wire.setClock(400000); 
  u8g2.begin();
  u8g2.setContrast(current_brightness); // Áp dụng độ sáng đã lưu

  // 1. Gán mảng dữ liệu vào thư viện UI
  ui.setCarouselItems(menu_items, TOTAL_MAIN_ITEMS, "< MAIN MENU >");
  ui.setPopupListItems(popup_items, TOTAL_POPUP_ITEMS);
  ui.setSidePopupItems(side_items, TOTAL_SIDE_ITEMS);

  // 2. Bật chế độ Demo vòng lặp vô hạn (Tạm tắt để dùng Serial Control)
  ui.enableAutoDemo(false);

  // 2.5. Bật chế độ xuất khung hình ra Serial cho PC Viewer (Tắt khi Release)
  ui.enablePCViewer(true);

  // 3. Khởi động UI
  ui.begin();
}

void loop() {
  // Lắng nghe lệnh từ cổng Serial (gửi từ Python Script)
  if (Serial.available() > 0) {
    char c = Serial.read();
    if (c == 'U') ui.up();        // Mũi tên lên / Trái
    else if (c == 'D') ui.down(); // Mũi tên xuống / Phải
    else if (c == 'P') {
      ui.setPopupListItems(popup_items, TOTAL_POPUP_ITEMS);
      ui.openPopup(); // Phím End
    }
    else if (c == 'S') ui.openSideList(); // Phím Home
    else if (c == 'C') { // Phím Esc
      if (ui.isOverlayOpen()) {
        ui.closeOverlay(); // Đóng Side List
      } else if (ui.getAppState() == STATE_TEXT_INPUT) {
        if (!ui.backspace()) {
           // Nếu backspace trả về false (nghĩa là đã xóa hết chữ và bấm tiếp), nó sẽ tự đóng và lùi về Popup/FullList
        }
      } else if (ui.getAppState() == STATE_POPUP) {
        ui.closeOverlay(); // Đóng Popup List
      } else if (ui.getAppState() == STATE_FULL_LIST) {
        if (current_level == LEVEL_WIFI) {
            current_level = LEVEL_SETTINGS; // Lùi về Settings
        }
        ui.closeOverlay();
      } else if (ui.getAppState() == STATE_SLIDER) {
        // [Cập nhật]: Hủy bỏ, khôi phục độ sáng cũ từ Flash
        current_brightness = saved_brightness;
        u8g2.setContrast(current_brightness);
        ui.closeOverlay(); // Đóng Slider, trả về Level trước đó
      } else if (ui.getAppState() == STATE_CAROUSEL && current_level == LEVEL_SETTINGS) {
        current_level = LEVEL_MAIN; // Lùi về Main Menu
        ui.setCarouselItems(menu_items, TOTAL_MAIN_ITEMS, "< MAIN MENU >");
      }
    }
    else if (c == 'E') { // Phím Enter
      ui.select();
      
      if (ui.getAppState() == STATE_SLIDER) {
          // [Cập nhật]: Lưu độ sáng vào Flash
          prefs.putInt("brightness", current_brightness);
          saved_brightness = current_brightness;
          ui.closeOverlay(); // Đóng Slider, xác nhận lưu
      } else if (!ui.isOverlayOpen() && ui.getAppState() == STATE_CAROUSEL) {
          const MenuItem* active_item = ui.getCurrentMenuItem();
          if (active_item && active_item->on_enter) {
              active_item->on_enter();
          }
      }
    }
  }

  // --- KIỂM TRA TRẠNG THÁI QUÉT WIFI ASYNC ---
  if (is_scanning_wifi) {
      int n = WiFi.scanComplete();
      if (n >= 0) {
          is_scanning_wifi = false;
          wifi_count = (n > MAX_WIFI_NETWORKS) ? MAX_WIFI_NETWORKS : n;
          if (wifi_count == 0) {
              strncpy(wifi_ssid[0], "No networks", 31);
              wifi_ssid_ptrs[0] = wifi_ssid[0];
              wifi_count = 1;
          } else {
              for (int i = 0; i < wifi_count; i++) {
                  // Lưu tên SSID thật vào raw
                  strncpy(wifi_raw_ssid[i], WiFi.SSID(i).c_str(), 31);
                  wifi_raw_ssid[i][31] = '\0';
                  // Định dạng tên + cường độ hiển thị trên OLED
                  snprintf(wifi_ssid[i], 31, "%s [%d]", wifi_raw_ssid[i], WiFi.RSSI(i));
                  wifi_ssid_ptrs[i] = wifi_ssid[i];
              }
          }
          if (current_level == LEVEL_WIFI && ui.getAppState() == STATE_FULL_LIST) {
              ui.setFullListCount(wifi_count);
          }
          WiFi.scanDelete(); // Xóa bộ đệm
      } else if (n == WIFI_SCAN_FAILED) {
          is_scanning_wifi = false;
          strncpy(wifi_ssid[0], "Scan Failed", 31);
          wifi_ssid_ptrs[0] = wifi_ssid[0];
          wifi_count = 1;
          if (current_level == LEVEL_WIFI && ui.getAppState() == STATE_FULL_LIST) {
              ui.setFullListCount(wifi_count);
          }
      }
  }

  // Cập nhật UI (60FPS)
  ui.update();
}

