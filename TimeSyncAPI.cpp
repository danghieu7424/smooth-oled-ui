#include "TimeSyncAPI.h"

TimeSyncAPI timeSync;

TimeSyncAPI::TimeSyncAPI() {
    _tz = 7;
    current_hour = 0;
    current_minute = 0;
    current_second = 0;
    solar_date_str = "";
    lunar_date_str = "";
    last_time_sync = 0;
    last_second_tick = 0;
    is_time_synced = false;
}

void TimeSyncAPI::begin(int tz) {
    _tz = tz;
}

bool TimeSyncAPI::update() {
    if (WiFi.status() != WL_CONNECTED) {
        Serial.println("[API] WiFi NOT connected!");
        return false;
    }

    Serial.println("[API] Trying to sync time...");
    WiFiClientSecure *client = new WiFiClientSecure;
    client->setInsecure();
    HTTPClient http;
    
    char url[64];
    snprintf(url, sizeof(url), "https://dh74.io.vn/api/time?tz=%d", _tz);
    
    bool success = false;
    if (http.begin(*client, url)) {
        int httpCode = http.GET();
        Serial.printf("[API] HTTP Code: %d\n", httpCode);
        if (httpCode == HTTP_CODE_OK) {
            String payload = http.getString();
            JsonDocument doc;
            DeserializationError error = deserializeJson(doc, payload);
            if (!error) {
                current_hour = doc["data"]["time"]["hour"];
                current_minute = doc["data"]["time"]["minute"];
                current_second = doc["data"]["time"]["second"];
                
                int s_d = doc["data"]["solar"]["day"];
                int s_m = doc["data"]["solar"]["month"];
                int s_y = doc["data"]["solar"]["year"];
                char buf[32];
                snprintf(buf, sizeof(buf), "%02d/%02d/%04d", s_d, s_m, s_y);
                solar_date_str = String(buf);
                
                int l_d = doc["data"]["lunar"]["day"];
                int l_m = doc["data"]["lunar"]["month"];
                char l_buf[64];
                snprintf(l_buf, sizeof(l_buf), "AL: %02d/%02d", l_d, l_m);
                lunar_date_str = String(l_buf);
                
                is_time_synced = true;
                last_time_sync = millis();
                last_second_tick = millis();
                
                Serial.println("[API] Sync SUCCESS!");
                success = true;
            } else {
                Serial.println("[API] JSON Parse Error!");
            }
        } else {
            Serial.println("[API] HTTP Request Failed!");
        }
        http.end();
    } else {
        Serial.println("[API] HTTP Begin Failed!");
    }
    delete client;
    return success;
}

bool TimeSyncAPI::tick() {
    if (!is_time_synced) return false;
    
    if (millis() - last_second_tick >= 1000) {
        last_second_tick += 1000;
        current_second++;
        if (current_second >= 60) {
            current_second = 0;
            current_minute++;
            if (current_minute >= 60) {
                current_minute = 0;
                current_hour++;
                if (current_hour >= 24) {
                    current_hour = 0;
                }
            }
        }
        return true; // Thời gian đã thay đổi
    }
    return false;
}
