#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\ExternalEEPROM.cpp"
#include "ExternalEEPROM.h"

ExternalEEPROM extEEPROM;

ExternalEEPROM::ExternalEEPROM() {
}

bool ExternalEEPROM::begin() {
    Wire.beginTransmission(EEPROM_ADDR);
    if (Wire.endTransmission() == 0) {
        return true;
    }
    return false;
}

void ExternalEEPROM::writeByte(uint16_t memoryAddress, uint8_t data) {
    Wire.beginTransmission(EEPROM_ADDR);
    Wire.write((int)(memoryAddress >> 8));   // MSB
    Wire.write((int)(memoryAddress & 0xFF)); // LSB
    Wire.write(data);
    Wire.endTransmission();
    delay(5); // AT24C256 cần khoảng 5ms để ghi
}

uint8_t ExternalEEPROM::readByte(uint16_t memoryAddress) {
    uint8_t rData = 0xFF;
    Wire.beginTransmission(EEPROM_ADDR);
    Wire.write((int)(memoryAddress >> 8));   // MSB
    Wire.write((int)(memoryAddress & 0xFF)); // LSB
    Wire.endTransmission();
    
    Wire.requestFrom(EEPROM_ADDR, (uint8_t)1);
    if (Wire.available()) {
        rData = Wire.read();
    }
    return rData;
}

void ExternalEEPROM::writeBytes(uint16_t memoryAddress, const uint8_t* data, size_t length) {
    // Để an toàn và tránh rớt trang (page boundary 64 byte), ghi từng byte cho cấu hình nhỏ.
    // Dành cho dữ liệu nhỏ lẻ (dưới 100 bytes).
    for (size_t i = 0; i < length; i++) {
        writeByte(memoryAddress + i, data[i]);
    }
}

void ExternalEEPROM::readBytes(uint16_t memoryAddress, uint8_t* data, size_t length) {
    // Đọc không bị giới hạn page boundary nhưng bị giới hạn bởi buffer Wire (thường 32-128 bytes)
    // Đọc từng chunk 32 bytes để an toàn tuyệt đối
    size_t bytesRead = 0;
    while (bytesRead < length) {
        size_t toRead = length - bytesRead;
        if (toRead > 32) toRead = 32;
        
        Wire.beginTransmission(EEPROM_ADDR);
        Wire.write((int)((memoryAddress + bytesRead) >> 8));
        Wire.write((int)((memoryAddress + bytesRead) & 0xFF));
        Wire.endTransmission();
        
        Wire.requestFrom(EEPROM_ADDR, (uint8_t)toRead);
        for (size_t i = 0; i < toRead; i++) {
            if (Wire.available()) {
                data[bytesRead + i] = Wire.read();
            }
        }
        bytesRead += toRead;
    }
}

void ExternalEEPROM::writeString(uint16_t memoryAddress, const String &str) {
    size_t len = str.length();
    for (size_t i = 0; i < len; i++) {
        writeByte(memoryAddress + i, str[i]);
    }
    // Ghi ký tự kết thúc chuỗi \0
    writeByte(memoryAddress + len, '\0');
}

String ExternalEEPROM::readString(uint16_t memoryAddress, size_t maxLength) {
    String result = "";
    for (size_t i = 0; i < maxLength; i++) {
        char c = (char)readByte(memoryAddress + i);
        if (c == '\0' || c == 0xFF) { // Đụng cuối chuỗi hoặc vùng nhớ rỗng
            break;
        }
        result += c;
    }
    return result;
}
