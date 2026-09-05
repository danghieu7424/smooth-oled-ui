#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\flash_diag\\flash_diag.ino"
#include <Arduino.h>
#include <esp32s3/rom/spi_flash.h>

#line 4 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\flash_diag\\flash_diag.ino"
void setup();
#line 40 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\flash_diag\\flash_diag.ino"
void loop();
#line 4 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\flash_diag\\flash_diag.ino"
void setup() {
    Serial.begin(115200);
    delay(3000);
    
    Serial.println("\n\n=== FLASH QE BIT FIX ===");
    
    uint32_t status = 0;
    esp_rom_spiflash_read_status(&g_rom_flashchip, &status);
    Serial.printf("Current Status (SR1/SR2): 0x%04X\n", status & 0xFFFF);
    
    if ((status & 0x0200) == 0) {
        Serial.println("QE (Quad Enable) bit is 0. Enabling it now...");
        
        // SR1 = 0x00, SR2 = 0x02 (QE bit is bit 1 of SR2, which is bit 9 of status)
        uint32_t new_status = (status & 0xFF00) | 0x0200; // Set bit 9
        new_status &= 0x02FF; // Clear BP bits just in case
        
        Serial.printf("Writing new status: 0x%04X\n", new_status);
        
        esp_rom_spiflash_unlock();
        esp_rom_spiflash_write_status(&g_rom_flashchip, new_status);
        
        uint32_t verify = 0;
        esp_rom_spiflash_read_status(&g_rom_flashchip, &verify);
        Serial.printf("Verified Status: 0x%04X\n", verify & 0xFFFF);
        
        if (verify & 0x0200) {
            Serial.println("SUCCESS! QE bit is now 1. Flash writes should work!");
        } else {
            Serial.println("FAILED to set QE bit!");
        }
    } else {
        Serial.println("QE bit is already 1. No fix needed.");
    }
}

void loop() {
    delay(1000);
}

