#ifndef TIMESYNCAPI_H
#define TIMESYNCAPI_H

#include <Arduino.h>
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
    uint32_t last_time_sync;
    uint32_t last_second_tick;
    bool is_time_synced;

    TimeSyncAPI();
    
    // Khởi tạo thư viện với múi giờ mặc định
    void begin(int tz = 7);
    
    // Gọi API để đồng bộ thời gian từ dh74.io.vn
    bool update();
    
    // Hàm gọi định kỳ trong loop() để đếm giây nội bộ
    bool tick();
};

extern TimeSyncAPI timeSync;

#endif
