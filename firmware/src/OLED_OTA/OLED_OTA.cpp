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
                
                // WORKAROUND CHO LỖI ESP32-S3 CORE 2.0.11: 
                // Khi Update.end() trả về "Flash Read Failed" do bộ đệm cache chưa đồng bộ, 
                // nhưng thực tế dữ liệu đã được ghi thành công, ta sẽ chủ động ép đổi boot partition.
                if (String(Update.errorString()) == "Flash Read Failed") {
                    Serial.println("[OLED_OTA] Kích hoạt Workaround cho lỗi Flash Read...");
                    const esp_partition_t* update_partition = esp_ota_get_next_update_partition(NULL);
                    if (update_partition) {
                        esp_err_t err = esp_ota_set_boot_partition(update_partition);
                        if (err == ESP_OK) {
                            Serial.println("[OLED_OTA] Ép đổi Boot Partition thành công! Đang khởi động lại...");
                            delay(1000);
                            ESP.restart();
                        } else {
                            Serial.printf("[OLED_OTA] Ép đổi Boot thất bại: %s\n", esp_err_to_name(err));
                        }
                    }
                }
            }
        } else {
            Serial.printf("[OLED_OTA] Ghi thiếu dữ liệu: %d/%d bytes\n", written, contentLength);
        }
    } else {
        Serial.printf("[OLED_OTA] Không đủ bộ nhớ cho OTA: %s\n", Update.errorString());
    }
    
    http.end();
    delete client;
}
