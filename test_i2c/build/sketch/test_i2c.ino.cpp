#include <Arduino.h>
#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\test_i2c\\test_i2c.ino"
#include <Wire.h>

#line 3 "D:\\all_projects\\rust\\rust\\display_oled\\test_i2c\\test_i2c.ino"
void setup();
#line 11 "D:\\all_projects\\rust\\rust\\display_oled\\test_i2c\\test_i2c.ino"
void loop();
#line 3 "D:\\all_projects\\rust\\rust\\display_oled\\test_i2c\\test_i2c.ino"
void setup() {
  Serial.begin(921600);
  while (!Serial); // wait for serial monitor
  
  Serial.println("\nI2C Scanner");
  Wire.begin(47, 48); // Khởi tạo I2C trên pin 47 (SDA), 48 (SCL)
}

void loop() {
  byte error, address;
  int nDevices;

  Serial.println("Scanning...");

  nDevices = 0;
  for(address = 1; address < 127; address++ ) {
    Wire.beginTransmission(address);
    error = Wire.endTransmission();

    if (error == 0) {
      Serial.print("I2C device found at address 0x");
      if (address < 16) 
        Serial.print("0");
      Serial.print(address, HEX);
      Serial.println("  !");
      nDevices++;
    }
    else if (error == 4) {
      Serial.print("Unknown error at address 0x");
      if (address < 16) 
        Serial.print("0");
      Serial.println(address, HEX);
    }    
  }
  
  if (nDevices == 0)
    Serial.println("No I2C devices found\n");
  else
    Serial.println("done\n");

  delay(5000); // Wait 5 seconds for next scan
}

