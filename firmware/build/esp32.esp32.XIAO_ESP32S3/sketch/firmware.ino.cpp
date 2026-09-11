#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>
#include <WiFi.h>
#include <EEPROM.h>
#include <nvs_flash.h>
#include <HTTPClient.h>
#include <WiFiClientSecure.h>
#include <ArduinoJson.h>
#include <WiFiMulti.h>
#include "src/SmoothOLED/SmoothOLED.h"
#include "src/TimeSyncAPI/TimeSyncAPI.h"
#include "src/HardwareRTC/HardwareRTC.h"
#include "src/ExternalEEPROM/ExternalEEPROM.h"
#include "src/OLED_OTA/OLED_OTA.h"
#include "src/WebConfig/WebConfig.h"
#include "src/WeatherAPI/WeatherAPI.h"
#include "src/ESPNowHub/ESPNowHub.h"

OLED_OTA ota("oled_project", "token123", "1.1.0");
WiFiMulti wifiMulti;

// ==========================================
// CẤU HÌNH DỰ ÁN
// ==========================================
int saved_brightness = 20;

#define LED_PIN 2
int saved_led_state = 0;

enum ActiveSlider { SLIDER_NONE, SLIDER_BRIGHTNESS, SLIDER_LED_SWITCH };
ActiveSlider active_slider = SLIDER_NONE;

// Clock State variables moved down

// Clock State variables moved down

#line 38 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void save_wifi_credentials(String ssid, String pwd);
#line 66 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
bool get_saved_password(String target_ssid, String &pwd);
#line 79 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void remove_wifi_credentials(String ssid);
#line 217 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void on_restart();
#line 221 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void on_power_off();
#line 382 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void setup();
#line 493 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void loop();
#line 38 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void save_wifi_credentials(String ssid, String pwd) {
    uint8_t count = extEEPROM.readByte(0x000F);
    if (count == 0xFF || count > 5) count = 0;
    
    int existing_idx = -1;
    for (int i = 0; i < count; i++) {
        String saved_ssid = extEEPROM.readString(0x0010 + i * 96, 32);
        if (saved_ssid == ssid) {
            existing_idx = i;
            break;
        }
    }
    
    if (existing_idx != -1) {
        extEEPROM.writeString(0x0010 + existing_idx * 96 + 32, pwd);
    } else {
        int new_idx = count;
        if (new_idx >= 5) {
            new_idx = 0;
        } else {
            count++;
            extEEPROM.writeByte(0x000F, count);
        }
        extEEPROM.writeString(0x0010 + new_idx * 96, ssid);
        extEEPROM.writeString(0x0010 + new_idx * 96 + 32, pwd);
    }
}

bool get_saved_password(String target_ssid, String &pwd) {
    uint8_t count = extEEPROM.readByte(0x000F);
    if (count == 0xFF || count > 5) return false;
    for (int i = 0; i < count; i++) {
        String saved_ssid = extEEPROM.readString(0x0010 + i * 96, 32);
        if (saved_ssid == target_ssid) {
            pwd = extEEPROM.readString(0x0010 + i * 96 + 32, 64);
            return true;
        }
    }
    return false;
}

