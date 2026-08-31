# [CHÚ THÍCH] Khai báo các biến cục bộ
$cli_path = "C:\Users\dangh\AppData\Local\Programs\Arduino IDE\resources\app\lib\backend\resources\arduino-cli.exe"
$build_dir = ".\build\esp32.esp32.XIAO_ESP32S3"
$elf_file = "$build_dir\face_rbot.ino.elf"

# [CHÚ THÍCH] ĐÃ LOẠI BỎ KHỐI LỆNH XÓA THƯ MỤC ĐỂ BẢO TOÀN CACHE

# [CHÚ THÍCH] BƯỚC 1 (MỚI): BIÊN DỊCH TĂNG DẦN (INCREMENTAL COMPILE)
# Kịch bản giờ đây chỉ biên dịch lại những file có sự thay đổi. Thời gian sẽ giảm từ vài phút xuống vài giây.
Write-Host "[INFO] Bắt đầu biên dịch (Incremental Compile)..." -ForegroundColor Cyan
& $cli_path compile --fqbn esp32:esp32:XIAO_ESP32S3 --build-path $build_dir face_rbot.ino

# [CHÚ THÍCH] BƯỚC 2: KIỂM TRA LỖI (FAIL-FAST)
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Quá trình biên dịch thất bại! Dừng việc nạp Flash." -ForegroundColor Red
    exit $LASTEXITCODE
}

# [CHÚ THÍCH] BƯỚC 3: NẠP FLASH VÀ MỞ MONITOR
Write-Host "[INFO] Biên dịch thành công. Bắt đầu nạp Flash qua espflash..." -ForegroundColor Green
espflash flash --chip esp32s3 $elf_file