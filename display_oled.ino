#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>
#include <Preferences.h>
#include <WiFi.h>
#include <EEPROM.h>
#include "SmoothOLED.h"

// Khởi tạo Preferences lưu trữ vào Flash (chỉ cho độ sáng)
Preferences prefs;


// Cấu trúc lưu WiFi bằng EEPROM thô (chống mọi lỗi NVS)
struct WiFiState {
    bool on;
    char ssid[32];
    char pwd[64];
};

void save_wifi_state(bool on, String ssid, String pwd) {
    WiFiState state;
    state.on = on;
    strncpy(state.ssid, ssid.c_str(), 31);
    state.ssid[31] = '\0';
    strncpy(state.pwd, pwd.c_str(), 63);
    state.pwd[63] = '\0';
    EEPROM.put(0, state);
    EEPROM.commit();
}

WiFiState load_wifi_state() {
    WiFiState state;
    EEPROM.get(0, state);
    // Kiểm tra nếu bộ nhớ rác (chưa từng được ghi)
    if (state.ssid[0] == (char)255 || state.ssid[0] < 32 || state.ssid[0] > 126) {
        state.on = false;
        state.ssid[0] = '\0';
        state.pwd[0] = '\0';
    }
    return state;
}

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

void on_restart() {
    ESP.restart(); // Sửa lại lệnh chuẩn của ESP32
}

void on_power_off() {
    esp_deep_sleep_start();
}

const MenuItem side_items[] = {
    {"Restart", nullptr, on_restart},
    {"PowerOff", nullptr, on_power_off}
};
const int TOTAL_SIDE_ITEMS = 2;


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

// Trạng thái kết nối WiFi
bool is_connecting_wifi = false;
uint32_t wifi_connect_start = 0;
String connecting_ssid = "";
String connecting_pwd = "";

// --- CÀI ĐẶT CÁC HÀM XỬ LÝ SỰ KIỆN ---

void on_wifi_selected(int idx);

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

char text_input_title_buf[64];

void on_wifi_selected(int idx) {
  // Chặn mở mật khẩu nếu đang Scanning, không có mạng, hoặc lỗi
  if (idx >= 0 && idx < wifi_count && strncmp(wifi_ssid[0], "Scanning", 8) != 0 && strncmp(wifi_ssid[0], "No networks", 10) != 0 && strncmp(wifi_ssid[0], "Scan Failed", 11) != 0) {
      // Nếu mạng này đang được kết nối rồi, báo luôn không cần nhập pass
      if (WiFi.status() == WL_CONNECTED && WiFi.SSID() == String(wifi_raw_ssid[idx])) {
          ui.openModal("Connected!", "Already connected to this network");
          return;
      }

      // KIỂM TRA: Nếu mạng này TRÙNG với mạng đã lưu trong Flash
      String saved_pwd = "";
      WiFiState state = load_wifi_state();
      
      if (String(wifi_raw_ssid[idx]) == String(state.ssid)) {
          // Đã có mật khẩu trong Flash -> Lấy ra để điền sẵn vào ô Input
          saved_pwd = String(state.pwd);
      }
      
      // Mở ô nhập Pass và ĐIỀN SẴN mật khẩu cũ (như thẻ input type="text" có value)
      // Người dùng chỉ cần ấn Enter để kết nối, hoặc ấn xóa để sửa
      snprintf(text_input_title_buf, sizeof(text_input_title_buf), "PWD: %s", wifi_raw_ssid[idx]);
      ui.openTextInput(text_input_title_buf, on_wifi_password_submit, saved_pwd.c_str());
  }
}

void on_enter_wifi() {
  current_level = LEVEL_WIFI;
  
  // Khởi tạo UI hiển thị tạm thời "Scanning..."
  wifi_count = 1;
  strncpy(wifi_ssid[0], "Scanning...", 31);
  wifi_ssid_ptrs[0] = wifi_ssid[0];
  ui.openFullList("WIFI NETWORKS", wifi_ssid_ptrs, wifi_count, on_wifi_selected);

  if (is_scanning_wifi) return; // Đang quét thì không kích hoạt lại
  
  WiFi.mode(WIFI_STA);
  // XÓA WiFi.disconnect() ở đây để không làm rớt mạng đang kết nối khi load lại menu
  
  WiFi.scanNetworks(true); // Quét bất đồng bộ (Async)
  is_scanning_wifi = true;
}

