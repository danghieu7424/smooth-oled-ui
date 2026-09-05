#include <WiFi.h>
#include "src/OLED_OTA.h"

// ==========================================
// CẤU HÌNH WIFI
// ==========================================
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";

// ==========================================
// CẤU HÌNH DỰ ÁN TỪ OTA HUB DASHBOARD
// ==========================================
// Copy thông tin từ Dashboard của dự án:
const char* PROJECT_ID = "007Rlq30Q2vU-esp32-tool"; // ID đầy đủ lấy từ Dashboard
const char* PROJECT_TOKEN = "YOUR_SECRET_TOKEN"; // Token xác nhận dự án
const char* CURRENT_VERSION = "1.0.0"; // Phiên bản firmware hiện tại trên thiết bị

// ==========================================
// CẤU HÌNH MQTT BROKER (Dùng để nhận tín hiệu cập nhật tức thời)
// ==========================================
const char* MQTT_BROKER = "broker.hivemq.com";
const uint16_t MQTT_PORT = 1883;
// Nếu dùng broker có mật khẩu:
// const char* MQTT_USER = "user";
// const char* MQTT_PASS = "pass";

// Khởi tạo đối tượng OTA
OLED_OTA ota(PROJECT_ID, PROJECT_TOKEN, CURRENT_VERSION);

void setup() {
    Serial.begin(115200);
    delay(1000);
    Serial.println("\n\n--- KHỞI ĐỘNG THIẾT BỊ ---");
    Serial.printf("Phiên bản hiện tại: %s\n", CURRENT_VERSION);

    // 1. Kết nối WiFi
    Serial.print("Đang kết nối WiFi");
    WiFi.begin(ssid, password);
    while (WiFi.status() != WL_CONNECTED) {
        delay(500);
        Serial.print(".");
    }
    Serial.println("\nWiFi Đã kết nối!");
    Serial.print("Địa chỉ IP: ");
    Serial.println(WiFi.localIP());

    // 2. Cấu hình MQTT Broker
    ota.setMqttBroker(MQTT_BROKER, MQTT_PORT);
    
    // 3. Khởi động OTA Service
    // Thiết bị sẽ kết nối MQTT và tự động subscribe vào topic của nó
    ota.begin();
    
    Serial.println("--- SẴN SÀNG ---");
}

void loop() {
    // Gọi hàm loop của OTA để duy trì kết nối MQTT và lắng nghe bản cập nhật
    ota.loop();
    
    // ==========================================
    // VIẾT CODE CHÍNH CỦA DỰ ÁN Ở ĐÂY
    // ==========================================
    
    // Ví dụ: Nhấp nháy LED
    // digitalWrite(LED_BUILTIN, HIGH);
    // delay(1000);
    // digitalWrite(LED_BUILTIN, LOW);
    // delay(1000);
}
