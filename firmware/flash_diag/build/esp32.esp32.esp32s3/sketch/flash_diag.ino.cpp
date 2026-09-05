#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\flash_diag\\flash_diag.ino"
/****
 * FLASH DIAGNOSTIC TOOL - ESP32-S3
 * Chẩn đoán toàn diện khả năng ghi Flash:
 *   1. Thông tin Flash chip (ID, size, mode)
 *   2. eFuse settings (encryption, secure boot)  
 *   3. Flash Status Register (write-protect bits)
 *   4. Ghi/đọc test tại nhiều địa chỉ khác nhau
 *   5. Thử mở khóa Write-Protect nếu phát hiện bị khóa
 ****/

#include <Arduino.h>
#include <esp_flash.h>
#include <esp_flash_spi_init.h>
#include <esp_partition.h>
#include <esp32s3/rom/spi_flash.h>
#include <esp_ota_ops.h>
#include <esp_efuse.h>
#include <esp_efuse_table.h>
#include <soc/efuse_reg.h>

#line 21 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\flash_diag\\flash_diag.ino"
void printHex(const uint8_t* data, int len);
#line 32 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\flash_diag\\flash_diag.ino"
bool testFlashAddress(uint32_t addr, const char* label);
#line 73 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\flash_diag\\flash_diag.ino"
void setup();
#line 208 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\flash_diag\\flash_diag.ino"
void loop();
#line 21 "D:\\all_projects\\rust\\rust\\display_oled\\firmware\\flash_diag\\flash_diag.ino"
void printHex(const uint8_t* data, int len) {
    for (int i = 0; i < len; i++) {
        Serial.printf("%02X ", data[i]);
    }
    Serial.println();
}

/****
 * Test ghi/đọc Flash tại một địa chỉ cụ thể
 * Xóa sector -> Ghi pattern -> Đọc lại -> So sánh
 ****/
bool testFlashAddress(uint32_t addr, const char* label) {
    Serial.printf("\n--- Test: %s (0x%08X) ---\n", label, addr);
    
    uint8_t write_data[16] __attribute__((aligned(4))) = {
        0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF
    };
    uint8_t read_data[16] __attribute__((aligned(4))) = {0};
    
    // Bước 1: Xóa sector (4KB)
    esp_err_t err = spi_flash_erase_range(addr, 4096);
    Serial.printf("  Erase: %s\n", esp_err_to_name(err));
    if (err != ESP_OK) return false;
    
    // Bước 2: Đọc sau khi xóa (phải là FF)
    spi_flash_read(addr, read_data, 16);
    Serial.printf("  After erase: ");
    printHex(read_data, 16);
    
    // Bước 3: Ghi dữ liệu test
    err = spi_flash_write(addr, write_data, 16);
    Serial.printf("  Write: %s\n", esp_err_to_name(err));
    
    // Bước 4: Đọc lại
    memset(read_data, 0, 16);
    spi_flash_read(addr, read_data, 16);
    Serial.printf("  Wrote:    ");
    printHex(write_data, 16);
    Serial.printf("  Readback: ");
    printHex(read_data, 16);
    
    // Bước 5: So sánh
    bool match = (memcmp(write_data, read_data, 16) == 0);
    Serial.printf("  Result: %s\n", match ? ">>> GHI ĐƯỢC <<<" : ">>> KHÔNG GHI ĐƯỢC <<<");
    
    // Xóa lại để không ảnh hưởng
    spi_flash_erase_range(addr, 4096);
    
    return match;
}