void remove_wifi_credentials(String ssid) {
    uint8_t count = extEEPROM.readByte(0x000F);
    if (count == 0xFF || count > 5) count = 0;
    
    for (int i = 0; i < count; i++) {
        String saved_ssid = extEEPROM.readString(0x0010 + i * 96, 32);
        if (saved_ssid == ssid) {
            for (int j = i; j < count - 1; j++) {
                String move_ssid = extEEPROM.readString(0x0010 + (j + 1) * 96, 32);
                String move_pwd = extEEPROM.readString(0x0010 + (j + 1) * 96 + 32, 64);
                extEEPROM.writeString(0x0010 + j * 96, move_ssid);
                extEEPROM.writeString(0x0010 + j * 96 + 32, move_pwd);
            }
            count--;
            extEEPROM.writeByte(0x000F, count);
            break;
        }
    }
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

// [Mới] Icon WiFi - Vẽ trên khung 24x24 px
static const unsigned char icon_wifi[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
  0x80, 0xff, 0x01, 0xe0, 0xff, 0x07, 0xf0, 0x00, 0x0f, 0x38, 0x00, 0x1c,  
  0x1c, 0xff, 0x38, 0xc0, 0xff, 0x03, 0xe0, 0x81, 0x07, 0x70, 0x00, 0x0e,  
  0x00, 0x7e, 0x00, 0x80, 0xff, 0x01, 0x80, 0xc3, 0x01, 0x00, 0x00, 0x00,  
  0x00, 0x18, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x18, 0x00,  
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

// [Mới] Icon esp now - Vẽ trên khung 24x24 px
static const unsigned char icon_esp_now[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x02, 0x60, 0x00, 0x06,  
  0x30, 0x00, 0x0c, 0x10, 0x42, 0x08, 0x18, 0x81, 0x18, 0x18, 0x99, 0x18,  
  0x18, 0x99, 0x18, 0x18, 0x81, 0x18, 0x10, 0x42, 0x08, 0x30, 0x18, 0x0c,  
  0x60, 0x18, 0x06, 0x40, 0x18, 0x02, 0x00, 0x18, 0x00, 0x00, 0x18, 0x00,  
  0x00, 0x18, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x3c, 0x00,  
  0x00, 0x7e, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

// [Mới] Icon LED - Vẽ trên khung 24x24 px
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
    {"ESP NOW", icon_esp_now, []() { espNowHub.broadcastToggle(); }},
    {"LED Switch", icon_led_switch, open_led_switch},
    {"Brightness", icon_brightness, open_brightness_slider}
};
const int TOTAL_SETTINGS_ITEMS = 4;

const char* wifi_more_items[] = {
    "Disconnect",
    "Remove Password",
    "Cancel"
};

bool wifi_manual_disconnected = false;

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

const char* about_items[9];
char about_buf[9][64];

void open_about_menu() {
    snprintf(about_buf[0], sizeof(about_buf[0]), "Chip: ESP32-S3");
    snprintf(about_buf[1], sizeof(about_buf[1]), "Model: %s", ESP.getChipModel());
    snprintf(about_buf[2], sizeof(about_buf[2]), "Rev: %d", ESP.getChipRevision());
    snprintf(about_buf[3], sizeof(about_buf[3]), "Flash: %d MB", ESP.getFlashChipSize() / (1024 * 1024));
    snprintf(about_buf[4], sizeof(about_buf[4]), "RAM: %d KB", ESP.getHeapSize() / 1024);
    snprintf(about_buf[5], sizeof(about_buf[5]), "MAC: %s", WiFi.macAddress().c_str());
    snprintf(about_buf[6], sizeof(about_buf[6]), "IP: %s", WiFi.localIP().toString().c_str());
    snprintf(about_buf[7], sizeof(about_buf[7]), "SDK: %s", ESP.getSdkVersion());
    snprintf(about_buf[8], sizeof(about_buf[8]), "Ver: %s", "1.1.0");

    for (int i = 0; i < 9; i++) {
        about_items[i] = about_buf[i];
    }
    
    ui.openFullList("About MCU", about_items, 9, nullptr);
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
  ui.updateClock(timeSync.current_hour, timeSync.current_minute, timeSync.current_second, timeSync.solar_date_str.c_str(), timeSync.lunar_date_str.c_str(), timeSync.current_temp_str.c_str(), weatherApi.icon_code.c_str());
  // Bỏ gọi timeSync.update() ở đây vì Core 1 đã có một vòng lặp tự động xử lý ngầm (polling mỗi 2 giây nếu chưa sync)
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
  u8g2.setContrast(current_brightness); // // Lệnh phần cứng đổi độ sáng OLED trực tiếp
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
          ui.openModal("Connected!", "Already connected to this network", true);
          return;
      }

      String saved_pwd = "";
      get_saved_password(String(wifi_raw_ssid[idx]), saved_pwd);
      
      // Mở ô nhập Pass và ĐIỀN SẴN mật khẩu cũ (như thẻ input type="text" có value)
      // Người dùng chỉ cần ấn Enter để kết nối, hoặc ấn xóa để sửa
      snprintf(text_input_title_buf, sizeof(text_input_title_buf), "PWD: %s", wifi_raw_ssid[idx]);
      ui.openTextInput(text_input_title_buf, on_wifi_password_submit, saved_pwd.c_str());
  }
}

