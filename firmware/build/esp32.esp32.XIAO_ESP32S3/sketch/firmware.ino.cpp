#include <Arduino.h>
#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
﻿#include <Arduino.h>
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
// Cáº¤U HÃŒNH Dá»° ÃN Tá»ª OTA HUB DASHBOARD
// ==========================================
const char* PROJECT_ID = "007Rlq30Q2vU-esp32-tool";
const char* PROJECT_TOKEN = "fc11b225f325609bb7309ad70f090a78";
const char* CURRENT_VERSION = "1.0.0";
const char* API_HOST = "192.168.7.7";
const uint16_t API_PORT = 7424;

OLED_OTA ota(PROJECT_ID, PROJECT_TOKEN, CURRENT_VERSION);

int saved_brightness = 20;

#define LED_PIN 2
int saved_led_state = 0;

enum ActiveSlider { SLIDER_NONE, SLIDER_BRIGHTNESS, SLIDER_LED_SWITCH };
ActiveSlider active_slider = SLIDER_NONE;

// Clock State variables moved down

// Clock State variables moved down

#line 39 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void save_wifi_credentials(String ssid, String pwd);
#line 45 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
bool load_wifi_credentials(String &ssid, String &pwd);
#line 164 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void on_restart();
#line 168 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void on_power_off();
#line 336 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void setup();
#line 443 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
void loop();
#line 39 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\firmware.ino"
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

// Khá»Ÿi táº¡o mÃ n hÃ¬nh
U8G2_SSD1306_128X64_NONAME_F_HW_I2C u8g2(U8G2_R0, /* reset=*/ U8X8_PIN_NONE);

// Truyá»n tham chiáº¿u mÃ n hÃ¬nh vÃ  UART (Ä‘á»ƒ Stream) vÃ o lÃµi thÆ° viá»‡n
SmoothOLED ui(&u8g2, &Serial);

// =======================================================================
// [DATA] Danh sÃ¡ch Icon (XBM 24x24)
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

