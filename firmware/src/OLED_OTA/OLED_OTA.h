#ifndef OLED_OTA_H
#define OLED_OTA_H

#include <Arduino.h>
#include <WiFi.h>
#include <WiFiClient.h>
#include <WiFiClientSecure.h>
#include <HTTPClient.h>
#include <esp_ota_ops.h>
#include <esp_partition.h>

/****
 * OLED_OTA — OTA Service dùng ESP-IDF OTA API trực tiếp
 * Lý do KHÔNG dùng Arduino Update.h:
 *   esp_partition_write() trên ESP32-S3 bị "silent fail" — trả OK nhưng không ghi Flash.
 *   esp_ota_write() dùng đường ghi OTA chuyên dụng, bypass lỗi này.
 ****/
class OLED_OTA {
public:
    OLED_OTA(const char* projectId, const char* projectToken, const char* currentVersion);
    
    void setApiEndpoint(const char* host, uint16_t port);
    void begin();
    void loop();

private:
    String _projectId;
    String _projectToken;
    String _currentVersion;
    
    String _apiHost;
    uint16_t _apiPort;
    
    String _deviceId;
    
    unsigned long _lastCheckAttempt;
    
    void _checkUpdate();
    void _performUpdate(String url, String newVersion);
};

#endif // OLED_OTA_H
