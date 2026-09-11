#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\src\\WebConfig\\WebConfig.h"
#ifndef WEBCONFIG_H
#define WEBCONFIG_H

#include <Arduino.h>
#include <WiFi.h>
#include <WebServer.h>

class WebConfig {
public:
    WebConfig();
    void begin();
    void loop();
    bool isConfiguring();
    void stop();
    
private:
    WebServer* _server;
    bool _configuring;
    unsigned long _startTime;
};

extern WebConfig webConfig;
#endif // WEBCONFIG_H
