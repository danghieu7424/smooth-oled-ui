#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\src\\ESPNowHub\\ESPNowHub.h"
#ifndef ESPNOWHUB_H
#define ESPNOWHUB_H

#include <Arduino.h>
#include <WiFi.h>
#include <esp_now.h>

class ESPNowHub {
public:
    ESPNowHub();
    void begin();
    void broadcastToggle();
    void sendCommand(uint8_t* mac, uint8_t cmd);

private:
    bool _initialized;
    uint8_t _broadcastMac[6];
};

extern ESPNowHub espNowHub;

#endif // ESPNOWHUB_H