void on_enter_wifi() {
  current_level = LEVEL_WIFI;
  
  // Khởi tạo UI hộp thoại tạm thời "Scanning..."
  wifi_count = 1;
  strncpy(wifi_ssid[0], "Scanning...", 31);
  wifi_ssid_ptrs[0] = wifi_ssid[0];
  ui.openFullList("WIFI NETWORKS", wifi_ssid_ptrs, wifi_count, on_wifi_selected);

  if (is_scanning_wifi) return; // // Đang quét thì không kích hoạt lại
  
  WiFi.mode(WIFI_STA);
  // XÓA WiFi.disconnect() ở đây để không làm rớt mạng đang kết nối khi load lại menu
  
  WiFi.scanNetworks(true); // // Quét bất đồng bộ (Async)
  is_scanning_wifi = true;
}

void on_wifi_password_submit(const char* pwd) {
  int idx = ui.getFullListSelectedIndex();
  if (idx >= 0 && idx < wifi_count) {
      connecting_ssid = wifi_raw_ssid[idx];
      connecting_pwd = pwd;
      
      save_wifi_credentials(connecting_ssid, connecting_pwd);
      wifiMulti.addAP(connecting_ssid.c_str(), connecting_pwd.c_str());
      
      wifi_manual_disconnected = false;
      Serial.printf("\n[WiFi] Connecting to %s with password: %s\n", connecting_ssid.c_str(), pwd);
      
      // Ngắt kết nối cũ (nếu có)
      WiFi.disconnect();
      delay(100);
      WiFi.begin(connecting_ssid.c_str(), pwd);
      
      is_connecting_wifi = true;
      wifi_connect_start = millis();
      
      // [MỚI] Cập nhật trạng thái Connecting... lên màn hình
      ui.openModal("Connecting...", connecting_ssid.c_str());
  }
}

SemaphoreHandle_t wifi_mutex = NULL;

void task_ui_core0(void *pvParameters);
void task_network_core1(void *pvParameters);