void setup() {
    Serial.begin(115200);
    delay(2000);
    
    Serial.println("\n\n========================================");
    Serial.println("  ESP32-S3 FLASH DIAGNOSTIC TOOL");
    Serial.println("========================================\n");

    /****
     * PHẦN 1: Thông tin Flash chip
     ****/
    Serial.println("=== 1. THÔNG TIN FLASH CHIP ===");
    
    uint32_t flash_id = 0;
    esp_flash_read_id(esp_flash_default_chip, &flash_id);
    Serial.printf("  Flash ID: 0x%06X\n", flash_id);
    Serial.printf("  Manufacturer: 0x%02X\n", flash_id & 0xFF);
    Serial.printf("  Device: 0x%04X\n", (flash_id >> 8) & 0xFFFF);
    
    uint32_t flash_size = 0;
    esp_flash_get_size(esp_flash_default_chip, &flash_size);
    Serial.printf("  Flash Size: %u MB (%u bytes)\n", flash_size / (1024*1024), flash_size);
    
    /****
     * PHẦN 2: Kiểm tra Flash Status Register (Write-Protect bits)
     ****/
    Serial.println("\n=== 2. FLASH STATUS REGISTER ===");
    
    // Đọc Status Register 1 (SR1)
    uint32_t sr1 = 0;
    esp_err_t err = (esp_rom_spiflash_read_status(&g_rom_flashchip, &sr1) == ESP_ROM_SPIFLASH_RESULT_OK) ? ESP_OK : ESP_FAIL;
    if (err == ESP_OK) {
        Serial.printf("  Status Register 1: 0x%02X\n", sr1 & 0xFF);
        Serial.printf("    SRP0 (Status Register Protect 0): %d\n", (sr1 >> 7) & 1);
        Serial.printf("    SEC  (Sector Protect):            %d\n", (sr1 >> 6) & 1);
        Serial.printf("    TB   (Top/Bottom Protect):        %d\n", (sr1 >> 5) & 1);
        Serial.printf("    BP2:                              %d\n", (sr1 >> 4) & 1);
        Serial.printf("    BP1:                              %d\n", (sr1 >> 3) & 1);
        Serial.printf("    BP0:                              %d\n", (sr1 >> 2) & 1);
        Serial.printf("    WEL  (Write Enable Latch):        %d\n", (sr1 >> 1) & 1);
        Serial.printf("    BUSY:                             %d\n", sr1 & 1);
        
        uint8_t bp = (sr1 >> 2) & 0x07; // BP2:BP0
        if (bp > 0) {
            Serial.printf("  !!! CẢNH BÁO: Block Protect bits = %d -> Một phần Flash bị KHÓA GHI!\n", bp);
        } else {
            Serial.println("  Block Protect: KHÔNG KHÓA (BP=000)");
        }
    } else {
        Serial.printf("  LỖI đọc SR1: %s\n", esp_err_to_name(err));
    }

    /****
     * PHẦN 3: Kiểm tra eFuse (Encryption, Secure Boot)
     ****/
    Serial.println("\n=== 3. eFUSE SETTINGS ===");
    
    // SPI_BOOT_CRYPT_CNT - Flash encryption counter
    uint32_t crypt_cnt = 0;
    esp_efuse_read_field_blob(ESP_EFUSE_SPI_BOOT_CRYPT_CNT, &crypt_cnt, 3);
    Serial.printf("  SPI_BOOT_CRYPT_CNT: %u", crypt_cnt);
    int ones = 0;
    for (int i = 0; i < 3; i++) { if (crypt_cnt & (1 << i)) ones++; }
    Serial.printf(" (%s)\n", (ones % 2 == 1) ? "ENCRYPTION ON" : "ENCRYPTION OFF");
    
    // SECURE_BOOT_EN
    uint8_t secure_boot = 0;
    esp_efuse_read_field_blob(ESP_EFUSE_SECURE_BOOT_EN, &secure_boot, 1);
    Serial.printf("  SECURE_BOOT_EN: %u (%s)\n", secure_boot, secure_boot ? "ON" : "OFF");

    // DIS_DOWNLOAD_MANUAL_ENCRYPT
    uint8_t dis_manual_enc = 0;
    esp_efuse_read_field_blob(ESP_EFUSE_DIS_DOWNLOAD_MANUAL_ENCRYPT, &dis_manual_enc, 1);
    Serial.printf("  DIS_DOWNLOAD_MANUAL_ENCRYPT: %u\n", dis_manual_enc);

    /****
     * PHẦN 4: Partition Table
     ****/
    Serial.println("\n=== 4. PARTITION TABLE ===");
    
    esp_partition_iterator_t it = esp_partition_find(ESP_PARTITION_TYPE_ANY, ESP_PARTITION_SUBTYPE_ANY, NULL);
    while (it != NULL) {
        const esp_partition_t* p = esp_partition_get(it);
        Serial.printf("  %-10s type=%d sub=%d addr=0x%08X size=%-8u enc=%s\n",
                      p->label, p->type, p->subtype, p->address, p->size,
                      p->encrypted ? "YES" : "NO");
        it = esp_partition_next(it);
    }
    esp_partition_iterator_release(it);

    /****
     * PHẦN 5: Test ghi/đọc tại nhiều địa chỉ khác nhau
     * Test ở cả vùng app0 (cuối), app1 (đầu), spiffs
     ****/
    Serial.println("\n=== 5. FLASH WRITE/READ TESTS ===");
    
    // Test 1: BỎ QUA
    bool t1 = false;
    
    // Test 2: Đầu vùng app1 của 4MB scheme (0x150000)
    bool t2 = testFlashAddress(0x00150000, "Dau app1 (0x150000)");
    
    // Test 3: Giữa vùng app1 của 4MB scheme (0x200000)
    bool t3 = testFlashAddress(0x00200000, "Giua app1 (0x200000)");
    
    // Test 4: Vùng spiffs của 4MB scheme (0x290000)
    bool t4 = testFlashAddress(0x00290000, "Dau spiffs (0x290000)");
    
    // Test 5: Cuối 4MB Flash (0x3F0000)
    bool t5 = testFlashAddress(0x003F0000, "Cuoi flash 4MB (0x3F0000)");

    /****
     * PHẦN 6: Tổng kết
     ****/
    Serial.println("\n=== 6. TỔNG KẾT ===");
    Serial.printf("  Cuối app0 (0x33F000): %s\n", t1 ? "OK" : "FAIL");
    Serial.printf("  Đầu app1  (0x340000): %s\n", t2 ? "OK" : "FAIL");
    Serial.printf("  Giữa app1 (0x400000): %s\n", t3 ? "OK" : "FAIL");
    Serial.printf("  Đầu spiffs(0x670000): %s\n", t4 ? "OK" : "FAIL");
    Serial.printf("  Cuối flash(0x700000): %s\n", t5 ? "OK" : "FAIL");
    
    if (!t2 && t1) {
        Serial.println("\n  >>> KẾT LUẬN: Flash bị Write-Protect TỪ ĐỊA CHỈ 0x340000 TRỞ LÊN!");
        Serial.println("  >>> Nguyên nhân: Block Protect bits trong Status Register hoặc eFuse.");
    } else if (!t2 && !t1) {
        Serial.println("\n  >>> KẾT LUẬN: Flash bị Write-Protect TOÀN BỘ từ phần mềm!");
    } else if (t2) {
        Serial.println("\n  >>> KẾT LUẬN: Flash GHI ĐƯỢC! Có thể do conflict khi chạy OTA.");
    }
    
    Serial.println("\n========================================");
    Serial.println("  KẾT THÚC CHẨN ĐOÁN");
    Serial.println("========================================\n");
}

void loop() {
    delay(10000);
}

