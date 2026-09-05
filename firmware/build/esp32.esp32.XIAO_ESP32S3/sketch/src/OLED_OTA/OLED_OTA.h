#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\src\\OLED_OTA\\OLED_OTA.h"
#ifndef OLED_OTA_H
#define OLED_OTA_H

#include <Arduino.h>
#include <WiFi.h>
#include <WiFiClient.h>
#include <WiFiClient.h>
#include <HTTPClient.h>
#include <HTTPUpdate.h>
#include <ArduinoJson.h>

class OLED_OTA {
public:
    // Khởi tạo thư viện với ID dự án, Token bảo mật và Phiên bản hiện tại
    OLED_OTA(const char* projectId, const char* projectToken, const char* currentVersion);
    
    // Cấu hình URL HTTP Server (Thay cho MQTT)
    void setApiEndpoint(const char* host, uint16_t port);
    
    // Khởi động dịch vụ
    void begin();
    
    // Gọi hàm này trong loop() của Arduino để kiểm tra cập nhật định kỳ
    void loop();

private:
    String _projectId;
    String _projectToken;
    String _currentVersion;
    
    String _apiHost;
    uint16_t _apiPort;
    
    String _deviceId;
    
    WiFiClient _wifiClient;
    
    unsigned long _lastCheckAttempt;
    
    void _checkUpdate();
    void _performUpdate(String url, String newVersion);
};

#endif // OLED_OTA_H
