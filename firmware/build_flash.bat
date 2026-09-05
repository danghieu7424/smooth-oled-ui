@echo off
setlocal
cd /d "%~dp0"

REM [CHÚ THÍCH] Khai báo các biến cục bộ trỏ đến công cụ và thư mục dự án
set CLI_PATH="C:\Users\dangh\AppData\Local\Programs\Arduino IDE\resources\app\lib\backend\resources\arduino-cli.exe"
set BUILD_DIR=.\build\esp32.esp32.XIAO_ESP32S3
set ELF_FILE=%BUILD_DIR%\firmware.ino.elf

REM [CHÚ THÍCH] ĐÃ LOẠI BỎ LỆNH RMDIR ĐỂ GIỮ LẠI CACHE (GIỐNG ARDUINO IDE)

REM [CHÚ THÍCH] BƯỚC 1: BIÊN DỊCH TĂNG DẦN (INCREMENTAL COMPILE)
REM arduino-cli sẽ tự kiểm tra và chỉ dịch lại những gì bạn vừa sửa đổi.
echo [INFO] Bat dau bien dich nhanh (Incremental Compile)...
"%CLI_PATH:"=%" compile --fqbn esp32:esp32:XIAO_ESP32S3:PartitionScheme=default_8MB --build-path "%BUILD_DIR%" firmware.ino

REM [CHÚ THÍCH] BƯỚC 2: KIỂM TRA LỖI (FAIL-FAST)
REM Nếu quá trình build có lỗi C++, biến %ERRORLEVEL% sẽ khác 0. Kịch bản sẽ dừng ngay lập tức.
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Qua trinh bien dich that bai! Dung viec nap Flash.
    exit /b %ERRORLEVEL%
)

REM [CHÚ THÍCH] BƯỚC 3: NẠP FLASH VÀ MỞ MONITOR
echo [INFO] Bien dich thanh cong. Bat dau nap Flash qua espflash...
espflash flash --chip esp32s3 --partition-table "%BUILD_DIR%\firmware.ino.partitions.bin" "%ELF_FILE%"

endlocal
