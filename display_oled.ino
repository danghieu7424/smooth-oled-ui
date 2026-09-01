#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>
#include <WiFi.h>
#include <EEPROM.h>
#include <nvs_flash.h>
#include <HTTPClient.h>
#include <WiFiClientSecure.h>
#include <ArduinoJson.h>
#include "SmoothOLED.h"

int saved_brightness = 20;

// Clock State variables moved down

// Cấu trúc lưu toàn bộ hệ thống bằng EEPROM thô (chống mọi lỗi NVS)
struct SystemState {
    bool on;
    char ssid[32];
    char pwd[64];
    int brightness;
};

void save_system_state(bool on, int brightness, String ssid, String pwd) {
    SystemState state;
    state.on = on;
    state.brightness = brightness;
    strncpy(state.ssid, ssid.c_str(), 31);
    state.ssid[31] = '\0';
    strncpy(state.pwd, pwd.c_str(), 63);
    state.pwd[63] = '\0';
    EEPROM.put(0, state);
    EEPROM.commit();
}

SystemState load_system_state() {
    SystemState state;
    EEPROM.get(0, state);
    
    // Nếu EEPROM hoàn toàn trống (chưa từng ghi), ESP32 sẽ trả về toàn 255 (0xFF)
    if (state.ssid[0] == (char)255) {
        state.on = false;
        state.ssid[0] = '\0';
        state.pwd[0] = '\0';
    } else {
        // Ép kết thúc chuỗi an toàn
        state.ssid[31] = '\0';
        state.pwd[63] = '\0';
    }
    
    // Đảm bảo brightness luôn nằm trong khoảng hợp lệ
    if (state.brightness < 0 || state.brightness > 255) {
        state.brightness = 20;
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
void open_home_clock();

const MenuItem menu_items[] = {
    {"Home", icon_home, open_home_clock},
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

// Clock State
int current_hour = 0;
int current_minute = 0;
int current_second = 0;
String solar_date_str = "";
String lunar_date_str = "";
uint32_t last_time_sync = 0;
uint32_t last_second_tick = 0;
bool is_time_synced = false;

void sync_time_with_api() {
    if (WiFi.status() == WL_CONNECTED) {
        WiFiClientSecure *client = new WiFiClientSecure;
        client->setInsecure();
        HTTPClient http;
        if (http.begin(*client, "https://dh74.io.vn/api/time?tz=7")) {
            int httpCode = http.GET();
            if (httpCode == HTTP_CODE_OK) {
                String payload = http.getString();
                JsonDocument doc;
                DeserializationError error = deserializeJson(doc, payload);
                if (!error) {
                    current_hour = doc["data"]["time"]["hour"];
                    current_minute = doc["data"]["time"]["minute"];
                    current_second = doc["data"]["time"]["second"];
                    
                    int s_d = doc["data"]["solar"]["day"];
                    int s_m = doc["data"]["solar"]["month"];
                    int s_y = doc["data"]["solar"]["year"];
                    char buf[32];
                    snprintf(buf, sizeof(buf), "%02d/%02d/%04d", s_d, s_m, s_y);
                    solar_date_str = String(buf);
                    
                    int l_d = doc["data"]["lunar"]["day"];
                    int l_m = doc["data"]["lunar"]["month"];
                    const char* l_y_can_chi = doc["data"]["lunar"]["year_can_chi"];
                    char l_buf[64];
                    snprintf(l_buf, sizeof(l_buf), "Am lich: %02d/%02d %s", l_d, l_m, l_y_can_chi);
                    lunar_date_str = String(l_buf);
                    
                    is_time_synced = true;
                    last_time_sync = millis();
                    last_second_tick = millis();
                    ui.updateClock(current_hour, current_minute, current_second, solar_date_str.c_str(), lunar_date_str.c_str());
                }
            }
            http.end();
        }
        delete client;
    }
}

void open_home_clock() {
  ui.openClock();
  ui.updateClock(current_hour, current_minute, current_second, solar_date_str.c_str(), lunar_date_str.c_str());
  if (!is_time_synced && WiFi.status() == WL_CONNECTED) {
      sync_time_with_api();
  }
}

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
      SystemState state = load_system_state();
      
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
  
  // Tự động khôi phục phân vùng NVS bị hỏng (Lý do cốt lõi khiến Flash không lưu được)
  esp_err_t err = nvs_flash_init();
  if (err == ESP_ERR_NVS_NO_FREE_PAGES || err == ESP_ERR_NVS_NEW_VERSION_FOUND) {
      nvs_flash_erase();
      nvs_flash_init();
  }
  
  // Nạp cấu hình từ Flash
  EEPROM.begin(512); // Khởi tạo vùng nhớ EEPROM thô để lưu WiFi và Brightness
  
  SystemState state = load_system_state();
  saved_brightness = state.brightness;
  current_brightness = saved_brightness;

  Wire.begin(47, 48);
  Wire.setClock(400000); 
  u8g2.begin();
  u8g2.setContrast(current_brightness); // Áp dụng độ sáng đã lưu

  // Khởi động lại: Xem trạng thái trước đó có đang kết nối không
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
                        SystemState state = load_system_state();
                        save_system_state(false, saved_brightness, state.ssid, state.pwd); // Chỉ cập nhật cờ on thành false
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
                      SystemState state = load_system_state();
                      save_system_state(false, saved_brightness, state.ssid, state.pwd); // Cập nhật trạng thái tắt kết nối vào Flash
                      
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
                  // [Cập nhật]: Lưu độ sáng vào Flash EEPROM
                  saved_brightness = current_brightness;
                  SystemState state = load_system_state();
                  save_system_state(state.on, saved_brightness, state.ssid, state.pwd);
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
          save_system_state(true, saved_brightness, connecting_ssid, connecting_pwd);
          
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

  // --- CLOCK TICK LOGIC ---
  if (is_time_synced) {
      if (millis() - last_second_tick >= 1000) {
          last_second_tick += 1000;
          current_second++;
          if (current_second >= 60) {
              current_second = 0;
              current_minute++;
              if (current_minute >= 60) {
                  current_minute = 0;
                  current_hour++;
                  if (current_hour >= 24) {
                      current_hour = 0;
                  }
              }
          }
          ui.updateClock(current_hour, current_minute, current_second, solar_date_str.c_str(), lunar_date_str.c_str());
      }
      
      // Đồng bộ lại với API mỗi 1 giờ để bù sai số và cập nhật ngày
      if (millis() - last_time_sync > 3600000) {
          sync_time_with_api();
      }
  }

  // Cập nhật UI (60FPS)
  ui.update();
}
