#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\src\\WeatherAPI\\WeatherAPI.cpp"
#include "WeatherAPI.h"

WeatherAPI weatherApi;

WeatherAPI::WeatherAPI() {
    temp_str = "--.-C";
    desc_str = "No Data";
    icon_code = "";
    is_synced = false;
    _apiKey = "DEMO_KEY"; // Thay API Key OpenWeatherMap vào đây
    _city = "Hanoi";
}

void WeatherAPI::setApiKey(const char* key, const char* city) {
    _apiKey = String(key);
    _city = String(city);
}

bool WeatherAPI::update() {
    if (WiFi.status() != WL_CONNECTED) return false;
    if (_apiKey == "DEMO_KEY") {
        // Dummy data khi chưa có API Key
        temp_str = "29.5C";
        desc_str = "Nắng nhẹ";
        icon_code = "01d";
        is_synced = true;
        return true;
    }

    HTTPClient http;
    String url = "http://api.openweathermap.org/data/2.5/weather?q=" + _city + "&appid=" + _apiKey + "&units=metric&lang=vi";
    
    bool success = false;
    if (http.begin(url)) {
        int httpCode = http.GET();
        if (httpCode == HTTP_CODE_OK) {
            String payload = http.getString();
            JsonDocument doc;
            DeserializationError error = deserializeJson(doc, payload);
            if (!error) {
                float t = doc["main"]["temp"];
                char t_buf[16];
                snprintf(t_buf, sizeof(t_buf), "%.1fC", t);
                temp_str = String(t_buf);
                
                desc_str = doc["weather"][0]["description"].as<String>();
                icon_code = doc["weather"][0]["icon"].as<String>();
                
                is_synced = true;
                success = true;
                Serial.printf("[Weather] Sync OK: %s, %s\n", temp_str.c_str(), desc_str.c_str());
            }
        }
        http.end();
    }
    return success;
}
