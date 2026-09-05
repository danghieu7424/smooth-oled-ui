me:[OLED_OTA] Lỗi OTA (3): Flash Read Failed
[OLED_OTA] Đang kiểm tra cập nhật tại: http://192.168.7.7:7424/api/firmware/007Rlq30Q2vU-esp32-tool
[OLED_OTA] Đang tiến hành cài đặt firmware mới...
[OLED_OTA] OTA Bắt đầu...
[OLED_OTA] Tiến trình: 100%
[OLED_OTA] Đã tải đủ toàn bộ file.

[OLED_OTA] Lỗi OTA (3): Flash Read Failed lỗi này là sao

AI:
Lỗi **`Lỗi OTA (3): Flash Read Failed`** xuất hiện tại bước `Update.end()`, sau khi ESP32 đã tải đủ 100% dung lượng file.

Mã lỗi số **`3`** trong thư viện `UpdateClass` của ESP32 Arduino chính là hằng số:

```cpp
#define UPDATE_ERROR_READ (3) // Trả về text: "Flash Read Failed"

```

---

[HW-ELEC]

### Nguyên nhân cốt lõi

Lỗi này **không phải do mạng hay do file truyền thiếu**. Tại thời điểm hoàn tất ghi Flash qua `Update.end()`, ESP32 gọi hàm nội bộ `esp_ota_end()` trong ESP-IDF. Hàm này sẽ đọc ngược lại dữ liệu vừa ghi trên phân vùng Flash OTA mục tiêu để kiểm tra tính toàn vẹn (MD5 hash / SHA256 / Magic byte header của file nhị phân).

Lỗi `Flash Read Failed` kích hoạt khi:

1. **Ghi đè lên chính phân vùng đang chạy (Partition Conflict):**
Trong `OLED_OTA.cpp` có đoạn:


```cpp
esp_ota_set_boot_partition(running);

```


Nếu phân vùng Partition Table của bạn không cấu hình Dual OTA (`ota_0` và `ota_1`), hoặc chỉ có một phân vùng duy nhất (như `factory`), hàm `esp_ota_get_next_update_partition(NULL)` sẽ trỏ vào chính phân vùng đang thực thi. Khi đó hệ thống từ chối hoặc việc đọc lại bị xung đột bộ nhớ Flash MMU (I-Cache).


2. **Dung lượng file vượt quá kích thước Partition:**
Nếu binary nạp lên lớn hơn kích thước partition OTA được chỉ định trong bảng phân vùng (`partitions.csv`), phần cuối ghi đè ra ngoài ranh giới vùng nhớ, dẫn đến việc đọc đối soát thất bại.
3. **Xung đột bus SPI Flash hoặc sụt áp đột ngột (Brownout):**
Sau khi nạp liên tục 100% dữ liệu, dòng tải ghi/đọc Flash kéo dài có thể gây sụt áp nếu tụ đệm nguồn 3.3V gần chân chip không đủ lớn ($<100\,\mu\text{F}$), khiến Flash SPI trả về dữ liệu rác khi ESP-IDF đọc lại.



---

[HW-ELEC]

### Cách xử lý triệt để

#### 1. Kiểm tra lại Partition Scheme trong IDE / PlatformIO

ESP32 bắt buộc phải có ít nhất 2 phân vùng OTA (`app0/ota_0` và `app1/ota_1`) để hoán đổi.

* Nếu dùng **Arduino IDE**: Vào **Tools** $\rightarrow$ **Partition Scheme** $\rightarrow$ Chọn **"Default 4MB with spiffs (1.2MB APP/1.5MB SPIFFS)"** hoặc **"Minimal SPIFFS (1.9MB APP with OTA)"**. Tuyệt đối không để *"No OTA"*.
* Nếu dùng **PlatformIO**, kiểm tra file `partitions.csv` phải có dạng:
```csv
# Name,   Type, SubType, Offset,  Size, Flags
nvs,      data, nvs,     0x9000,  0x5000,
otadata,  data, ota,     0xe000,  0x2000,
app0,     app,  ota_0,   0x10000, 0x140000,
app1,     app,  ota_1,   0x150000,0x140000,
spiffs,   data, spiffs,  0x290000,0x160000,

```



#### 2. Xóa bỏ can thiệp cưỡng bức boot partition trong `OLED_OTA.cpp`

Đoạn code can thiệp `esp_ota_set_boot_partition(running)` trong `OLED_OTA.cpp` dòng 29–40 có thể gây sai lệch trạng thái con trỏ phân vùng OTA của bootloader:

Thay thế toàn bộ hàm `begin()` trong `OLED_OTA.cpp`:

