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

    
    httpUpdate.onStart([]() { Serial.println("[OLED_OTA] OTA Bắt đầu..."); });
    httpUpdate.onEnd([]() { Serial.println("\n[OLED_OTA] OTA Hoàn tất. Đang khởi động lại..."); });
    httpUpdate.onProgress([](int cur, int total) {
        Serial.printf("[OLED_OTA] Tiến trình: %d%%\r", (cur * 100) / total);
    });
    httpUpdate.onError([](int err) {
        Serial.printf("[OLED_OTA] Lỗi cập nhật[%d]: %s\n", err, httpUpdate.getLastErrorString().c_str());
    });
    
    httpUpdate.rebootOnUpdate(true); // Automatically reboot after successful update
    
    t_httpUpdate_return ret = httpUpdate.update(*client, url, _currentVersion);
    
    switch (ret) {
        case HTTP_UPDATE_FAILED:
            Serial.printf("[OLED_OTA] Lỗi OTA (%d): %s\n", httpUpdate.getLastError(), httpUpdate.getLastErrorString().c_str());
            break;
        case HTTP_UPDATE_NO_UPDATES:
            Serial.println("[OLED_OTA] Không có bản cập nhật mới (HTTP 304).");
            break;
        case HTTP_UPDATE_OK:
            Serial.println("[OLED_OTA] Cập nhật thành công!");
            break;
    }
    
    delete client;
}
