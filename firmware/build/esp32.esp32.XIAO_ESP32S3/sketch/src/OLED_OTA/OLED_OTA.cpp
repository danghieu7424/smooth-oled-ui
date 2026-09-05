#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\src\\OLED_OTA\\OLED_OTA.cpp"
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
    
    // Check update immediately on boot
    _lastCheckAttempt = millis() - 60000; 

    // FIX: Nếu otadata bị xóa (empty), ESP-IDF OTA API sẽ nhầm tưởng 
    // phân vùng kế tiếp là app0 và ghi đè lên chính nó gây lỗi Flash Read Failed.
    // Do đó ta cần khởi tạo otadata trỏ về phân vùng đang chạy.
    const esp_partition_t* running = esp_ota_get_running_partition();
    const esp_partition_t* configured = esp_ota_get_boot_partition();
    if (configured == NULL || configured->address != running->address) {
        Serial.printf("[OLED_OTA] Khởi tạo otadata. Đang chạy: 0x%x\n", running->address);
        esp_err_t err = esp_ota_set_boot_partition(running);
        if (err != ESP_OK) {
            Serial.printf("[OLED_OTA] LỖI: Không thể set boot partition (%s)\n", esp_err_to_name(err));
        } else {
            Serial.println("[OLED_OTA] Đã set boot partition thành công!");
        }
    }
}

void OLED_OTA::loop() {
    if (WiFi.status() == WL_CONNECTED) {
        unsigned long now = millis();
        // Check every 60 seconds
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
    // Append Token to the URL if not already present
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
        secureClient->setInsecure(); // Bỏ qua xác thực chứng chỉ SSL
        client = secureClient;
    } else {
        client = new WiFiClient();
    }
    
    // Tăng timeout để tránh lỗi Stream Read Failed (lỗi 3)
    client->setTimeout(10000);

    HTTPClient http;
    http.begin(*client, url);
    http.addHeader("x-ESP32-version", _currentVersion);
    
    int httpCode = http.GET();
    if (httpCode == HTTP_CODE_NOT_MODIFIED) {
        Serial.println("[OLED_OTA] Không có bản cập nhật mới (HTTP 304).");
    } else if (httpCode == HTTP_CODE_OK) {
        int contentLength = http.getSize();
        bool canBegin = contentLength > 0 ? Update.begin(contentLength) : Update.begin(UPDATE_SIZE_UNKNOWN);
        
        if (canBegin) {
            Serial.println("[OLED_OTA] OTA Bắt đầu...");
            WiFiClient *tcp = http.getStreamPtr();
            
            size_t written = Update.writeStream(*tcp);
            
            if (written == contentLength) {
                Serial.println("\n[OLED_OTA] Đã tải đủ toàn bộ file.");
                
                // Cố gắng kết thúc OTA
                if (Update.end(true)) {
                    Serial.println("\n[OLED_OTA] OTA Hoàn tất. Đang khởi động lại...");
                    delay(1000);
                    ESP.restart();
                } else {
                    if (Update.getError() == UPDATE_ERROR_READ) {
                        Serial.println("\n[OLED_OTA] Bỏ qua lỗi Flash Read Failed (Bug ESP32 Core).");
                        const esp_partition_t* next = esp_ota_get_next_update_partition(NULL);
                        if (next) {
                            esp_err_t err = esp_ota_set_boot_partition(next);
                            if (err == ESP_OK) {
                                Serial.println("[OLED_OTA] Đã ép buộc đổi phân vùng Boot thành công. Khởi động lại...");
                                delay(1000);
                                ESP.restart();
                            } else {
                                Serial.printf("[OLED_OTA] Ép buộc đổi phân vùng thất bại: %d\n", err);
                            }
                        }
                    } else {
                        Serial.printf("\n[OLED_OTA] Lỗi OTA (%d): %s\n", Update.getError(), Update.errorString());
                    }
                }
            } else {
                Serial.printf("\n[OLED_OTA] Lỗi: Tải bị gián đoạn. Đã tải: %u/%u\n", written, contentLength);
                Update.end(); // Abort
            }
        } else {
            Serial.printf("[OLED_OTA] Không thể bắt đầu OTA (%d): %s\n", Update.getError(), Update.errorString());
        }
    } else {
        Serial.printf("[OLED_OTA] Lỗi kết nối HTTP: %d\n", httpCode);
    }
    
    http.end();
    delete client;
}