```cpp
void OLED_OTA::begin() {
    uint8_t mac[6];
    WiFi.macAddress(mac);
    char macStr[18];
    sprintf(macStr, "%02X%02X%02X%02X%02X%02X", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    _deviceId = String(macStr);

    Serial.println("\n[OLED_OTA] Khởi tạo OTA Service (HTTP Polling)");
    Serial.println("[OLED_OTA] Device ID: " + _deviceId);
    Serial.println("[OLED_OTA] Phiên bản hiện tại: " + _currentVersion);
    
    // Check update immediately on boot
    _lastCheckAttempt = millis() - 60000; 

    // Kiểm tra trạng thái phân vùng OTA hiện tại để chẩn đoán
    const esp_partition_t* running = esp_ota_get_running_partition();
    const esp_partition_t* next = esp_ota_get_next_update_partition(NULL);
    Serial.printf("[OLED_OTA] Đang chạy tại partition: %s (Offset: 0x%08X, Size: %u KB)\n", 
                  running ? running->label : "NULL", 
                  running ? running->address : 0,
                  running ? running->size / 1024 : 0);
                  
    if (next) {
        Serial.printf("[OLED_OTA] Phân vùng OTA tiếp theo: %s (Offset: 0x%08X, Size: %u KB)\n", 
                      next->label, next->address, next->size / 1024);
    } else {
        Serial.println("[OLED_OTA] CẢNH BÁO: Không tìm thấy phân vùng OTA kế tiếp! Kiểm tra Partition Scheme.");
    }
}

```

Kiểm tra log nạp: nếu `Phân vùng OTA tiếp theo` báo `NULL` hoặc trùng tên/địa chỉ với `Đang chạy tại partition`, nguyên nhân chính là do bảng phân vùng hiện tại chỉ có 1 vùng app duy nhất.

me: tôi dùng cái này [text](firmware/build_flash.bat) [text](firmware/build_flash.ps1) [text](firmware/firmware.ino)

AI:Nguyên nhân trực tiếp gây ra lỗi **`Flash Read Failed`** nằm ngay ở tham số build trong file build script của bạn:

```cmd
compile --fqbn esp32:esp32:XIAO_ESP32S3:PartitionScheme=default_8MB
```[cite: 5, 6]

---

[HW-ELEC]
### Bản chất lỗi từ cấu hình PartitionScheme
Trong gói bo mạch ESP32 của Arduino (`esp32:esp32`), sơ đồ phân vùng mặc định **`default_8MB`** có bảng phân vùng như sau:
* `nvs` (20KB)
* `otadata` (8KB)
* `app0` / `factory` (**3MB**)
* `spiffs` (~4.8MB)
* **KHÔNG hề có phân vùng `app1` (ota_1)**.

Khi thư viện `UpdateClass` (`Update.begin()` và `Update.write()`) thực thi[cite: 2]:
1. Nó gọi hàm ESP-IDF `esp_ota_get_next_update_partition(NULL)`.
2. Do bảng phân vùng `default_8MB` chỉ có duy nhất **1 phân vùng app** (`app0`), hàm này buộc phải trỏ vào chính phân vùng đang chạy mã lệnh.
3. ESP32 ghi đè dữ liệu OTA đè thẳng lên vùng nhớ Flash mà CPU đang đọc để chạy chương trình.
4. Đến bước `Update.end()`[cite: 2], ESP-IDF kiểm tra lại tính toàn vẹn (read check / hash verify) thì bộ nhớ đệm cache (MMU) bị lỗi do phân vùng thực thi đã bị can thiệp, kích hoạt mã lỗi số **3: `UPDATE_ERROR_READ` (Flash Read Failed)**[cite: 1, 2].

---

### Giải pháp khắc phục

Đổi sơ đồ phân vùng sang sơ đồ có **Dual OTA** (2 phân vùng app song song để hoán đổi an toàn). 

Với chip **XIAO ESP32-S3 (Flash 8MB)**, Arduino Core hỗ trợ sẵn scheme **`8MB with fat/ota`** hoặc **`ota_8MB`** (mỗi phân vùng app khoảng 3.3MB).

Sửa lại lệnh biên dịch trong script `.bat` và `.ps1`[cite: 5, 6]:

#### 1. Sửa file script build (`.bat`)[cite: 5]
Thay thế dòng `compile`[cite: 5]:
```cmd
"%CLI_PATH:"=%" compile --fqbn esp32:esp32:XIAO_ESP32S3:PartitionScheme=ota_8MB --build-path "%BUILD_DIR%" firmware.ino

```

*(Nếu core bản cũ của bạn không nhận `ota_8MB`, dùng Scheme: `PartitionScheme=fat_8MB`)*.

#### 2. Sửa file script PowerShell (`.ps1`)



Thay thế dòng `compile`:

```powershell
& $cli_path compile --fqbn esp32:esp32:XIAO_ESP32S3:PartitionScheme=ota_8MB --build-path $build_dir firmware.ino

```

---

### Lưu ý quan trọng khi nạp lần đầu sau khi đổi Partition

Do bảng phân vùng thay đổi kích thước và địa chỉ:

1. Bạn cần cắm cáp Type-C và chạy lệnh nạp qua dây bằng `espflash` một lần để ghi lại Bảng phân vùng (`partitions.bin`) mới vào chip.


2. Lệnh trong file script của bạn đã có sẵn cờ xóa `otadata`:


```cmd
espflash flash --chip esp32s3 --erase-parts otadata --partition-table "%BUILD_DIR%\firmware.ino.partitions.bin" "%ELF_FILE%"
```[cite: 5]
Lệnh này sẽ đưa trạng thái OTA về ban đầu (`ota_0`).

```


3. Sau khi nạp dây xong bản mới, các lần cập nhật OTA tiếp theo qua Wi-Fi sẽ tự động luân phiên giữa `ota_0` và `ota_1` mà không còn bị lỗi xung đột đọc Flash.