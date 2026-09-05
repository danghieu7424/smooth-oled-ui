@echo off
setlocal

cd /d "%~dp0"
set CLI_PATH="C:\Users\dangh\AppData\Local\Programs\Arduino IDE\resources\app\lib\backend\resources\arduino-cli.exe"
set BUILD_DIR=%TEMP%\build_firmware

echo [INFO] Xoa thu muc build cu de tranh loi Permission denied...
if exist "%BUILD_DIR%" rmdir /s /q "%BUILD_DIR%"

echo [INFO] Bien dich Firmware...
"%CLI_PATH:"=%" compile --fqbn esp32:esp32:XIAO_ESP32S3 --build-path "%BUILD_DIR%" firmware.ino

if errorlevel 1 (
    echo [ERROR] Bien dich that bai!
    exit /b 1
)

set ELF_FILE=%BUILD_DIR%\firmware.ino.elf
echo [INFO] Nap Firmware vao chip...
espflash flash -M -B 115200 --chip esp32s3 --erase-parts otadata --partition-table "%BUILD_DIR%\firmware.ino.partitions.bin" "%ELF_FILE%"

endlocal
