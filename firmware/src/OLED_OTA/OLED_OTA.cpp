#include "OLED_OTA.h"

OLED_OTA::OLED_OTA(const char* projectId, const char* projectToken, const char* currentVersion) {
    _projectId = String(projectId);
    _projectToken = String(projectToken);
    _currentVersion = String(currentVersion);
    _lastCheckAttempt = 0;
}

void OLED_OTA::setApiEndpoint(const char* host, uint16_t port) {
    _apiHost = String(host);
    _apiPort = port;
}

void OLED_OTA::begin() {
    uint8_t mac[6];
    WiFi.macAddress(mac);
    char macStr[18];
    sprintf(macStr, "%02X%02X%02X%02X%02X%02X", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    _deviceId = String(macStr);

    Serial.println("\n[OLED_OTA] Khởi tạo OTA Service (HTTP Polling)");
    Serial.println("[OLED_OTA] Device ID: " + _deviceId);
    Serial.println("[OLED_OTA] Phiên bản hiện tại: " + _currentVersion);
    
    _lastCheckAttempt = millis() - 60000; 

    const esp_partition_t* running = esp_ota_get_running_partition();
    const esp_partition_t* next_update = esp_ota_get_next_update_partition(NULL);
    
    Serial.printf("[OLED_OTA] Đang chạy: %s (0x%08X, %u KB)\n", 
                  running ? running->label : "NULL", 
                  running ? running->address : 0,
                  running ? running->size / 1024 : 0);
    Serial.printf("[OLED_OTA] OTA Target: %s (0x%08X, %u KB)\n", 
                  next_update ? next_update->label : "NULL", 
                  next_update ? next_update->address : 0,
                  next_update ? next_update->size / 1024 : 0);
}

void OLED_OTA::loop() {
    if (WiFi.status() == WL_CONNECTED) {
        unsigned long now = millis();
        if (now - _lastCheckAttempt > 60000) {
            _lastCheckAttempt = now;
            _checkUpdate();
        }
    }
}

void OLED_OTA::_checkUpdate() {
    String protocol = (_apiPort == 443) ? "https://" : "http://";
    String url = protocol + _apiHost;
    if (_apiPort != 80 && _apiPort != 443) {
        url += ":" + String(_apiPort);
    }
    url += "/api/firmware/" + _projectId;
    
    Serial.println("[OLED_OTA] Đang kiểm tra cập nhật tại: " + url);
    _performUpdate(url, "unknown");
}

/****
 * OTA bằng esp_partition_write() trực tiếp - bypass hoàn toàn esp_ota_write()
 * Lý do: esp_ota_write() trên ESP32-S3 + Arduino Core 2.0.11 bị bug
 *        trả về ESP_OK nhưng KHÔNG ghi gì xuống Flash (toàn 0xFF)
 * 
 * Quy trình mới:
 *   1. esp_partition_erase_range() - Xóa sạch partition đích
 *   2. esp_partition_write()       - Ghi RAW từng chunk 4KB trực tiếp
 *   3. esp_partition_read()        - Đọc lại chunk đầu để verify
 *   4. esp_ota_set_boot_partition()- Đổi cờ khởi động
 ****/
void OLED_OTA::_performUpdate(String url, String newVersion) {
    if (url.indexOf("?") == -1) {
        url += "?token=" + _projectToken;
    } else {
        url += "&token=" + _projectToken;
    }

    Serial.println("[OLED_OTA] Đang tiến hành cài đặt firmware mới...");
    
    bool isHttps = url.startsWith("https");
    WiFiClient* client = nullptr;
    
    if (isHttps) {
        WiFiClientSecure* secureClient = new WiFiClientSecure();
        secureClient->setInsecure();
        client = secureClient;
    } else {
        client = new WiFiClient();
    }
    
    client->setTimeout(30);

    HTTPClient http;
    http.begin(*client, url);
    http.addHeader("x-ESP32-version", _currentVersion);
    http.setTimeout(30000);
    
    int httpCode = http.GET();
    
    if (httpCode == HTTP_CODE_NOT_MODIFIED) {
        Serial.println("[OLED_OTA] Không có bản cập nhật mới (HTTP 304).");
        http.end();
        delete client;
        return;
    }
    
    if (httpCode != HTTP_CODE_OK) {
        Serial.printf("[OLED_OTA] Lỗi kết nối HTTP: %d\n", httpCode);
        http.end();
        delete client;
        return;
    }

    int contentLength = http.getSize();
    Serial.printf("[OLED_OTA] Kích thước firmware: %d bytes\n", contentLength);
    
    if (contentLength <= 0) {
        Serial.println("[OLED_OTA] Lỗi: Server không gửi Content-Length.");
        http.end();
        delete client;
        return;
    }

    /****
     * BƯỚC 1: Xác định partition đích
     ****/
    const esp_partition_t* update_partition = esp_ota_get_next_update_partition(NULL);
    if (update_partition == NULL) {
        Serial.println("[OLED_OTA] LỖI: Không tìm thấy phân vùng OTA đích!");
        http.end();
        delete client;
        return;
    }
    
    Serial.printf("[OLED_OTA] Ghi vào partition: %s (0x%08X, %u KB)\n", 
                  update_partition->label, update_partition->address, update_partition->size / 1024);

    if ((size_t)contentLength > update_partition->size) {
        Serial.printf("[OLED_OTA] LỖI: File quá lớn! %d > %u\n", contentLength, update_partition->size);
        http.end();
        delete client;
        return;
    }

    /****
     * BƯỚC 2: Xóa sạch partition đích TRỰC TIẾP bằng esp_partition_erase_range()
     * Phải xóa toàn bộ vùng sẽ ghi (làm tròn lên bội số của 4KB sector)
     ****/
    // Làm tròn kích thước lên bội số 4096 (SPI Flash sector size)
    size_t erase_size = ((contentLength + 4095) / 4096) * 4096;
    
    Serial.printf("[OLED_OTA] Đang xóa %u bytes trên partition %s...\n", erase_size, update_partition->label);
    esp_err_t err = esp_partition_erase_range(update_partition, 0, erase_size);
    if (err != ESP_OK) {
        Serial.printf("[OLED_OTA] LỖI esp_partition_erase_range: %s (0x%x)\n", esp_err_to_name(err), err);
        http.end();
        delete client;
        return;
    }
    Serial.println("[OLED_OTA] Xóa partition thành công.");

    /****
     * BƯỚC 2.5: TEST GHI/ĐỌC FLASH TRỰC TIẾP bằng spi_flash_write()
     * Bypass hoàn toàn partition layer để xác định lỗi phần cứng hay phần mềm
     ****/
    {
        uint32_t test_addr = update_partition->address; // 0x340000
        uint8_t test_write[16] __attribute__((aligned(4))) = {0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03, 0x04,
                                                               0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C};
        uint8_t test_read[16] __attribute__((aligned(4))) = {0};
        
        Serial.printf("[OLED_OTA] === FLASH TEST tại địa chỉ vật lý 0x%08X ===\n", test_addr);
        
        // Test 1: spi_flash_write trực tiếp
        esp_err_t wr = spi_flash_write(test_addr, test_write, 16);
        Serial.printf("[OLED_OTA] spi_flash_write: %s (0x%x)\n", esp_err_to_name(wr), wr);
        
        // Test 2: spi_flash_read trực tiếp
        esp_err_t rd = spi_flash_read(test_addr, test_read, 16);
        Serial.printf("[OLED_OTA] spi_flash_read:  %s (0x%x)\n", esp_err_to_name(rd), rd);
        
        Serial.printf("[OLED_OTA] Đã ghi : ");
        for (int i = 0; i < 16; i++) Serial.printf("%02X ", test_write[i]);
        Serial.println();
        
        Serial.printf("[OLED_OTA] Đọc lại: ");
        for (int i = 0; i < 16; i++) Serial.printf("%02X ", test_read[i]);
        Serial.println();
        
        bool match = true;
        for (int i = 0; i < 16; i++) {
            if (test_write[i] != test_read[i]) { match = false; break; }
        }
        
        if (match) {
            Serial.println("[OLED_OTA] >>> SPI Flash GHI/ĐỌC KHỚP! Partition layer bị bug.");
        } else {
            Serial.println("[OLED_OTA] >>> SPI Flash GHI/ĐỌC KHÔNG KHỚP! Lỗi phần cứng Flash hoặc Write-Protected.");
            
            // Test 3: Thử esp_flash_write trên chip mặc định
            esp_err_t wr2 = esp_flash_write(esp_flash_default_chip, test_write, test_addr, 16);
            Serial.printf("[OLED_OTA] esp_flash_write: %s (0x%x)\n", esp_err_to_name(wr2), wr2);
            
            uint8_t test_read2[16] __attribute__((aligned(4))) = {0};
            esp_flash_read(esp_flash_default_chip, test_read2, test_addr, 16);
            Serial.printf("[OLED_OTA] esp_flash_read:  ");
            for (int i = 0; i < 16; i++) Serial.printf("%02X ", test_read2[i]);
            Serial.println();
            
            // Test 4: Kiểm tra flash encryption
            Serial.printf("[OLED_OTA] Partition encrypted flag: %s\n", update_partition->encrypted ? "YES" : "NO");
        }
        
        Serial.println("[OLED_OTA] === KẾT THÚC FLASH TEST ===");
        
        // Xóa lại partition sau khi test
        esp_partition_erase_range(update_partition, 0, 4096);
    }

    /****
     * BƯỚC 3: Đọc dữ liệu từ HTTP và GHI TRỰC TIẾP bằng esp_partition_write()
     * Mỗi chunk đều kiểm tra lỗi. Chunk đầu tiên được đọc lại verify ngay.
     ****/
    Serial.println("[OLED_OTA] OTA Bắt đầu ghi Flash...");
    WiFiClient *tcp = http.getStreamPtr();
    
    // Buffer căn chỉnh 32-bit cho ESP32 Flash SPI yêu cầu alignment
    static uint8_t buf[4096] __attribute__((aligned(4)));
    static uint8_t verify_buf[16] __attribute__((aligned(4)));
    
    size_t written = 0;
    int lastPercent = -1;
    unsigned long lastDataTime = millis();
    bool writeError = false;

    while (written < (size_t)contentLength) {
        size_t remaining = (size_t)contentLength - written;
        size_t toRead = (remaining > sizeof(buf)) ? sizeof(buf) : remaining;
        
        size_t bytesRead = tcp->readBytes(buf, toRead);
        
        if (bytesRead == 0) {
            if (millis() - lastDataTime > 30000) {
                Serial.printf("\n[OLED_OTA] Timeout! Đã ghi: %u/%d bytes\n", written, contentLength);
                writeError = true;
                break;
            }
            delay(10);
            continue;
        }
        
        lastDataTime = millis();
        
        // In 16 byte đầu tiên
        if (written == 0) {
            Serial.printf("[OLED_OTA] First 16 bytes: ");
            for (int i = 0; i < (bytesRead > 16 ? 16 : (int)bytesRead); i++) {
                Serial.printf("%02X ", buf[i]);
            }
            Serial.println();
            
            if (buf[0] != 0xE9) {
                Serial.printf("[OLED_OTA] LỖI: Magic byte = 0x%02X (phải là 0xE9)!\n", buf[0]);
                writeError = true;
                break;
            }
        }

        // Pad chunk cuối cùng lên bội số 4 byte (yêu cầu của esp_partition_write)
        size_t writeLen = bytesRead;
        if (writeLen % 4 != 0) {
            size_t padded = ((writeLen + 3) / 4) * 4;
            // Điền 0xFF vào phần pad
            for (size_t i = writeLen; i < padded; i++) {
                buf[i] = 0xFF;
            }
            writeLen = padded;
        }

        // GHI TRỰC TIẾP XUỐNG FLASH
        err = esp_partition_write(update_partition, written, buf, writeLen);
        if (err != ESP_OK) {
            Serial.printf("\n[OLED_OTA] LỖI esp_partition_write offset=%u len=%u: %s (0x%x)\n", 
                          written, writeLen, esp_err_to_name(err), err);
            writeError = true;
            break;
        }
        
        // VERIFY NGAY CHUNK ĐẦU TIÊN - đọc lại từ Flash để chắc chắn
        if (written == 0) {
            err = esp_partition_read(update_partition, 0, verify_buf, 16);
            if (err != ESP_OK) {
                Serial.printf("[OLED_OTA] LỖI đọc lại verify: %s\n", esp_err_to_name(err));
                writeError = true;
                break;
            }
            Serial.printf("[OLED_OTA] Verify chunk đầu: ");
            for (int i = 0; i < 16; i++) {
                Serial.printf("%02X ", verify_buf[i]);
            }
            Serial.println();
            
            if (verify_buf[0] != 0xE9) {
                Serial.println("[OLED_OTA] LỖI NGHIÊM TRỌNG: Ghi Flash thành công nhưng đọc lại sai!");
                Serial.println("[OLED_OTA] Flash có thể bị hỏng phần cứng hoặc Write-Protected.");
                writeError = true;
                break;
            }
            Serial.println("[OLED_OTA] Verify chunk đầu OK! Flash ghi được.");
        }
        
        written += bytesRead;
        
        int percent = (written * 100) / contentLength;
        if (percent - lastPercent >= 5 || percent == 100) {
            Serial.printf("[OLED_OTA] Tiến trình: %d%% (%u/%d)\n", percent, written, contentLength);
            lastPercent = percent;
        }
    }
    
    http.end();
    
    if (writeError || written != (size_t)contentLength) {
        Serial.printf("[OLED_OTA] Tải/ghi thất bại. Đã ghi: %u/%d\n", written, contentLength);
        delete client;
        return;
    }

    /****
     * BƯỚC 4: Verify 16 byte cuối cùng trên Flash
     ****/
    Serial.println("[OLED_OTA] Đã ghi đủ. Verify 16 byte đầu trên Flash...");
    err = esp_partition_read(update_partition, 0, verify_buf, 16);
    Serial.printf("[OLED_OTA] Flash byte đầu: ");
    for (int i = 0; i < 16; i++) {
        Serial.printf("%02X ", verify_buf[i]);
    }
    Serial.println();

    /****
     * BƯỚC 5: Đổi cờ khởi động sang partition mới
     * Không dùng esp_ota_end() vì chúng ta không dùng esp_ota_begin()
     ****/
    Serial.println("[OLED_OTA] Đang đổi Boot partition...");
    err = esp_ota_set_boot_partition(update_partition);
    if (err != ESP_OK) {
        Serial.printf("[OLED_OTA] LỖI esp_ota_set_boot_partition: %s (0x%x)\n", esp_err_to_name(err), err);
        delete client;
        return;
    }
    
    Serial.println("[OLED_OTA] OTA Hoàn tất! Đang khởi động lại...");
    delete client;
    delay(1000);
    ESP.restart();
}