void setup() {
  Serial.begin(921600);
  
  pinMode(LED_PIN, OUTPUT);

  // --- Khởi tạo I2C Bus 0 cho OLED (Core 0) ---
  Wire.begin(4, 5);
  Wire.setClock(400000); 

  // --- Khởi tạo và kiểm tra RTC & EEPROM (Core 1 sẽ dùng I2C1 / Wire1) ---
  // Khởi tạo Bus 1 (Sensor) ở đây để các module gọi begin() thành công
  Wire1.begin(6, 7);
  Wire1.setClock(100000);

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
  u8g2.setContrast(current_brightness);

  // --- KẾT NỐI WIFI MẶC ĐỊNH ---
  WiFi.mode(WIFI_STA);
  WiFi.disconnect(true);
  delay(100);
  
  uint8_t wifi_cnt = extEEPROM.readByte(0x000F);
  if (wifi_cnt != 0xFF && wifi_cnt <= 5) {
      for (int i = 0; i < wifi_cnt; i++) {
          String s_ssid = extEEPROM.readString(0x0010 + i * 96, 32);
          String s_pwd = extEEPROM.readString(0x0010 + i * 96 + 32, 64);
          wifiMulti.addAP(s_ssid.c_str(), s_pwd.c_str());
          Serial.printf("[BOOT] Loaded WiFi: SSID='%s'\n", s_ssid.c_str());
      }
  } else {
      wifiMulti.addAP(connecting_ssid.c_str(), connecting_pwd.c_str());
      Serial.printf("[BOOT] Default WiFi: SSID='%s'\n", connecting_ssid.c_str());
  }

  WiFi.setAutoReconnect(true);

  // 1. Gán mảng dữ liệu vào thư viện UI
  ui.setCarouselItems(menu_items, TOTAL_MAIN_ITEMS, "< MAIN MENU >");
  ui.setPopupListItems(popup_items, TOTAL_POPUP_ITEMS);
  ui.setSidePopupItems(side_items, TOTAL_SIDE_ITEMS);

  // 2. Cấu hình UI
  ui.enableAutoDemo(false);
  ui.enablePCViewer(false);

  // 3. Khởi động UI
  ui.begin();

  // 4. Cấu hình TimeSync
  timeSync.begin(7);
  if (rtc.begin()) {
      timeSync.syncFromRTC();
  }
  
  pinMode(LED_PIN, OUTPUT);
  digitalWrite(LED_PIN, saved_led_state == 1 ? HIGH : LOW);
  
  open_home_clock();

  // 5. Khởi tạo OTA Service & Mở rộng
  ota.setApiEndpoint("api.github.com", 443);
  ota.begin();
  weatherApi.setApiKey("b05cb47c6258d12e9b43a7419e8cbdf9", "Hanoi");
  espNowHub.begin();

  // Tạo Mutex cho các biến dùng chung
  wifi_mutex = xSemaphoreCreateMutex();

  // Task UI trên Core 0
  xTaskCreatePinnedToCore(
      task_network_core1,
      "Task_UI",
      8192,
      NULL,
      1,
      NULL,
      0
  );

  // Task Network trên Core 1
  xTaskCreatePinnedToCore(
      task_ui_core0,
      "Task_Network",
      8192,
      NULL,
      1,
      NULL,
      1
  );
}

void loop() {
    // Xóa task loop của Arduino để giải phóng tài nguyên
    vTaskDelete(NULL);
}

