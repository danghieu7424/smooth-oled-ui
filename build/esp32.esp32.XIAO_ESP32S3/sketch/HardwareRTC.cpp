#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\HardwareRTC.cpp"
#include "HardwareRTC.h"

HardwareRTC rtc;

HardwareRTC::HardwareRTC() {
}

bool HardwareRTC::begin() {
    Wire.beginTransmission(DS3231_ADDR);
    if (Wire.endTransmission() == 0) {
        return true;
    }
    return false;
}

uint8_t HardwareRTC::decToBcd(uint8_t val) {
    return ( (val / 10 * 16) + (val % 10) );
}

uint8_t HardwareRTC::bcdToDec(uint8_t val) {
    return ( (val / 16 * 10) + (val % 16) );
}

void HardwareRTC::readTime(int &hour, int &minute, int &second) {
    Wire.beginTransmission(DS3231_ADDR);
    Wire.write(0x00); // Đặt con trỏ thanh ghi về 0x00
    Wire.endTransmission();

    Wire.requestFrom(DS3231_ADDR, (uint8_t)3);
    if (Wire.available() >= 3) {
        second = bcdToDec(Wire.read() & 0x7F);
        minute = bcdToDec(Wire.read());
        hour = bcdToDec(Wire.read() & 0x3F); // Bỏ qua cờ 12/24h
    }
}

void HardwareRTC::readDate(int &day, int &month, int &year) {
    Wire.beginTransmission(DS3231_ADDR);
    Wire.write(0x04); // Đặt con trỏ thanh ghi về 0x04 (Date)
    Wire.endTransmission();

    Wire.requestFrom(DS3231_ADDR, (uint8_t)3);
    if (Wire.available() >= 3) {
        day = bcdToDec(Wire.read());
        month = bcdToDec(Wire.read() & 0x1F); // Bỏ qua cờ Century
        year = bcdToDec(Wire.read()) + 2000;
    }
}

void HardwareRTC::adjust(int hour, int minute, int second, int day, int month, int year) {
    Wire.beginTransmission(DS3231_ADDR);
    Wire.write(0x00); // Bắt đầu ghi từ thanh ghi 0x00
    Wire.write(decToBcd(second));
    Wire.write(decToBcd(minute));
    Wire.write(decToBcd(hour));
    Wire.write(1); // Day of week (không quan trọng lắm trong hiển thị hiện tại)
    Wire.write(decToBcd(day));
    Wire.write(decToBcd(month));
    Wire.write(decToBcd(year - 2000));
    Wire.endTransmission();
}