// [Má»šI] Icon WiFi - Váº½ trÃªn khung 24x24 px
static const unsigned char icon_wifi[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
  0x80, 0xff, 0x01, 0xe0, 0xff, 0x07, 0xf0, 0x00, 0x0f, 0x38, 0x00, 0x1c,  
  0x1c, 0xff, 0x38, 0xc0, 0xff, 0x03, 0xe0, 0x81, 0x07, 0x70, 0x00, 0x0e,  
  0x00, 0x7e, 0x00, 0x80, 0xff, 0x01, 0x80, 0xc3, 0x01, 0x00, 0x00, 0x00,  
  0x00, 0x18, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x18, 0x00,  
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

// [Má»šI] Icon esp now - Váº½ trÃªn khung 24x24 px
static const unsigned char icon_esp_now[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x02, 0x60, 0x00, 0x06,  
  0x30, 0x00, 0x0c, 0x10, 0x42, 0x08, 0x18, 0x81, 0x18, 0x18, 0x99, 0x18,  
  0x18, 0x99, 0x18, 0x18, 0x81, 0x18, 0x10, 0x42, 0x08, 0x30, 0x18, 0x0c,  
  0x60, 0x18, 0x06, 0x40, 0x18, 0x02, 0x00, 0x18, 0x00, 0x00, 0x18, 0x00,  
  0x00, 0x18, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x3c, 0x00,  
  0x00, 0x7e, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

// [Má»šI] Icon LED - Váº½ trÃªn khung 24x24 px
static const unsigned char icon_led_switch[] U8X8_PROGMEM = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
  0xf8, 0xff, 0x1f, 0xfc, 0xff, 0x3f, 0x0c, 0x00, 0x30, 0x0c, 0x00, 0x30, 
  0x0c, 0x00, 0x30, 0x8c, 0x01, 0x30, 0x8c, 0x03, 0x30, 0x0c, 0x07, 0x30, 
  0x0c, 0x07, 0x30, 0x8c, 0x03, 0x30, 0x8c, 0xf1, 0x33, 0x0c, 0xf0, 0x33, 
  0x0c, 0x00, 0x30, 0x0c, 0x00, 0x30, 0xfc, 0xff, 0x3f, 0xf8, 0xff, 0x1f, 
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

// --- KHAI BÃO CÃC HÃ€M Xá»¬ LÃ Sá»° KIá»†N (CALLBACKS) ---
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
    // {"LED Switch", icon_led_switch, open_led_switch},
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
    ESP.restart(); // Sá»­a láº¡i lá»‡nh chuáº©n cá»§a ESP32
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
    snprintf(about_buf[8], sizeof(about_buf[8]), "Ver: %s", CURRENT_VERSION);

    for (int i = 0; i < 9; i++) {
        about_items[i] = about_buf[i];
    }
    
    ui.openFullList("About MCU", about_items, 9, nullptr);
}

// =======================================================================
// [SETUP & LOOP]
// =======================================================================

// Quáº£n lÃ½ tráº¡ng thÃ¡i Menu
enum MenuLevel { LEVEL_MAIN, LEVEL_SETTINGS, LEVEL_WIFI };
MenuLevel current_level = LEVEL_MAIN;
int current_brightness = 20; // Äá»™ sÃ¡ng hiá»‡n táº¡i


// Quáº£n lÃ½ WiFi
#define MAX_WIFI_NETWORKS 15
char wifi_ssid[MAX_WIFI_NETWORKS][32];
char wifi_raw_ssid[MAX_WIFI_NETWORKS][32];
const char* wifi_ssid_ptrs[MAX_WIFI_NETWORKS];
int wifi_count = 0;
bool is_scanning_wifi = false;

// Tráº¡ng thÃ¡i káº¿t ná»‘i WiFi
bool is_connecting_wifi = false;
uint32_t wifi_connect_start = 0;
String connecting_ssid = "";
String connecting_pwd = "";

// --- CÃ€I Äáº¶T CÃC HÃ€M Xá»¬ LÃ Sá»° KIá»†N ---

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
  u8g2.setContrast(current_brightness); // Lá»‡nh pháº§n cá»©ng Ä‘á»•i Ä‘á»™ sÃ¡ng OLED trá»±c tiáº¿p
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
  // Cháº·n má»Ÿ máº­t kháº©u náº¿u Ä‘ang Scanning, khÃ´ng cÃ³ máº¡ng, hoáº·c lá»—i
  if (idx >= 0 && idx < wifi_count && strncmp(wifi_ssid[0], "Scanning", 8) != 0 && strncmp(wifi_ssid[0], "No networks", 10) != 0 && strncmp(wifi_ssid[0], "Scan Failed", 11) != 0) {
      // Náº¿u máº¡ng nÃ y Ä‘ang Ä‘Æ°á»£c káº¿t ná»‘i rá»“i, bÃ¡o luÃ´n khÃ´ng cáº§n nháº­p pass
      if (WiFi.status() == WL_CONNECTED && WiFi.SSID() == String(wifi_raw_ssid[idx])) {
          ui.openModal("Connected!", "Already connected to this network");
          return;
      }

      // KIá»‚M TRA: Náº¿u máº¡ng nÃ y TRÃ™NG vá»›i máº¡ng Ä‘Ã£ lÆ°u trong AT24C256
      String saved_pwd = "";
      String saved_ssid = "";
      if (load_wifi_credentials(saved_ssid, saved_pwd)) {
          if (String(wifi_raw_ssid[idx]) != saved_ssid) {
              saved_pwd = ""; // Náº¿u khÃ´ng khá»›p SSID thÃ¬ khÃ´ng dÃ¹ng pwd nÃ y
          }
      }
      
      // Má»Ÿ Ã´ nháº­p Pass vÃ  ÄIá»€N Sáº´N máº­t kháº©u cÅ© (nhÆ° tháº» input type="text" cÃ³ value)
      // NgÆ°á»i dÃ¹ng chá»‰ cáº§n áº¥n Enter Ä‘á»ƒ káº¿t ná»‘i, hoáº·c áº¥n xÃ³a Ä‘á»ƒ sá»­a
      snprintf(text_input_title_buf, sizeof(text_input_title_buf), "PWD: %s", wifi_raw_ssid[idx]);
      ui.openTextInput(text_input_title_buf, on_wifi_password_submit, saved_pwd.c_str());
  }
}

void on_enter_wifi() {
  current_level = LEVEL_WIFI;
  
  // Khá»Ÿi táº¡o UI hiá»ƒn thá»‹ táº¡m thá»i "Scanning..."
  wifi_count = 1;
  strncpy(wifi_ssid[0], "Scanning...", 31);
  wifi_ssid_ptrs[0] = wifi_ssid[0];
  ui.openFullList("WIFI NETWORKS", wifi_ssid_ptrs, wifi_count, on_wifi_selected);

  if (is_scanning_wifi) return; // Äang quÃ©t thÃ¬ khÃ´ng kÃ­ch hoáº¡t láº¡i
  
  WiFi.mode(WIFI_STA);
  // XÃ“A WiFi.disconnect() á»Ÿ Ä‘Ã¢y Ä‘á»ƒ khÃ´ng lÃ m rá»›t máº¡ng Ä‘ang káº¿t ná»‘i khi load láº¡i menu
  
  WiFi.scanNetworks(true); // QuÃ©t báº¥t Ä‘á»“ng bá»™ (Async)
  is_scanning_wifi = true;
}