void on_wifi_password_submit(const char* pwd) {
  int idx = ui.getFullListSelectedIndex();
  if (idx >= 0 && idx < wifi_count) {
      connecting_ssid = wifi_raw_ssid[idx];
      connecting_pwd = pwd;
      
      Serial.printf("\n[WiFi] Connecting to %s with password: %s\n", connecting_ssid.c_str(), pwd);
      
      // Ngắt kết nối cũ (nếu có)
      WiFi.disconnect();
      delay(100);
      WiFi.begin(connecting_ssid.c_str(), pwd);
      
      is_connecting_wifi = true;
      wifi_connect_start = millis();
      
      // Hiển thị trạng thái Connecting... lên màn hình
      ui.openModal("Connecting...", connecting_ssid.c_str());
  }
}

void setup() {
  Serial.begin(921600);
  
  // Nạp cấu hình từ Flash
  prefs.begin("oled-ui", false);
  EEPROM.begin(512); // Khởi tạo vùng nhớ EEPROM thô để lưu WiFi
  
  saved_brightness = prefs.getInt("brightness", 20); // Nếu chưa lưu, mặc định là 20
  current_brightness = saved_brightness;

  Wire.begin(47, 48);
  Wire.setClock(400000); 
  u8g2.begin();
  u8g2.setContrast(current_brightness); // Áp dụng độ sáng đã lưu

  // Khởi động lại: Xem trạng thái trước đó có đang kết nối không
  WiFiState state = load_wifi_state();
  if (state.on) {
      // Nếu có thì bật wifi và kết nối lại ssid và password cũ
      WiFi.mode(WIFI_STA);
      WiFi.disconnect(true); // Xóa state lơ lửng của phần cứng
      delay(100);
      WiFi.begin(state.ssid, state.pwd);
      WiFi.setAutoReconnect(true); // Tự động kết nối lại nếu rớt mạng
  } else {
      // Nếu không thì không bật
      WiFi.mode(WIFI_OFF);
  }

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
    
    if (c == '\x1B') {
        // Lệnh điều hướng
        uint32_t t = millis();
        while (!Serial.available() && millis() - t < 50) { delay(1); }
        if (Serial.available()) {
            char cmd = Serial.read();
            if (cmd == 'U') ui.up();        // Mũi tên Lên
            else if (cmd == 'D') ui.down(); // Mũi tên Xuống
            else if (cmd == 'L') ui.left(); // Mũi tên Trái
            else if (cmd == 'R') ui.right();// Mũi tên Phải
            else if (cmd == 'P') {
              ui.setPopupListItems(popup_items, TOTAL_POPUP_ITEMS);
              ui.openPopup(); // Phím End
            }
            else if (cmd == 'S') ui.openSideList(); // Phím Home
            else if (cmd == 'C') { // Phím Esc
              if (ui.isOverlayOpen()) {
                ui.closeOverlay(); // Đóng Side List
              } else if (ui.getAppState() == STATE_TEXT_INPUT) {
                ui.closeOverlay(); // Esc -> Thoát thẳng nhập Pass
              } else if (ui.getAppState() == STATE_POPUP || ui.getAppState() == STATE_MODAL) {
                ui.closeOverlay(); // Đóng Popup/Modal
              } else if (ui.getAppState() == STATE_FULL_LIST) {
                if (current_level == LEVEL_WIFI) {
                    current_level = LEVEL_SETTINGS; // Lùi về Settings
                    ui.setCarouselItems(settings_items, TOTAL_SETTINGS_ITEMS, "< SETTINGS >");
                    // Nếu thoát ra mà không có kết nối nào, tắt Wi-Fi để tiết kiệm pin
                    if (WiFi.status() != WL_CONNECTED) {
                        WiFi.mode(WIFI_OFF);
                        WiFiState state = load_wifi_state();
                        save_wifi_state(false, state.ssid, state.pwd); // Chỉ cập nhật cờ on thành false
                    }
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
            else if (cmd == 'B') { // Phím Backspace
              if (ui.getAppState() == STATE_TEXT_INPUT) {
                ui.backspace();
              } else if (current_level == LEVEL_WIFI) {
                  // Đang ở danh sách Wi-Fi (hoặc Modal), ấn Backspace để ngắt kết nối hiện tại
                  if (WiFi.status() == WL_CONNECTED) {
                      WiFi.disconnect();
                      WiFiState state = load_wifi_state();
                      save_wifi_state(false, state.ssid, state.pwd); // Cập nhật trạng thái tắt kết nối vào Flash
                      
                      // Xóa dấu * khỏi danh sách ngay lập tức
                      for (int i = 0; i < wifi_count; i++) {
                          if (wifi_ssid[i][0] == '*') {
                              String temp = String(wifi_ssid[i]).substring(2); // Cắt bỏ "* "
                              strncpy(wifi_ssid[i], temp.c_str(), 31);
                              wifi_ssid[i][31] = '\0';
                          }
                      }
                      ui.openModal("Disconnected", "Wi-Fi is now disconnected");
                  }
              }
            }
            else if (cmd == 'E') { // Phím Enter
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
    } else {
        // Ký tự gõ trực tiếp từ bàn phím
        ui.inputChar(c);
    }
  }

  // --- THEO DÕI TRẠNG THÁI WIFI NGẦM ĐỂ CẬP NHẬT DẤU * ---
  static wl_status_t last_wifi_status = WL_DISCONNECTED;
  wl_status_t current_status = WiFi.status();
  if (current_status != last_wifi_status) {
      last_wifi_status = current_status;
      if (current_status == WL_CONNECTED) {
          String connected_ssid = WiFi.SSID();
          for (int i = 0; i < wifi_count; i++) {
              if (String(wifi_raw_ssid[i]) == connected_ssid) {
                  if (wifi_ssid[i][0] != '*') {
                      char temp[32];
                      snprintf(temp, 32, "* %s", wifi_ssid[i]);
                      strncpy(wifi_ssid[i], temp, 31);
                      wifi_ssid[i][31] = '\0';
                  }
              }
          }
      } else {
          for (int i = 0; i < wifi_count; i++) {
              if (wifi_ssid[i][0] == '*') {
                  String temp = String(wifi_ssid[i]).substring(2);
                  strncpy(wifi_ssid[i], temp.c_str(), 31);
                  wifi_ssid[i][31] = '\0';
              }
          }
      }
  }

  // --- KIỂM TRA TRẠNG THÁI QUÉT WIFI ASYNC ---
  if (is_scanning_wifi) {
    int16_t scan_result = WiFi.scanComplete();
    if (scan_result >= 0) {
      is_scanning_wifi = false;
      wifi_count = 0;
      if (scan_result == 0) {
        wifi_count = 1;
        strncpy(wifi_ssid[0], "No networks", 31);
        wifi_ssid_ptrs[0] = wifi_ssid[0];
      } else {
        wifi_count = (scan_result > MAX_WIFI_NETWORKS) ? MAX_WIFI_NETWORKS : scan_result;
        for (int i = 0; i < wifi_count; i++) {
          String ssid = WiFi.SSID(i);
          strncpy(wifi_raw_ssid[i], ssid.c_str(), 31);
          wifi_raw_ssid[i][31] = '\0';
          
          long rssi = WiFi.RSSI(i);
          int quality = 0;
          if (rssi <= -100) quality = 0;
          else if (rssi >= -50) quality = 100;
          else quality = 2 * (rssi + 100);
          
          if (WiFi.status() == WL_CONNECTED && ssid == WiFi.SSID()) {
              snprintf(wifi_ssid[i], 32, "* %s [%d%%]", ssid.c_str(), quality);
          } else {
              snprintf(wifi_ssid[i], 32, "%s [%d%%]", ssid.c_str(), quality);
          }
          wifi_ssid_ptrs[i] = wifi_ssid[i];
        }
      }
      ui.setFullListCount(wifi_count);
      WiFi.scanDelete();
    } else if (scan_result == WIFI_SCAN_FAILED) {
      is_scanning_wifi = false;
      wifi_count = 1;
      strncpy(wifi_ssid[0], "Scan Failed", 31);
      wifi_ssid_ptrs[0] = wifi_ssid[0];
      ui.setFullListCount(wifi_count);
    }
  }

  // --- XỬ LÝ KẾT NỐI WIFI (NON-BLOCKING) ---
  if (is_connecting_wifi) {
      if (WiFi.status() == WL_CONNECTED) {
          is_connecting_wifi = false;
          
          // Mật khẩu đúng và kết nối thành công: Lưu trạng thái bật và SSID/PWD vào Flash EEPROM
          save_wifi_state(true, connecting_ssid, connecting_pwd);
          
          Serial.printf("\n[WiFi] Connected successfully to %s\n", connecting_ssid.c_str());
          
          // Hiển thị thông báo thành công
          ui.openModal("Connected!", connecting_ssid.c_str());
          
          // Dấu * sẽ được logic theo dõi ngầm ở trên tự động thêm vào!
          
      } else if (millis() - wifi_connect_start > 10000) {
          // Timeout sau 10 giây
          is_connecting_wifi = false;
          WiFi.disconnect();
          Serial.println("\n[WiFi] Connection timeout or failed");
          
          // Hiển thị lại Text Input với Mật khẩu cũ để người dùng sửa thay vì hiển thị Modal Failed
          snprintf(text_input_title_buf, sizeof(text_input_title_buf), "FAIL: %s", connecting_ssid.c_str());
          ui.openTextInput(text_input_title_buf, on_wifi_password_submit, connecting_pwd.c_str());
      }
  }

  // Cập nhật UI (60FPS)
  ui.update();
}
