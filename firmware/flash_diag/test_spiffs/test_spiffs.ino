#include <Arduino.h>
#include <SPIFFS.h>

void setup() {
    Serial.begin(115200);
    delay(2000);
    
    Serial.println("\n\n--- TEST SPIFFS ---");
    if (!SPIFFS.begin(true)) {
        Serial.println("SPIFFS Mount Failed");
        return;
    }
    Serial.println("SPIFFS Mounted!");
    
    File f = SPIFFS.open("/test.txt", "w");
    if (!f) {
        Serial.println("Failed to open file for writing");
    } else {
        f.println("Hello OTA Flash Test!");
        f.close();
        Serial.println("Wrote to /test.txt");
    }
    
    File f2 = SPIFFS.open("/test.txt", "r");
    if (!f2) {
        Serial.println("Failed to open file for reading");
    } else {
        String s = f2.readString();
        f2.close();
        Serial.printf("Read back: '%s'\n", s.c_str());
    }
    Serial.println("--- END ---");
}

void loop() {
    delay(1000);
}