void on_wifi_password_submit(const char* pwd) {
  int idx = ui.getFullListSelectedIndex();
  if (idx >= 0 && idx < wifi_count) {
      connecting_ssid = wifi_raw_ssid[idx];
      connecting_pwd = pwd;
      
      // [Má»šI] LÆ°u Credentials vÃ o AT24C256
      save_wifi_credentials(connecting_ssid, connecting_pwd);
      
      Serial.printf("\n[WiFi] Connecting to %s with password: %s\n", connecting_ssid.c_str(), pwd);
      
      // Ngáº¯t káº¿t ná»‘i cÅ© (náº¿u cÃ³)
      WiFi.disconnect();
      delay(100);
      WiFi.begin(connecting_ssid.c_str(), pwd);
      
      is_connecting_wifi = true;
      wifi_connect_start = millis();
      
      // Hiá»ƒn thá»‹ tráº¡ng thÃ¡i Connecting... lÃªn mÃ n hÃ¬nh
      ui.openModal("Connecting...", connecting_ssid.c_str());
  }
}

SemaphoreHandle_t wifi_mutex = NULL;

void task_ui_core0(void *pvParameters);
void task_network_core1(void *pvParameters);

void setup() {
  Serial.begin(921600);
  
  pinMode(LED_PIN, OUTPUT);

  // --- Khá»Ÿi táº¡o I2C Bus 0 cho OLED (Core 0) ---
  Wire.begin(4, 5);
  Wire.setClock(400000); 

  // --- Khá»Ÿi táº¡o vÃ  kiá»ƒm tra RTC & EEPROM (Core 1 sáº½ dÃ¹ng I2C1 / Wire1) ---
  // Khá»Ÿi táº¡o Bus 1 (Sensor) á»Ÿ Ä‘Ã¢y Ä‘á»ƒ cÃ¡c module gá»i begin() thÃ nh cÃ´ng
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

  // --- Káº¾T Ná»I WIFI Máº¶C Äá»ŠNH ---
  WiFi.mode(WIFI_STA);
  WiFi.disconnect(true);
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
  WiFi.setAutoReconnect(true);

  // 1. GÃ¡n máº£ng dá»¯ liá»‡u vÃ o thÆ° viá»‡n UI
  ui.setCarouselItems(menu_items, TOTAL_MAIN_ITEMS, "< MAIN MENU >");
  ui.setPopupListItems(popup_items, TOTAL_POPUP_ITEMS);
  ui.setSidePopupItems(side_items, TOTAL_SIDE_ITEMS);

  // 2. Cáº¥u hÃ¬nh UI
  ui.enableAutoDemo(false);
  ui.enablePCViewer(false);

  // 3. Khá»Ÿi Ä‘á»™ng UI
  ui.begin();

  // 4. Cáº¥u hÃ¬nh TimeSync
  timeSync.begin(7);
  if (rtc.begin()) {
      timeSync.syncFromRTC();
  }
  
  pinMode(LED_PIN, OUTPUT);
  digitalWrite(LED_PIN, saved_led_state == 1 ? HIGH : LOW);
  
  open_home_clock();

  // 5. Khá»Ÿi táº¡o OTA Service
  ota.setApiEndpoint(API_HOST, API_PORT);
  ota.begin();

  // Táº¡o Mutex cho cÃ¡c biáº¿n dÃ¹ng chung
  wifi_mutex = xSemaphoreCreateMutex();

  // Táº¡o Task UI trÃªn Core 0
  xTaskCreatePinnedToCore(
      task_ui_core0,
      "Task_UI",
      8192,
      NULL,
      1,
      NULL,
      0
  );

  // Táº¡o Task Network trÃªn Core 1
  xTaskCreatePinnedToCore(
      task_network_core1,
      "Task_Network",
      8192,
      NULL,
      1,
      NULL,
      1
  );
}

void loop() {
    // XÃ³a task loop cá»§a Arduino Ä‘á»ƒ giáº£i phÃ³ng tÃ i nguyÃªn
    vTaskDelete(NULL);
}

