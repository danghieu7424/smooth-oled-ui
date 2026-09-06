@echo off
setlocal
cd /d "%~dp0"

set ARDUINO_DIRECTORIES_DATA=D:\.AppData\Arduino15
set CLI_PATH="C:\Users\dangh\AppData\Local\Programs\Arduino IDE\resources\app\lib\backend\resources\arduino-cli.exe"
set BUILD_DIR=.\build\esp32.esp32.XIAO_ESP32S3

echo [INFO] Bien dich Flash Diagnostic Tool...
"%CLI_PATH:"=%" compile --fqbn esp32:esp32:esp32s3:FlashMode=qio,FlashSize=16M,PartitionScheme=app3M_fat9M_16MB,PSRAM=disabled,CDCOnBoot=cdc --build-path "%BUILD_DIR%" flash_diag.ino

if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Bien dich that bai!
    exit /b %ERRORLEVEL%
)

set ELF_FILE=%BUILD_DIR%\flash_diag.ino.elf
echo [INFO] Nap Flash Diagnostic vao chip...
espflash flash -M -B 115200 --chip esp32s3 --erase-parts otadata --partition-table "%BUILD_DIR%\flash_diag.ino.partitions.bin" "%ELF_FILE%"

endlocal
