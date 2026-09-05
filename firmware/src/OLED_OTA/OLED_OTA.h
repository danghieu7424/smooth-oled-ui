#ifndef OLED_OTA_H
#define OLED_OTA_H

#include <Arduino.h>
#include <WiFi.h>
#include <WiFiClient.h>
#include <PubSubClient.h>
#include <HTTPClient.h>
#include <HTTPUpdate.h>
#include <ArduinoJson.h>

class OLED_OTA {
public:
    // Khởi tạo thư viện với ID dự án, Token bảo mật và Phiên bản hiện tại
    OLED_OTA(const char* projectId, const char* projectToken, const char* currentVersion);
    
    // Cấu hình MQTT Broker
    void setMqttBroker(const char* broker, uint16_t port, const char* user = "", const char* pass = "");
    
    // Khởi động dịch vụ (Connect MQTT, đăng ký topic)
    void begin();
    
    // Gọi hàm này trong loop() của Arduino để duy trì kết nối MQTT
    void loop();

private:
    String _projectId;
    String _projectToken;
    String _currentVersion;
    
    String _mqttBroker;
    uint16_t _mqttPort;
    String _mqttUser;
    String _mqttPass;
    
    String _deviceId;
    String _otaTopic;
    
    WiFiClient _wifiClient;
    PubSubClient* _mqttClient;
    
    unsigned long _lastReconnectAttempt;
    
    bool _connectMqtt();
    void _mqttCallback(char* topic, byte* payload, unsigned int length);
    void _performUpdate(String url, String newVersion);
};

#endif // OLED_OTA_H
