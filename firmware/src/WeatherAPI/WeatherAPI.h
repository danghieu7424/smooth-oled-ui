#ifndef WEATHERAPI_H
#define WEATHERAPI_H

#include <Arduino.h>
#include <WiFi.h>
#include <HTTPClient.h>
#include <ArduinoJson.h>

class WeatherAPI {
public:
    WeatherAPI();
    void setApiKey(const char* key, const char* city);
    bool update();
    
    String temp_str;
    String desc_str;
    String icon_code;
    bool is_synced;

private:
    String _apiKey;
    String _city;
};

extern WeatherAPI weatherApi;

#endif // WEATHERAPI_H
