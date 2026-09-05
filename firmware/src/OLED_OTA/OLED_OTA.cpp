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

    Serial.println("\n[OLED_OTA] Khởi tạo OTA Service (ESP-IDF Direct)");
    Serial.println("[OLED_OTA] Device ID: " + _deviceId);
    Serial.println("[OLED_OTA] Phiên bản hiện tại: " + _currentVersion);
    
    // Kích hoạt check ngay lập tức khi boot
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
 * OTA bằng ESP-IDF API trực tiếp — bypass hoàn toàn Arduino Update.h
 * 
 * Lý do: Update.h dùng ESP.partitionWrite() có bug "silent fail" trên ESP32-S3:
 *   - esp_partition_write() trả ESP_OK nhưng dữ liệu không ghi xuống Flash
 *   - _verifyEnd() đọc lại thấy magic byte vẫn 0xFF → "Flash Read Failed"
 * 
 * Quy trình ESP-IDF OTA:
 *   1. esp_ota_begin()  — mở handle, erase partition đích
 *   2. esp_ota_write()  — ghi từng chunk 4KB (có verify nội bộ)
 *   3. esp_ota_end()    — validate image header + checksum
 *   4. esp_ota_set_boot_partition() — chuyển boot sang partition mới
 ****/
void OLED_OTA::_performUpdate(String url, String newVersion) {
    if (url.indexOf("?") == -1) {
        url += "?token=" + _projectToken;
    } else {
        url += "&token=" + _projectToken;
    }

    Serial.println("[OLED_OTA] Đang tiến hành cài đặt firmware mới...");
    
    /****
     * BƯỚC 1: Thiết lập kết nối HTTP
     * Dùng WiFiClient trên STACK (không new/delete) để tránh crash pbuf_free
     * khi destructor WiFiClient giải phóng pbuf đã bị lwIP thu hồi
     ****/
    bool isHttps = url.startsWith("https");
    WiFiClient tcpClient;
    WiFiClientSecure secureClient;
    
    WiFiClient* client;
    if (isHttps) {
        secureClient.setInsecure();
        client = &secureClient;
    } else {
        client = &tcpClient;
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
        return;
    }
    
    if (httpCode != HTTP_CODE_OK) {
        Serial.printf("[OLED_OTA] Lỗi kết nối HTTP: %d\n", httpCode);
        http.end();
        return;
    }

    int contentLength = http.getSize();
    Serial.printf("[OLED_OTA] Kích thước firmware: %d bytes\n", contentLength);
    
    if (contentLength <= 0) {
        Serial.println("[OLED_OTA] Lỗi: Server không gửi Content-Length.");
        http.end();
        return;
    }

    /****
     * BƯỚC 2: Mở OTA handle bằng ESP-IDF API
     * esp_ota_begin() sẽ tự erase toàn bộ partition đích
     * OTA_SIZE_UNKNOWN cho phép ghi không cần biết trước kích thước chính xác
     ****/
    const esp_partition_t* update_partition = esp_ota_get_next_update_partition(NULL);
    if (!update_partition) {
        Serial.println("[OLED_OTA] Lỗi: Không tìm thấy OTA partition.");
        http.end();
        return;
    }
    
    Serial.printf("[OLED_OTA] Ghi vào partition: %s (0x%08X, %u KB)\n",
                  update_partition->label,
                  update_partition->address,
                  update_partition->size / 1024);

    esp_ota_handle_t ota_handle = 0;
    esp_err_t err = esp_ota_begin(update_partition, OTA_SIZE_UNKNOWN, &ota_handle);
    if (err != ESP_OK) {
        Serial.printf("[OLED_OTA] esp_ota_begin() thất bại: %s (0x%x)\n", esp_err_to_name(err), err);
        http.end();
        return;
    }

    /****
     * BƯỚC 3: Đọc stream HTTP và ghi từng chunk 4KB bằng esp_ota_write()
     * esp_ota_write() ghi trực tiếp qua SPI Flash driver, không qua lớp
     * "tối ưu skip 0xFF" của Arduino Update.h
     ****/
    WiFiClient* tcp = http.getStreamPtr();
    
    // Buffer 4KB — bằng đúng 1 sector SPI Flash
    const size_t CHUNK_SIZE = 4096;
    uint8_t* buf = (uint8_t*)malloc(CHUNK_SIZE);
    if (!buf) {
        Serial.println("[OLED_OTA] Lỗi: Không đủ RAM cho buffer OTA.");
        esp_ota_abort(ota_handle);
        http.end();
        return;
    }
    
    size_t totalWritten = 0;
    bool writeError = false;
    
    while (totalWritten < (size_t)contentLength) {
        // Tính toán số byte cần đọc trong chunk này
        size_t bytesToRead = CHUNK_SIZE;
        if (totalWritten + bytesToRead > (size_t)contentLength) {
            bytesToRead = (size_t)contentLength - totalWritten;
        }
        
        // Đọc từ stream với retry
        size_t bytesRead = 0;
        int retries = 0;
        while (bytesRead < bytesToRead && retries < 300) {
            size_t got = tcp->readBytes(buf + bytesRead, bytesToRead - bytesRead);
            if (got == 0) {
                retries++;
                delay(100); // 100ms * 300 = 30s timeout
            } else {
                retries = 0;
                bytesRead += got;
            }
        }
        
        if (bytesRead != bytesToRead) {
            Serial.printf("[OLED_OTA] Stream timeout: đọc %u/%u bytes\n", bytesRead, bytesToRead);
            writeError = true;
            break;
        }
        
        // Ghi chunk xuống Flash bằng ESP-IDF OTA API
        err = esp_ota_write(ota_handle, buf, bytesRead);
        if (err != ESP_OK) {
            Serial.printf("[OLED_OTA] esp_ota_write() thất bại tại offset %u: %s\n", 
                          totalWritten, esp_err_to_name(err));
            writeError = true;
            break;
        }
        
        totalWritten += bytesRead;
        
        // Log tiến trình mỗi 100KB
        if (totalWritten % (100 * 1024) < CHUNK_SIZE) {
            Serial.printf("[OLED_OTA] Đã ghi: %u / %d bytes (%u%%)\n", 
                          totalWritten, contentLength,
                          (unsigned)(totalWritten * 100 / contentLength));
        }
    }
    
    free(buf);
    http.end();
    
    if (writeError || totalWritten != (size_t)contentLength) {
        Serial.printf("[OLED_OTA] Ghi thất bại: %u/%d bytes\n", totalWritten, contentLength);
        esp_ota_abort(ota_handle);
        return;
    }

    /****
     * BƯỚC 4: Kết thúc OTA — validate image header + checksum
     * esp_ota_end() kiểm tra:
     *   - Magic byte 0xE9 tại offset 0
     *   - Image header CRC
     *   - Segment checksums
     ****/
    err = esp_ota_end(ota_handle);
    if (err != ESP_OK) {
        Serial.printf("[OLED_OTA] esp_ota_end() thất bại: %s (0x%x)\n", esp_err_to_name(err), err);
        return;
    }

    /****
     * BƯỚC 5: Chuyển boot partition sang firmware mới
     * Sau restart, bootloader sẽ load từ partition mới
     ****/
    err = esp_ota_set_boot_partition(update_partition);
    if (err != ESP_OK) {
        Serial.printf("[OLED_OTA] esp_ota_set_boot_partition() thất bại: %s\n", esp_err_to_name(err));
        return;
    }

    Serial.printf("[OLED_OTA] OTA Hoàn tất! Đã ghi %u bytes. Đang khởi động lại...\n", totalWritten);
    delay(1000);
    ESP.restart();
}
