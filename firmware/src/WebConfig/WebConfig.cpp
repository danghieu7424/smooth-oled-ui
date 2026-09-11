#include "WebConfig.h"

extern void save_wifi_credentials(String ssid, String pwd); // Khai báo từ firmware.ino

const char* html_page = R"HTML(
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>ESP32 OLED Config</title>
    <style>
        body { font-family: 'Inter', sans-serif; background: #121212; color: #fff; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; }
        .glass { background: rgba(255, 255, 255, 0.1); border-radius: 16px; padding: 30px; backdrop-filter: blur(10px); -webkit-backdrop-filter: blur(10px); border: 1px solid rgba(255, 255, 255, 0.2); box-shadow: 0 4px 30px rgba(0, 0, 0, 0.5); text-align: center; width: 300px; }
        input { width: 90%; padding: 10px; margin: 10px 0; border: none; border-radius: 8px; background: rgba(255,255,255,0.2); color: #fff; font-size: 16px; outline: none; }
        input::placeholder { color: #ccc; }
        button { width: 100%; padding: 12px; border: none; border-radius: 8px; background: #007BFF; color: #fff; font-size: 16px; font-weight: bold; cursor: pointer; transition: 0.3s; }
        button:hover { background: #0056b3; }
    </style>
</head>
<body>
    <div class="glass">
        <h2>WiFi Config</h2>
        <form action="/save" method="POST">
            <input type="text" name="ssid" placeholder="Tên WiFi (SSID)" required>
            <input type="password" name="pwd" placeholder="Mật khẩu WiFi">
            <button type="submit">Kết Nối</button>
        </form>
    </div>
</body>
</html>
)HTML";

WebConfig webConfig;

WebConfig::WebConfig() {
    _server = nullptr;
    _configuring = false;
}

void WebConfig::begin() {
    WiFi.mode(WIFI_AP);
    WiFi.softAP("ESP32-OLED-Hub");
    
    if (_server) delete _server;
    _server = new WebServer(80);
    
    _server->on("/", HTTP_GET, [this]() {
        _server->send(200, "text/html", html_page);
    });
    
    _server->on("/save", HTTP_POST, [this]() {
        String ssid = _server->arg("ssid");
        String pwd = _server->arg("pwd");
        
        _server->send(200, "text/html", "<h2>Đã lưu! Đang khởi động lại...</h2>");
        
        save_wifi_credentials(ssid, pwd);
        delay(1000);
        ESP.restart();
    });
    
    _server->begin();
    _configuring = true;
    _startTime = millis();
    Serial.println("[WebConfig] Đã khởi tạo điểm phát sóng ESP32-OLED-Hub (192.168.4.1)");
}

void WebConfig::loop() {
    if (_configuring && _server) {
        _server->handleClient();
        
        // Timeout sau 5 phút tự thoát config
        if (millis() - _startTime > 300000) {
            stop();
        }
    }
}

bool WebConfig::isConfiguring() {
    return _configuring;
}

void WebConfig::stop() {
    if (_configuring) {
        if (_server) {
            _server->stop();
            delete _server;
            _server = nullptr;
        }
        WiFi.softAPdisconnect(true);
        _configuring = false;
        Serial.println("[WebConfig] Đã tắt chế độ WebConfig");
    }
}
