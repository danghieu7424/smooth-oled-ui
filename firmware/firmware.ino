#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>
#include <WiFi.h>
#include <EEPROM.h>
#include <nvs_flash.h>
#include <HTTPClient.h>
#include <WiFiClientSecure.h>
#include <ArduinoJson.h>
#include "src/SmoothOLED/SmoothOLED.h"
#include "src/TimeSyncAPI/TimeSyncAPI.h"
#include "src/HardwareRTC/HardwareRTC.h"
#include "src/ExternalEEPROM/ExternalEEPROM.h"
#include "src/OLED_OTA/OLED_OTA.h"

// ==========================================
// CẤU HÌNH DỰ ÁN TỪ OTA HUB DASHBOARD
// ==========================================
const char* PROJECT_ID = "007Rlq30Q2vU-esp32-tool";
const char* PROJECT_TOKEN = "57c510c895b69f432be01fd9a8bc9d51";
const char* CURRENT_VERSION = "1.0.1";
const char* MQTT_BROKER = "192.168.7.7";
const uint16_t MQTT_PORT = 1883;

OLED_OTA ota(PROJECT_ID, PROJECT_TOKEN, CURRENT_VERSION);

int saved_brightness = 20;

#define LED_PIN 2
int saved_led_state = 0;

enum ActiveSlider { SLIDER_NONE, SLIDER_BRIGHTNESS, SLIDER_LED_SWITCH };
ActiveSlider active_slider = SLIDER_NONE;

// Clock State variables moved down

// Clock State variables moved down

void save_wifi_credentials(String ssid, String pwd) {
    extEEPROM.writeString(0x0010, ssid);
    extEEPROM.writeString(0x0040, pwd);
    extEEPROM.writeByte(0x000F, 0xAA); // Signature
}

