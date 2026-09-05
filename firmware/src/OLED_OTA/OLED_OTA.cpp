#include "OLED_OTA.h"

OLED_OTA::OLED_OTA(const char* projectId, const char* projectToken, const char* currentVersion) {
    _projectId = String(projectId);
    _projectToken = String(projectToken);
    _currentVersion = String(currentVersion);
    _mqttClient = new PubSubClient(_wifiClient);
    _lastReconnectAttempt = 0;
}

void OLED_OTA::setMqttBroker(const char* broker, uint16_t port, const char* user, const char* pass) {
    _mqttBroker = String(broker);
    _mqttPort = port;
    _mqttUser = String(user);
    _mqttPass = String(pass);
    _mqttClient->setServer(_mqttBroker.c_str(), _mqttPort);
}

void OLED_OTA::begin() {
    // Generate a unique device ID using MAC address
    uint8_t mac[6];
    WiFi.macAddress(mac);
    char macStr[18];
    sprintf(macStr, "%02X%02X%02X%02X%02X%02X", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    _deviceId = String(macStr);

    // Topic format: projects/{project_id}/devices/{device_id}/ota
    _otaTopic = "projects/" + _projectId + "/devices/" + _deviceId + "/ota";
    
    Serial.println("\n[OLED_OTA] Khởi tạo OTA Service");
    Serial.println("[OLED_OTA] Device ID: " + _deviceId);
    Serial.println("[OLED_OTA] OTA Topic: " + _otaTopic);
    Serial.println("[OLED_OTA] Phiên bản hiện tại: " + _currentVersion);

    // Set callback for MQTT
    _mqttClient->setCallback([this](char* topic, byte* payload, unsigned int length) {
        this->_mqttCallback(topic, payload, length);
    });

    _connectMqtt();
}

void OLED_OTA::loop() {
    if (!_mqttClient->connected()) {
        unsigned long now = millis();
        if (now - _lastReconnectAttempt > 5000) {
            _lastReconnectAttempt = now;
            if (_connectMqtt()) {
                _lastReconnectAttempt = 0;
            }
        }
    } else {
        _mqttClient->loop();
    }
}

bool OLED_OTA::_connectMqtt() {
    Serial.print("[OLED_OTA] Đang kết nối MQTT Broker...");
    
    // Create a random client ID
    String clientId = "ESP32Client-" + String(random(0xffff), HEX);
    
    bool connected = false;
    if (_mqttUser.length() > 0) {
        connected = _mqttClient->connect(clientId.c_str(), _mqttUser.c_str(), _mqttPass.c_str());
    } else {
        connected = _mqttClient->connect(clientId.c_str());
    }

    if (connected) {
        Serial.println(" Thành công!");
        // Subscribe to OTA topic
        _mqttClient->subscribe(_otaTopic.c_str());
        Serial.println("[OLED_OTA] Đã subscribe vào topic: " + _otaTopic);
        return true;
    } else {
        Serial.print(" Thất bại, rc=");
        Serial.println(_mqttClient->state());
        return false;
    }
}

void OLED_OTA::_mqttCallback(char* topic, byte* payload, unsigned int length) {
    Serial.println("[OLED_OTA] Nhận được thông báo từ topic: " + String(topic));
    
    // Parse JSON payload
    StaticJsonDocument<512> doc;
    DeserializationError error = deserializeJson(doc, payload, length);

    if (error) {
        Serial.print("[OLED_OTA] Lỗi phân tích JSON: ");
        Serial.println(error.c_str());
        return;
    }

    const char* version = doc["version"];
    const char* url = doc["url"];
    
    if (version && url) {
        String newVersion = String(version);
        String downloadUrl = String(url);
        
        Serial.println("[OLED_OTA] Phiên bản mới khả dụng: " + newVersion);
        Serial.println("[OLED_OTA] URL tải xuống: " + downloadUrl);
        
        // Simple version comparison (can be improved for semantic versioning)
        if (newVersion != _currentVersion) {
            Serial.println("[OLED_OTA] Bắt đầu cập nhật OTA...");
            _performUpdate(downloadUrl, newVersion);
        } else {
            Serial.println("[OLED_OTA] Thiết bị đã ở phiên bản mới nhất.");
        }
    }
}

void OLED_OTA::_performUpdate(String url, String newVersion) {
    // Append Token to the URL if not already present
    if (url.indexOf("?") == -1) {
        url += "?token=" + _projectToken;
    } else {
        url += "&token=" + _projectToken;
    }

    Serial.println("[OLED_OTA] Đang tiến hành cài đặt firmware mới...");
    
    WiFiClient client;
    
    httpUpdate.onStart([]() { Serial.println("[OLED_OTA] OTA Bắt đầu..."); });
    httpUpdate.onEnd([]() { Serial.println("\n[OLED_OTA] OTA Hoàn tất. Đang khởi động lại..."); });
    httpUpdate.onProgress([](int cur, int total) {
        Serial.printf("[OLED_OTA] Tiến trình: %d%%\r", (cur * 100) / total);
    });
    httpUpdate.onError([](int err) {
        Serial.printf("[OLED_OTA] Lỗi cập nhật[%d]: %s\n", err, httpUpdate.getLastErrorString().c_str());
    });
    
    httpUpdate.rebootOnUpdate(true); // Automatically reboot after successful update
    
    t_httpUpdate_return ret = httpUpdate.update(client, url, _currentVersion);
    
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
}