// ==========================================
// TASK: UI & ANIMATION (CORE 0)
// ==========================================
void task_ui_core0(void *pvParameters) {
    for (;;) {
        // 1. Xá»­ lÃ½ Input tá»« Serial (NÃºt báº¥m mÃ´ phá»ng)
        if (Serial.available() > 0) {
            char c = Serial.read();
            if (c == '\x1B') {
                uint32_t t = millis();
                while (!Serial.available() && millis() - t < 50) { vTaskDelay(1); }
                if (Serial.available()) {
                    char cmd = Serial.read();
                    if (cmd == 'U') ui.up();
                    else if (cmd == 'D') ui.down();
                    else if (cmd == 'L') ui.left();
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
                        }
                        ui.closeOverlay();
                    } else if (ui.getAppState() == STATE_SLIDER) {
                        if (active_slider == SLIDER_BRIGHTNESS) {
                            current_brightness = saved_brightness;
                            u8g2.setContrast(current_brightness);
                        } else if (active_slider == SLIDER_LED_SWITCH) {
                            open_led_switch();
                        }
                        active_slider = SLIDER_NONE;
                        ui.closeOverlay();
                    } else if (ui.getAppState() == STATE_CAROUSEL && current_level == LEVEL_SETTINGS) {
                        current_level = LEVEL_MAIN;
                        ui.setCarouselItems(menu_items, TOTAL_MAIN_ITEMS, "< MAIN MENU >");
                    }
                }
            }
            else if (c == 'B') {
                if (ui.getAppState() == STATE_TEXT_INPUT) {
                    ui.backspace();
                } else if (current_level == LEVEL_WIFI) {
                    if (WiFi.status() == WL_CONNECTED) {
                        WiFi.disconnect();
                        extEEPROM.writeByte(0x000F, 0x00);
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
                        ui.openModal("Disconnected", "Wi-Fi is now disconnected");
                    }
                }
            }
            else if (c == 'E') {
                ui.select();
                if (ui.getAppState() == STATE_SLIDER) {
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

        // 2. Logic Äá»“ng há»“ (Tick)
        if (timeSync.tick()) {
            ui.updateClock(timeSync.current_hour, timeSync.current_minute, timeSync.current_second, timeSync.solar_date_str.c_str(), timeSync.lunar_date_str.c_str(), timeSync.current_temp_str.c_str());
        }

        // 3. Váº½ lÃªn mÃ n hÃ¬nh OLED (Qua I2C0)
        ui.update();

        // 4. Delay Ä‘á»ƒ giá»¯ 60FPS
        vTaskDelay(pdMS_TO_TICKS(16));
    }
}

// ==========================================
// TASK: NETWORK & BACKGROUND (CORE 1)
// ==========================================
void task_network_core1(void *pvParameters) {
    static wl_status_t last_wifi_status = WL_DISCONNECTED;

    for (;;) {
        // --- 1. THEO DÃ•I TRáº NG THÃI WIFI ---
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

        // --- 2. Xá»¬ LÃ QUÃ‰T WIFI Báº¤T Äá»’NG Bá»˜ ---
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

        // --- 3. Xá»¬ LÃ Káº¾T Ná»I WIFI (NON-BLOCKING) ---
        if (is_connecting_wifi) {
            if (WiFi.status() == WL_CONNECTED) {
                is_connecting_wifi = false;
                save_wifi_credentials(connecting_ssid, connecting_pwd);
                Serial.printf("\n[WiFi] Connected successfully to %s\n", connecting_ssid.c_str());
                ui.openModal("Connected!", connecting_ssid.c_str());
            } else if (millis() - wifi_connect_start > 10000) {
                is_connecting_wifi = false;
                WiFi.disconnect();
                Serial.println("\n[WiFi] Connection timeout or failed");
                snprintf(text_input_title_buf, sizeof(text_input_title_buf), "FAIL: %s", connecting_ssid.c_str());
                ui.openTextInput(text_input_title_buf, on_wifi_password_submit, connecting_pwd.c_str());
            }
        }

        // --- 4. DUY TRÃŒ Káº¾T Ná»I OTA ---
        ota.loop();
        
        // --- 5. Äá»’NG Bá»˜ THá»œI GIAN QUA API ---
        if (timeSync.api_synced) {
            if (millis() - timeSync.last_time_sync > 3600000) {
                timeSync.update();
            }
        } else {
            static uint32_t last_sync_try = 0;
            if (millis() - last_sync_try > 2000) {
                last_sync_try = millis();
                if (WiFi.status() == WL_CONNECTED) {
                    timeSync.update();
                }
            }
        }

        vTaskDelay(pdMS_TO_TICKS(50));
    }
}

