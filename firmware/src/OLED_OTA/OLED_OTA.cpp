#include "OLED_OTA.h"

OLED_OTA::OLED_OTA(const char* projectId, const char* projectToken, const char* currentVersion) {
    _projectId = String(projectId);
    _projectToken = String(projectToken);
    _currentVersion = String(currentVersion);
    _lastCheckAttempt = 0;
}

void OLED_OTA::setApiEndpoint(const char* host, uint16_t port) {
    _apiHost = String(host);
    _apiPort = port;
}

void OLED_OTA::begin() {
    uint8_t mac[6];
    WiFi.macAddress(mac);
    char macStr[18];
    sprintf(macStr, "%02X%02X%02X%02X%02X%02X", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    _deviceId = String(macStr);

    Serial.println("\n[OLED_OTA] Khởi tạo OTA Service (HTTP Polling)");
    Serial.println("[OLED_OTA] Device ID: " + _deviceId);
    Serial.println("[OLED_OTA] Phiên bản hiện tại: " + _currentVersion);
    
    _lastCheckAttempt = millis() - 60000; 

    const esp_partition_t* running = esp_ota_get_running_partition();
    const esp_partition_t* next_update = esp_ota_get_next_update_partition(NULL);
    
    Serial.printf("[OLED_OTA] Đang chạy: %s (0x%08X, %u KB)\n", 
                  running ? running->label : "NULL", 
                  running ? running->address : 0,
                  running ? running->size / 1024 : 0);
    Serial.printf("[OLED_OTA] OTA Target: %s (0x%08X, %u KB)\n", 
                  next_update ? next_update->label : "NULL", 
                  next_update ? next_update->address : 0,
                  next_update ? next_update->size / 1024 : 0);
}

void OLED_OTA::loop() {
    if (WiFi.status() == WL_CONNECTED) {
        unsigned long now = millis();
        if (now - _lastCheckAttempt > 60000) {
            _lastCheckAttempt = now;
            _checkUpdate();
        }
    }
}

void OLED_OTA::_checkUpdate() {
    String protocol = (_apiPort == 443) ? "https://" : "http://";
    String url = protocol + _apiHost;
    if (_apiPort != 80 && _apiPort != 443) {
        url += ":" + String(_apiPort);
    }
    url += "/api/firmware/" + _projectId;
    
    Serial.println("[OLED_OTA] Đang kiểm tra cập nhật tại: " + url);
    _performUpdate(url, "unknown");
}

/****
 * OTA bằng esp_partition_write() trực tiếp - bypass hoàn toàn esp_ota_write()
 * Lý do: esp_ota_write() trên ESP32-S3 + Arduino Core 2.0.11 bị bug
 *        trả về ESP_OK nhưng KHÔNG ghi gì xuống Flash (toàn 0xFF)
 * 
 * Quy trình mới:
 *   1. esp_partition_erase_range() - Xóa sạch partition đích
void OLED_OTA::_performUpdate(String url, String newVersion) {
    if (url.indexOf("?") == -1) {
        url += "?token=" + _projectToken;
    } else {
        url += "&token=" + _projectToken;
    }

    Serial.println("[OLED_OTA] Đang tiến hành cài đặt firmware mới...");
    
    bool isHttps = url.startsWith("https");
    WiFiClient* client = nullptr;
    
    if (isHttps) {
        WiFiClientSecure* secureClient = new WiFiClientSecure();
        secureClient->setInsecure();
        client = secureClient;
    } else {
        client = new WiFiClient();
    }
    
    client->setTimeout(30);

    HTTPClient http;
    http.begin(*client, url);
    http.addHeader("x-ESP32-version", _currentVersion);
    http.setTimeout(30000);
    
    int httpCode = http.GET();
    
    if (httpCode == HTTP_CODE_NOT_MODIFIED) {
        Serial.println("[OLED_OTA] Không có bản cập nhật mới (HTTP 304).");
        http.end();
        delete client;
        return;
    }
    
    if (httpCode != HTTP_CODE_OK) {
        Serial.printf("[OLED_OTA] Lỗi kết nối HTTP: %d\n", httpCode);
        http.end();
        delete client;
        return;
    }

    int contentLength = http.getSize();
    Serial.printf("[OLED_OTA] Kích thước firmware: %d bytes\n", contentLength);
    
    if (contentLength <= 0) {
        Serial.println("[OLED_OTA] Lỗi: Server không gửi Content-Length.");
        http.end();
        delete client;
        return;
    }

    Serial.println("[OLED_OTA] Bắt đầu quá trình Update OTA...");
    if (Update.begin(contentLength)) {
        WiFiClient *tcp = http.getStreamPtr();
        size_t written = Update.writeStream(*tcp);
        
        if (written == (size_t)contentLength) {
            Serial.println("[OLED_OTA] Ghi firmware thành công, đang kiểm tra...");
            if (Update.end()) {
                Serial.println("[OLED_OTA] OTA Hoàn tất! Đang khởi động lại...");
                delay(1000);
                ESP.restart();
            } else {
                Serial.printf("[OLED_OTA] Lỗi Update.end(): %s\n", Update.errorString());
            }
        } else {
            Serial.printf("[OLED_OTA] Ghi thiếu dữ liệu: %d/%d bytes\n", written, contentLength);
        }
    } else {
        Serial.printf("[OLED_OTA] Không đủ bộ nhớ cho OTA: %s\n", Update.errorString());
    }
    
    http.end();
    
    if (writeError || written != (size_t)contentLength) {
        Serial.printf("[OLED_OTA] Tải/ghi thất bại. Đã ghi: %u/%d\n", written, contentLength);
        delete client;
        return;
    }

    /****
     * BƯỚC 4: Verify 16 byte cuối cùng trên Flash
     ****/
    Serial.println("[OLED_OTA] Đã ghi đủ. Verify 16 byte đầu trên Flash...");
    err = esp_partition_read(update_partition, 0, verify_buf, 16);
    Serial.printf("[OLED_OTA] Flash byte đầu: ");
    for (int i = 0; i < 16; i++) {
        Serial.printf("%02X ", verify_buf[i]);
    }
    Serial.println();

    /****
     * BƯỚC 5: Đổi cờ khởi động sang partition mới
     * Không dùng esp_ota_end() vì chúng ta không dùng esp_ota_begin()
     ****/
    Serial.println("[OLED_OTA] Đang đổi Boot partition...");
    err = esp_ota_set_boot_partition(update_partition);
    if (err != ESP_OK) {
        Serial.printf("[OLED_OTA] LỖI esp_ota_set_boot_partition: %s (0x%x)\n", esp_err_to_name(err), err);
        delete client;
        return;
    }
    
    Serial.println("[OLED_OTA] OTA Hoàn tất! Đang khởi động lại...");
    delete client;
    delay(1000);
    ESP.restart();
}
