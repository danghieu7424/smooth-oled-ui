#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\src\\ExternalEEPROM\\ExternalEEPROM.h"
#ifndef EXTERNAL_EEPROM_H
#define EXTERNAL_EEPROM_H

#include <Arduino.h>
#include <Wire.h>

class ExternalEEPROM {
public:
    ExternalEEPROM();
    
    // Kiểm tra kết nối với EEPROM
    bool begin();
    
    // Đọc/Ghi 1 byte
    void writeByte(uint16_t memoryAddress, uint8_t data);
    uint8_t readByte(uint16_t memoryAddress);
    
    // Đọc/Ghi nhiều byte (ví dụ: mảng)
    void writeBytes(uint16_t memoryAddress, const uint8_t* data, size_t length);
    void readBytes(uint16_t memoryAddress, uint8_t* data, size_t length);
    
    // Đọc/Ghi chuỗi String
    void writeString(uint16_t memoryAddress, const String &str);
    String readString(uint16_t memoryAddress, size_t maxLength);

private:
    const uint8_t EEPROM_ADDR = 0x50; // Mặc định của module AT24C256
};

extern ExternalEEPROM extEEPROM;

#endif
