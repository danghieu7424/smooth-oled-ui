#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\src\\TimeSyncAPI\\TimeSyncAPI.h"
#ifndef TIMESYNCAPI_H
#define TIMESYNCAPI_H

#include <Arduino.h>
#include <WiFi.h>
#include <WiFiClientSecure.h>
#include <HTTPClient.h>
#include <ArduinoJson.h>

class TimeSyncAPI {
private:
    int _tz;
    
public:
    int current_hour;
    int current_minute;
    int current_second;
    String solar_date_str;
    String lunar_date_str;
    String current_temp_str;
    uint32_t last_time_sync;
    uint32_t last_second_tick;
    bool is_time_synced;
    bool api_synced; // Cờ kiểm tra xem đã lấy giờ chuẩn từ web chưa

    TimeSyncAPI();
    
    // Khởi tạo thư viện với múi giờ mặc định
    void begin(int tz = 7);
    
    // Đọc giờ từ RTC để khỏi chờ mạng
    void syncFromRTC();
    
    // Gọi API để đồng bộ thời gian từ dh74.io.vn
    bool update();
    
    // Hàm gọi định kỳ trong loop() để đếm giây nội bộ
    bool tick();
};

extern TimeSyncAPI timeSync;

#endif