// ==========================================
// TASK: UI & ANIMATION (CORE 0)
// ==========================================
void task_ui_core0(void *pvParameters) {
    uint32_t last_interaction_time = millis();
    for (;;) {
        // 1. Xử lý Input từ Serial (Nút bấm mô phỏng)
        if (Serial.available() > 0) {
            last_interaction_time = millis();
            ui.enableScreensaver(false);
            char c = Serial.read();
            if (c == '\x1B') {
                uint32_t t = millis();
                while (!Serial.available() && millis() - t < 50) { vTaskDelay(1); }
                if (Serial.available()) {
                    char cmd = Serial.read();
                    if (cmd == 'U') ui.up();
                    else if (cmd == 'D') ui.down();
                    else if (cmd == 'L') {
                        if (ui.getAppState() == STATE_MODAL && current_level == LEVEL_WIFI) {
                            ui.closeOverlay();
                            ui.setPopupListItems(wifi_more_items, 3);
                            ui.openPopup();
                        } else {
                            ui.left();
                        }
                    }
                    else if (cmd == 'R') ui.right();
                    else if (cmd == 'P') {
                        ui.setPopupListItems(popup_items, TOTAL_POPUP_ITEMS);
                        ui.openPopup();
                    }
                    else if (cmd == 'S') ui.openSideList();
                    else if (cmd == 'V') ui.enablePCViewer(true);
                    else if (cmd == 'v') ui.enablePCViewer(false);
                    else if (cmd == 'C') {
                        if (ui.isOverlayOpen()) {
                            ui.closeOverlay();
                        } else if (ui.getAppState() == STATE_TEXT_INPUT) {
                            ui.closeOverlay();
                        } else if (ui.getAppState() == STATE_POPUP || ui.getAppState() == STATE_MODAL) {
                            ui.closeOverlay();
                        } else if (ui.getAppState() == STATE_CLOCK) {
                            ui.closeOverlay();
                        } else if (ui.getAppState() == STATE_FULL_LIST) {
                            if (current_level == LEVEL_WIFI) {
                                current_level = LEVEL_SETTINGS;
                                ui.setCarouselItems(settings_items, TOTAL_SETTINGS_ITEMS, "< SETTINGS >");
                                if (WiFi.status() != WL_CONNECTED) {
                                    WiFi.mode(WIFI_OFF);
                                    extEEPROM.writeByte(0x000F, 0x00);
                                }
                            }
                        } else if (ui.getAppState() == STATE_SLIDER) {
                            if (active_slider == SLIDER_BRIGHTNESS) {
                                current_brightness = saved_brightness;
                                u8g2.setContrast(current_brightness);
                            } else if (active_slider == SLIDER_LED_SWITCH) {
                                open_led_switch();
                            }
                            active_slider = SLIDER_NONE;
                        } else if (ui.getAppState() == STATE_CAROUSEL && current_level == LEVEL_SETTINGS) {
                            current_level = LEVEL_MAIN;
                            ui.setCarouselItems(menu_items, TOTAL_MAIN_ITEMS, "< MAIN MENU >");
                        }
                        ui.closeOverlay();
                    }
                }
            }
            else if (c == '\b' || c == 127) {
                if (ui.getAppState() == STATE_TEXT_INPUT) {
                    ui.backspace();
                } else if (current_level == LEVEL_WIFI) {
                    if (WiFi.status() == WL_CONNECTED) {
                        WiFi.disconnect();
                        wifi_manual_disconnected = true;
                        if (xSemaphoreTake(wifi_mutex, pdMS_TO_TICKS(100)) == pdTRUE) {
                            for (int i = 0; i < wifi_count; i++) {
                                if (wifi_ssid[i][0] == '*') {
                                    String temp = String(wifi_ssid[i]).substring(2);
                                    strncpy(wifi_ssid[i], temp.c_str(), 31);
                                    wifi_ssid[i][31] = '\0';
                                }
                            }
                            xSemaphoreGive(wifi_mutex);
                        }
                        ui.openModal("Disconnected!", "Đã ngắt kết nối tạm thời");
                    }
                }
            }
            else if (c == '\n' || c == '\r') {
                AppState before_select = ui.getAppState();
                ui.select();
                
                if (before_select == STATE_POPUP && current_level == LEVEL_WIFI) {
                    int sel = ui.getPopupSelectedIndex();
                    String current_ssid = WiFi.SSID();
                    if (sel == 0) { // Disconnect
                        WiFi.disconnect();
                        wifi_manual_disconnected = true;
                        ui.openModal("Disconnected!", "Đã ngắt kết nối tạm thời");
                    } else if (sel == 1) { // Remove
                        WiFi.disconnect();
                        remove_wifi_credentials(current_ssid);
                        wifi_manual_disconnected = true;
                        ui.openModal("Removed!", "Đã xóa khỏi bộ nhớ EEPROM");
                    }
                }
                else if (ui.getAppState() == STATE_SLIDER) {
                    if (active_slider == SLIDER_BRIGHTNESS) {
                        saved_brightness = current_brightness;
                        extEEPROM.writeByte(0x0000, (uint8_t)saved_brightness);
                    } else if (active_slider == SLIDER_LED_SWITCH) {
                        on_led_change(saved_led_state);
                    }
                    active_slider = SLIDER_NONE;
                    ui.closeOverlay();
                } else if (!ui.isOverlayOpen() && ui.getAppState() == STATE_CAROUSEL) {
                    const MenuItem* active_item = ui.getCurrentMenuItem();
                    if (active_item && active_item->on_enter) {
                        active_item->on_enter();
                    }
                }
            } else {
                ui.inputChar(c);
            }
        }

        if (millis() - last_interaction_time > 30 * 60 * 1000) {
            esp_deep_sleep_start();
        } else if (millis() - last_interaction_time > 5 * 60 * 1000) {
            ui.enableScreensaver(true);
        }

        // 2. Logic Đồng hồ (Tick)
        if (timeSync.tick() || weatherApi.is_synced) {
            weatherApi.is_synced = false;
            ui.updateClock(timeSync.current_hour, timeSync.current_minute, timeSync.current_second, timeSync.solar_date_str.c_str(), timeSync.lunar_date_str.c_str(), timeSync.current_temp_str.c_str(), weatherApi.icon_code.c_str());
        }

        // 3. Vẽ lên màn hình OLED (Render)
        ui.update();

        // 4. Delay để giữ 60FPS
        vTaskDelay(pdMS_TO_TICKS(16));
    }
}

