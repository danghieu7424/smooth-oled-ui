#include "HardwareRTC.h"

HardwareRTC rtc;

HardwareRTC::HardwareRTC() {
}

bool HardwareRTC::begin() {
    Wire1.beginTransmission(DS3231_ADDR);
    if (Wire1.endTransmission() == 0) {
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
    Wire1.beginTransmission(DS3231_ADDR);
    Wire1.write(0x00); // Đặt con trỏ thanh ghi về 0x00
    Wire1.endTransmission();

    Wire1.requestFrom(DS3231_ADDR, (uint8_t)3);
    if (Wire1.available() >= 3) {
        second = bcdToDec(Wire1.read() & 0x7F);
        minute = bcdToDec(Wire1.read());
        hour = bcdToDec(Wire1.read() & 0x3F); // Bỏ qua cờ 12/24h
    }
}

void HardwareRTC::readDate(int &day, int &month, int &year) {
    Wire1.beginTransmission(DS3231_ADDR);
    Wire1.write(0x04); // Đặt con trỏ thanh ghi về 0x04 (Date)
    Wire1.endTransmission();

    Wire1.requestFrom(DS3231_ADDR, (uint8_t)3);
    if (Wire1.available() >= 3) {
        day = bcdToDec(Wire1.read());
        month = bcdToDec(Wire1.read() & 0x1F); // Bỏ qua cờ Century
        year = bcdToDec(Wire1.read()) + 2000;
    }
}

void HardwareRTC::adjust(int hour, int minute, int second, int day, int month, int year) {
    Wire1.beginTransmission(DS3231_ADDR);
    Wire1.write(0x00); // Bắt đầu ghi từ thanh ghi 0x00
    Wire1.write(decToBcd(second));
    Wire1.write(decToBcd(minute));
    Wire1.write(decToBcd(hour));
    Wire1.write(1); // Day of week (không quan trọng lắm trong hiển thị hiện tại)
    Wire1.write(decToBcd(day));
    Wire1.write(decToBcd(month));
    Wire1.write(decToBcd(year - 2000));
    Wire1.endTransmission();
}

float HardwareRTC::readTemperature() {
    Wire1.beginTransmission(DS3231_ADDR);
    Wire1.write(0x11);
    Wire1.endTransmission();

    Wire1.requestFrom(DS3231_ADDR, (uint8_t)2);
    if (Wire1.available() >= 2) {
        int8_t msb = Wire1.read();
        uint8_t lsb = Wire1.read();
        return (float)msb + ((lsb >> 6) * 0.25f);
    }
    return 0.0f;
}
