#ifndef HARDWARE_RTC_H
#define HARDWARE_RTC_H

#include <Arduino.h>
#include <Wire.h>

class HardwareRTC {
public:
    HardwareRTC();
    
    // Khởi tạo RTC
    bool begin();
    
    void readTime(int &hour, int &minute, int &second);
    void readDate(int &day, int &month, int &year);
    float readTemperature();
    
    // Cài đặt thời gian cho RTC (được gọi khi đồng bộ API thành công)
    void adjust(int hour, int minute, int second, int day, int month, int year);
    
private:
    const uint8_t DS3231_ADDR = 0x68;
    
    uint8_t decToBcd(uint8_t val);
    uint8_t bcdToDec(uint8_t val);
};

extern HardwareRTC rtc;

#endif