// ==========================================
// TASK: NETWORK & BACKGROUND (CORE 1)
// ==========================================
void task_network_core1(void *pvParameters) {
    static wl_status_t last_wifi_status = WL_DISCONNECTED;

    for (;;) {
        // --- 1. CẬP NHẬT TRẠNG THÁI WIFI ---
        wl_status_t current_status = WiFi.status();
        if (current_status != last_wifi_status) {
            last_wifi_status = current_status;
            if (wifi_mutex && xSemaphoreTake(wifi_mutex, pdMS_TO_TICKS(100)) == pdTRUE) {
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
                xSemaphoreGive(wifi_mutex);
            }
        }

        // --- 2. XỬ LÝ QUÉT WIFI BẤT ĐỒNG BỘ ---
        if (is_scanning_wifi) {
            int16_t scan_result = WiFi.scanComplete();
            if (scan_result >= 0) {
                is_scanning_wifi = false;
                if (wifi_mutex && xSemaphoreTake(wifi_mutex, pdMS_TO_TICKS(100)) == pdTRUE) {
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
                    xSemaphoreGive(wifi_mutex);
                }
                WiFi.scanDelete();
            } else if (scan_result == WIFI_SCAN_FAILED) {
                is_scanning_wifi = false;
                if (wifi_mutex && xSemaphoreTake(wifi_mutex, pdMS_TO_TICKS(100)) == pdTRUE) {
                    wifi_count = 1;
                    strncpy(wifi_ssid[0], "Scan Failed", 31);
                    wifi_ssid_ptrs[0] = wifi_ssid[0];
                    ui.setFullListCount(wifi_count);
                    xSemaphoreGive(wifi_mutex);
                }
            }
        }

        // --- 3. XỬ LÝ KẾT NỐI WIFI TỰ ĐỘNG ---
        if (current_status != WL_CONNECTED && !is_connecting_wifi && !webConfig.isConfiguring() && !is_scanning_wifi && !wifi_manual_disconnected) {
            static uint32_t last_multi_run = 0;
            if (millis() - last_multi_run > 5000) {
                last_multi_run = millis();
                wifiMulti.run();
            }
        }

        // --- 3.1. XỬ LÝ KẾT NỐI WIFI THỦ CÔNG ---
        if (is_connecting_wifi) {
            if (WiFi.status() == WL_CONNECTED) {
                is_connecting_wifi = false;
                Serial.printf("\n[WiFi] Connected successfully to %s\n", connecting_ssid.c_str());
                ui.openModal("Connected!", connecting_ssid.c_str());
            } else if (millis() - wifi_connect_start > 10000) {
                is_connecting_wifi = false;
                WiFi.disconnect();
                Serial.println("\n[WiFi] Connection timeout or failed");
                if (!webConfig.isConfiguring()) {
                    webConfig.begin();
                }
                snprintf(text_input_title_buf, sizeof(text_input_title_buf), "FAIL: %s", connecting_ssid.c_str());
                ui.openTextInput(text_input_title_buf, on_wifi_password_submit, connecting_pwd.c_str());
            }
        }

        // --- 4. DUY TRÌ KẾT NỐI OTA ---
        ota.loop();
        webConfig.loop();
        
        // --- 5. ĐỒNG BỘ THỜI GIAN VÀ THỜI TIẾT QUA API ---
        if (timeSync.api_synced) {
            if (millis() - timeSync.last_time_sync > 3600000) {
                timeSync.update();
                weatherApi.update();
            }
        } else {
            static uint32_t last_sync_try = 0;
            if (millis() - last_sync_try > 2000) {
                last_sync_try = millis();
                if (WiFi.status() == WL_CONNECTED) {
                    timeSync.update();
                    weatherApi.update();
                }
            }
        }

        vTaskDelay(pdMS_TO_TICKS(50));
    }
}

