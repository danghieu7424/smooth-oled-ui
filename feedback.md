[HW-ELEC]

Hiện tượng restart bị trả về giá trị mặc định (`brightness = 20`) xuất phát từ 2 nguyên nhân cốt lõi trong hàm `setup()` và cách kiểm tra dữ liệu:

1. **Thứ tự gọi hàm trong `setup()`:** Lệnh `nvs_flash_erase()` được gọi nếu NVS gặp lỗi, nhưng quan trọng nhất là bạn gọi `load_system_state()` **ngay trước khi** hoặc khi phân vùng EEPROM trên Flash chưa được phân biệt với dữ liệu rác (uninitialized memory).


2. **Logic kiểm tra dữ liệu khởi tạo (`state.ssid[0] == (char)255`) không chuẩn:** Khi chip Flash của ESP32 vừa nạp hoặc vùng nhớ NVS bị phân mảnh, giá trị của `state.brightness` đọc lên có thể là `0xFFFFFFFF` (tức `-1`), khiến điều kiện `if (state.brightness < 0 || state.brightness > 255)` luôn kích hoạt và ghi đè giá trị về `20`.


3. **Thư viện EEPROM của ESP32 đã deprecated (lạc hậu):** `EEPROM.h` trên ESP32 chỉ là một lớp bọc (wrapper) mô phỏng đè lên NVS và rất dễ xung đột khi gọi `nvs_flash_init()` / `nvs_flash_erase()` thủ công. Cách chuẩn xác và ổn định nhất trên ESP32 là sử dụng trực tiếp thư viện **`Preferences.h`**.



Dưới đây là phần code thay thế cụ thể:

---

### Code Thay Thế: Chuyển sang dùng `Preferences.h` (Chuẩn Non-Volatile Storage ESP32)

Thay thế hoàn toàn phần cấu hình `EEPROM` trong file chính bằng đoạn mã dưới đây:

```cpp
#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>
#include <WiFi.h>
#include <Preferences.h> // Thay thế EEPROM.h và nvs_flash.h
#include "SmoothOLED.h"

Preferences prefs;

int saved_brightness = 20;

// Cấu trúc lưu trạng thái hệ thống
struct SystemState {
    bool on;
    char ssid[32];
    char pwd[64];
    int brightness;
};

// Hàm lưu trạng thái an toàn vào NVS
void save_system_state(bool on, int brightness, String ssid, String pwd) {
    prefs.begin("sys_cfg", false); // Mở namespace "sys_cfg" ở chế độ Read/Write
    prefs.putBool("wifi_on", on);
    prefs.putInt("bright", brightness);
    prefs.putString("ssid", ssid);
    prefs.putString("pwd", pwd);
    prefs.end();
}

// Hàm đọc trạng thái từ NVS
SystemState load_system_state() {
    SystemState state;
    prefs.begin("sys_cfg", true); // Mở ở chế độ Read-Only
    
    state.on = prefs.getBool("wifi_on", false);
    state.brightness = prefs.getInt("bright", 20); // Nếu chưa có, mặc định là 20
    
    String s_ssid = prefs.getString("ssid", "");
    String s_pwd = prefs.getString("pwd", "");
    
    strncpy(state.ssid, s_ssid.c_str(), 31);
    state.ssid[31] = '\0';
    strncpy(state.pwd, s_pwd.c_str(), 63);
    state.pwd[63] = '\0';
    
    prefs.end();
    
    // Ràng buộc giới hạn an toàn cho độ sáng
    if (state.brightness < 0 || state.brightness > 255) {
        state.brightness = 20;
    }
    return state;
}

```

---

### Cập nhật lại `setup()`

Xóa bỏ các lệnh `nvs_flash_*` và `EEPROM.begin(512)`, thay thế bằng:

```cpp
void setup() {
  Serial.begin(921600);
  
  // 1. Đọc cấu hình đã lưu từ NVS Preferences
  SystemState state = load_system_state();
  saved_brightness = state.brightness;
  current_brightness = saved_brightness;

  // 2. Khởi tạo I2C và Màn hình OLED
  Wire.begin(47, 48);
  Wire.setClock(400000); 
  u8g2.begin();
  u8g2.setContrast(current_brightness); // Áp dụng độ sáng vừa đọc từ NVS

  // 3. Khôi phục trạng thái WiFi
  if (state.on && strlen(state.ssid) > 0) {
      WiFi.mode(WIFI_STA);
      WiFi.disconnect(true);
      delay(100);
      WiFi.begin(state.ssid, state.pwd);
      WiFi.setAutoReconnect(true);
  } else {
      WiFi.mode(WIFI_OFF);
  }

  // 4. Gán cấu hình UI và khởi động
  ui.setCarouselItems(menu_items, TOTAL_MAIN_ITEMS, "< MAIN MENU >");
  ui.setPopupListItems(popup_items, TOTAL_POPUP_ITEMS);
  ui.setSidePopupItems(side_items, TOTAL_SIDE_ITEMS);
  ui.enableAutoDemo(false);
  ui.enablePCViewer(true);
  ui.begin();
}

```