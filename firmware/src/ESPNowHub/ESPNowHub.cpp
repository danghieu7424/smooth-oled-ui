#include "ESPNowHub.h"

ESPNowHub espNowHub;

ESPNowHub::ESPNowHub() {
    _initialized = false;
    uint8_t mac[6] = {0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF};
    memcpy(_broadcastMac, mac, 6);
}

void ESPNowHub::begin() {
    if (esp_now_init() == ESP_OK) {
        _initialized = true;
        
        esp_now_peer_info_t peerInfo;
        memset(&peerInfo, 0, sizeof(peerInfo));
        memcpy(peerInfo.peer_addr, _broadcastMac, 6);
        peerInfo.channel = 0;  
        peerInfo.encrypt = false;
        
        if (esp_now_add_peer(&peerInfo) != ESP_OK){
            Serial.println("[ESP-NOW] Lỗi thêm peer broadcast");
        } else {
            Serial.println("[ESP-NOW] Khởi tạo thành công (Broadcast Mode)");
        }
    } else {
        Serial.println("[ESP-NOW] Khởi tạo thất bại");
    }
}

void ESPNowHub::broadcastToggle() {
    if (!_initialized) return;
    uint8_t cmd = 1; // 1 = Toggle
    sendCommand(_broadcastMac, cmd);
}

void ESPNowHub::sendCommand(uint8_t* mac, uint8_t cmd) {
    if (!_initialized) return;
    esp_err_t result = esp_now_send(mac, &cmd, 1);
    if (result == ESP_OK) {
        Serial.println("[ESP-NOW] Đã gửi lệnh thành công");
    } else {
        Serial.println("[ESP-NOW] Gửi lệnh thất bại");
    }
}
