#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\TimeSyncAPI.cpp"
#include "TimeSyncAPI.h"
#include "HardwareRTC.h"

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
    api_synced = false;
    current_temp_str = "--.-C";
}

// Hàm hỗ trợ tính thứ (0=CN, 1=T2, ...)
static int getDayOfWeek(int d, int m, int y) {
    static int t[] = { 0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4 };
    y -= m < 3 ? 1 : 0;
    return ( y + y/4 - y/100 + y/400 + t[m-1] + d ) % 7;
}


void TimeSyncAPI::begin(int tz) {
    _tz = tz;
}

void TimeSyncAPI::syncFromRTC() {
    int h, m, s, d, mo, y;
    rtc.readTime(h, m, s);
    rtc.readDate(d, mo, y);
    
    current_hour = h;
    current_minute = m;
    current_second = s;
    
    int dow = getDayOfWeek(d, mo, y);
    const char* dow_str[] = {"CN", "T2", "T3", "T4", "T5", "T6", "T7"};
    char buf[64];
    snprintf(buf, sizeof(buf), "%s %02d Th%02d %04d", dow_str[dow], d, mo, y);
    solar_date_str = String(buf);
    
    float t = rtc.readTemperature();
    char temp_buf[16];
    snprintf(temp_buf, sizeof(temp_buf), "%.1fC", t);
    current_temp_str = String(temp_buf);
    
    // Lưu ý: RTC không lưu Âm lịch, ta đành để tạm "--/--" hoặc tính toán nếu có thuật toán
    // Hiện tại chờ API cập nhật Âm lịch sau.
    if (lunar_date_str == "") {
        lunar_date_str = "AL: -- Th--";
    }
    
    is_time_synced = true;
    last_second_tick = millis();
    Serial.printf("[RTC] Synced: %02d:%02d:%02d %s\n", h, m, s, buf);
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
                int dow = getDayOfWeek(s_d, s_m, s_y);
                const char* dow_str[] = {"CN", "T2", "T3", "T4", "T5", "T6", "T7"};
                char buf[64];
                snprintf(buf, sizeof(buf), "%s %02d Th%02d %04d", dow_str[dow], s_d, s_m, s_y);
                solar_date_str = String(buf);
                
                int l_d = doc["data"]["lunar"]["day"];
                int l_m = doc["data"]["lunar"]["month"];
                char l_buf[64];
                snprintf(l_buf, sizeof(l_buf), "AL: %02d Th%02d", l_d, l_m);
                lunar_date_str = String(l_buf);
                
                is_time_synced = true;
                api_synced = true;
                last_time_sync = millis();
                last_second_tick = millis();
                
                // Cập nhật lại cho DS3231
                rtc.adjust(current_hour, current_minute, current_second, s_d, s_m, s_y);
                
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
    
    // Cập nhật mốc thời gian kể cả khi lỗi để tránh spam request liên tục làm treo máy
    last_time_sync = millis();
    
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
            
            // Cập nhật nhiệt độ mỗi 1 phút (khi sang phút mới)
            float t = rtc.readTemperature();
            char temp_buf[16];
            snprintf(temp_buf, sizeof(temp_buf), "%.1fC", t);
            current_temp_str = String(temp_buf);
            
            if (current_minute >= 60) {
                current_minute = 0;
                current_hour++;
                if (current_hour >= 24) {
                    current_hour = 0;
                    
                    // Sang ngày mới: Buộc cập nhật ngày tháng
                    if (WiFi.status() == WL_CONNECTED) {
                        if (!update()) {
                            syncFromRTC();
                        }
                    } else {
                        syncFromRTC();
                    }
                }
            }
        }
        return true; // Thời gian đã thay đổi
    }
    return false;
}