bool load_wifi_credentials(String &ssid, String &pwd) {
    if (extEEPROM.readByte(0x000F) == 0xAA) {
        ssid = extEEPROM.readString(0x0010, 32);
        pwd = extEEPROM.readString(0x0040, 64);
        return true;
    }
    return false;
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

// [MỚI] Icon LED - Vẽ trên khung 24x24 px
static const unsigned char icon_led_switch[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
  0xf8, 0xff, 0x1f, 0xfc, 0xff, 0x3f, 0x0c, 0x00, 0x30, 0x0c, 0x00, 0x30, 
  0x0c, 0x00, 0x30, 0x8c, 0x01, 0x30, 0x8c, 0x03, 0x30, 0x0c, 0x07, 0x30, 
  0x0c, 0x07, 0x30, 0x8c, 0x03, 0x30, 0x8c, 0xf1, 0x33, 0x0c, 0xf0, 0x33, 
  0x0c, 0x00, 0x30, 0x0c, 0x00, 0x30, 0xfc, 0xff, 0x3f, 0xf8, 0xff, 0x1f, 
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

// --- KHAI BÁO CÁC HÀM XỬ LÝ SỰ KIỆN (CALLBACKS) ---
void open_settings_menu();
void open_brightness_slider();
void open_led_switch();
void on_led_change(int val);
void on_brightness_change(int val);
void on_enter_wifi();
void on_wifi_password_submit(const char* pwd);
void open_home_clock();

void open_about_menu();

const MenuItem menu_items[] = {
    {"Home", icon_home, open_home_clock},
    {"Settings", icon_settings, open_settings_menu},
    {"About", icon_about, open_about_menu}
};
const int TOTAL_MAIN_ITEMS = 3;

const MenuItem settings_items[] = {
    {"WiFi", icon_wifi, on_enter_wifi},
    {"ESP NOW", icon_esp_now, nullptr},
    {"LED Switch", icon_led_switch, open_led_switch},
    {"Brightness", icon_brightness, open_brightness_slider}
};
const int TOTAL_SETTINGS_ITEMS = 4;

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

const char* about_items[8];
char about_buf[8][64];

void open_about_menu() {
    snprintf(about_buf[0], sizeof(about_buf[0]), "Chip: ESP32-S3");
    snprintf(about_buf[1], sizeof(about_buf[1]), "Model: %s", ESP.getChipModel());
    snprintf(about_buf[2], sizeof(about_buf[2]), "Rev: %d", ESP.getChipRevision());
    snprintf(about_buf[3], sizeof(about_buf[3]), "Flash: %d MB", ESP.getFlashChipSize() / (1024 * 1024));
    snprintf(about_buf[4], sizeof(about_buf[4]), "RAM: %d KB", ESP.getHeapSize() / 1024);
    snprintf(about_buf[5], sizeof(about_buf[5]), "MAC: %s", WiFi.macAddress().c_str());
    snprintf(about_buf[6], sizeof(about_buf[6]), "SDK: %s", ESP.getSdkVersion());
    snprintf(about_buf[7], sizeof(about_buf[7]), "Ver: %s", CURRENT_VERSION);

    for (int i = 0; i < 8; i++) {
        about_items[i] = about_buf[i];
    }
    
    ui.openFullList("About MCU", about_items, 8, nullptr);
}

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



void open_home_clock() {
  ui.openClock();
  ui.updateClock(timeSync.current_hour, timeSync.current_minute, timeSync.current_second, timeSync.solar_date_str.c_str(), timeSync.lunar_date_str.c_str(), timeSync.current_temp_str.c_str());
  if (!timeSync.api_synced && WiFi.status() == WL_CONNECTED) {
      timeSync.update();
  }
}

void open_settings_menu() {
  current_level = LEVEL_SETTINGS;
  ui.setCarouselItems(settings_items, TOTAL_SETTINGS_ITEMS, "< SETTINGS >");
}

void open_brightness_slider() {
  active_slider = SLIDER_BRIGHTNESS;
  ui.openSlider("Brightness", current_brightness, 255, on_brightness_change);
}

void on_brightness_change(int val) {
  current_brightness = val;
  u8g2.setContrast(current_brightness); // Lệnh phần cứng đổi độ sáng OLED trực tiếp
}

void open_led_switch() {
  active_slider = SLIDER_LED_SWITCH;
  ui.openSlider("LED Switch", saved_led_state, 1, on_led_change);
}

void on_led_change(int val) {
  saved_led_state = val;
  digitalWrite(LED_PIN, val == 1 ? HIGH : LOW);
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

      // KIỂM TRA: Nếu mạng này TRÙNG với mạng đã lưu trong AT24C256
      String saved_pwd = "";
      String saved_ssid = "";
      if (load_wifi_credentials(saved_ssid, saved_pwd)) {
          if (String(wifi_raw_ssid[idx]) != saved_ssid) {
              saved_pwd = ""; // Nếu không khớp SSID thì không dùng pwd này
          }
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
      
      // [MỚI] Lưu Credentials vào AT24C256
      save_wifi_credentials(connecting_ssid, connecting_pwd);
      
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
  
  // Tự động khôi phục phân vùng NVS bị hỏng (Lý do cốt lý khiến Flash không lưu được)
  esp_err_t err = nvs_flash_init();
  if (err == ESP_ERR_NVS_NO_FREE_PAGES || err == ESP_ERR_NVS_NEW_VERSION_FOUND) {
      nvs_flash_erase();
      nvs_flash_init();
  }
  
  Wire.begin(47, 48);
  Wire.setClock(400000); 
  
  pinMode(LED_PIN, OUTPUT);

  // --- Khởi tạo và kiểm tra RTC & EEPROM ---
  if (rtc.begin()) {
      Serial.println("[HW] DS3231 RTC found!");
  } else {
      Serial.println("[HW] DS3231 RTC NOT found!");
  }
  
  bool eeprom_ready = extEEPROM.begin();
  if (eeprom_ready) {
      Serial.println("[HW] AT24C256 EEPROM found!");
      uint8_t at24_b = extEEPROM.readByte(0x0000);
      if (at24_b != 0xFF) {
          saved_brightness = at24_b;
          current_brightness = saved_brightness;
          Serial.printf("[HW] Loaded brightness %d from AT24C256\n", saved_brightness);
      }
  } else {
      Serial.println("[HW] AT24C256 EEPROM NOT found!");
  }

  u8g2.begin();
  u8g2.setContrast(current_brightness); // Áp dụng độ sáng đã lưu

  // --- KẾT NỐI WIFI MẶC ĐỊNH ---
  WiFi.mode(WIFI_STA);
  WiFi.disconnect(true); // Xóa state lơ lửng của phần cứng
  delay(100);
  
  String saved_ssid = "";
  String saved_pwd = "";
  if (eeprom_ready && load_wifi_credentials(saved_ssid, saved_pwd)) {
      connecting_ssid = saved_ssid;
      connecting_pwd = saved_pwd;
      Serial.printf("[BOOT] Using AT24C256 WiFi: SSID='%s'\n", connecting_ssid.c_str());
  } else {
      Serial.printf("[BOOT] Using HARDCODED WiFi: SSID='%s'\n", connecting_ssid.c_str());
  }
  
  WiFi.begin(connecting_ssid.c_str(), connecting_pwd.c_str());
  WiFi.setAutoReconnect(true); // Tự động kết nối lại nếu rớt mạng

  // 1. Gán mảng dữ liệu vào thư viện UI
  ui.setCarouselItems(menu_items, TOTAL_MAIN_ITEMS, "< MAIN MENU >");
  ui.setPopupListItems(popup_items, TOTAL_POPUP_ITEMS);
  ui.setSidePopupItems(side_items, TOTAL_SIDE_ITEMS);

  // 2. Tắt chế độ xuất khung hình ra Serial cho PC Viewer (Mặc định OFF)
  ui.enableAutoDemo(false);
  ui.enablePCViewer(false);

  // 3. Khởi động UI
  ui.begin();

  // 4. Cấu hình TimeSync và mở Màn hình Đồng hồ
  timeSync.begin(7);
  if (rtc.begin()) {
      timeSync.syncFromRTC();
  }
  
  pinMode(LED_PIN, OUTPUT);
  digitalWrite(LED_PIN, saved_led_state == 1 ? HIGH : LOW);
  
  open_home_clock();

  // 5. Khởi tạo OTA Service (Nó sẽ tự kết nối MQTT khi WiFi sẵn sàng ở loop)
  ota.setMqttBroker(MQTT_BROKER, MQTT_PORT);
  ota.begin();
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
            else if (cmd == 'V') ui.enablePCViewer(true);
            else if (cmd == 'v') ui.enablePCViewer(false);
            else if (cmd == 'C') { // Phím Esc
              if (ui.isOverlayOpen()) {
                ui.closeOverlay(); // Đóng Side List
              } else if (ui.getAppState() == STATE_TEXT_INPUT) {
                ui.closeOverlay(); // Esc -> Thoát thẳng nhập Pass
              } else if (ui.getAppState() == STATE_POPUP || ui.getAppState() == STATE_MODAL) {
                ui.closeOverlay(); // Đóng Popup/Modal
              } else if (ui.getAppState() == STATE_CLOCK) {
                ui.closeOverlay(); // Thoát khỏi đồng hồ về Menu chính
              } else if (ui.getAppState() == STATE_FULL_LIST) {
                if (current_level == LEVEL_WIFI) {
                    current_level = LEVEL_SETTINGS; // Lùi về Settings
                    ui.setCarouselItems(settings_items, TOTAL_SETTINGS_ITEMS, "< SETTINGS >");
                    // Nếu thoát ra mà không có kết nối nào, tắt Wi-Fi để tiết kiệm pin
                    if (WiFi.status() != WL_CONNECTED) {
                        WiFi.mode(WIFI_OFF);
                        extEEPROM.writeByte(0x000F, 0x00); // Tắt kết nối tự động
                    }
                }
                ui.closeOverlay();
              } else if (ui.getAppState() == STATE_SLIDER) {
                // [Cập nhật]: Hủy bỏ, khôi phục trạng thái cũ dựa theo Slider nào đang mở
                if (active_slider == SLIDER_BRIGHTNESS) {
                    current_brightness = saved_brightness;
                    u8g2.setContrast(current_brightness);
                } else if (active_slider == SLIDER_LED_SWITCH) {
                  open_led_switch();
                }
                active_slider = SLIDER_NONE;
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
                      extEEPROM.writeByte(0x000F, 0x00); // Tắt kết nối tự động
                      
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
                  // [Cập nhật]: Chỉ lưu giá trị của công tắc/thanh gạt đang mở
                  if (active_slider == SLIDER_BRIGHTNESS) {
                      saved_brightness = current_brightness; // Lấy từ biến cục bộ đã được callback cập nhật
                      extEEPROM.writeByte(0x0000, (uint8_t)saved_brightness);
                  } else if (active_slider == SLIDER_LED_SWITCH) {
                      on_led_change(saved_led_state);
                  }
                  active_slider = SLIDER_NONE;
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
          
          // Mật khẩu đúng và kết nối thành công: Lưu SSID/PWD vào AT24C256
          save_wifi_credentials(connecting_ssid, connecting_pwd);
          
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

  // --- CLOCK TICK & SYNC LOGIC ---
  if (timeSync.tick()) {
      ui.updateClock(timeSync.current_hour, timeSync.current_minute, timeSync.current_second, timeSync.solar_date_str.c_str(), timeSync.lunar_date_str.c_str(), timeSync.current_temp_str.c_str());
  }
  
  // --- DUY TRÌ KẾT NỐI OTA ---
  ota.loop();
  
  if (timeSync.api_synced) {
      // Đồng bộ lại với API mỗi 1 giờ để bù sai số và cập nhật ngày
      if (millis() - timeSync.last_time_sync > 3600000) {
          timeSync.update();
      }
  } else {
      // Chưa đồng bộ được thời gian từ web -> Thử đồng bộ liên tục mỗi 2 giây nếu có mạng
      static uint32_t last_sync_try = 0;
      if (millis() - last_sync_try > 2000) {
          last_sync_try = millis();
          if (WiFi.status() == WL_CONNECTED) {
              timeSync.update();
          }
      }
  }

  // Cập nhật UI (60FPS)
  ui.update();
}
