param (
    [string]$Version = "",
    [string]$Notes = "Cập nhật qua OTA"
)

$ErrorActionPreference = "Stop"

# 1. Thông tin cấu hình (Sửa lại cho phù hợp nếu cần)
$PROJECT_ID = "007Rlq30Q2vU-esp32-tool"
$API_HOST = "192.168.7.7"
$API_PORT = 7424
$CLI_PATH = "C:\Users\dangh\AppData\Local\Programs\Arduino IDE\resources\app\lib\backend\resources\arduino-cli.exe"
$BUILD_DIR = "$env:TEMP\build_firmware"
$FIRMWARE_INO = ".\firmware.ino"

Write-Host "======================================" -ForegroundColor Cyan
Write-Host " OTA PUBLISH TOOL" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan

# Nếu không nhập version, tự động đọc từ firmware.ino
if ([string]::IsNullOrWhiteSpace($Version)) {
    Write-Host "[INFO] Đang tìm phiên bản trong firmware.ino..."
    $content = Get-Content $FIRMWARE_INO
    $versionLine = $content | Select-String 'const char\* CURRENT_VERSION = "(.*?)";'
    if ($versionLine) {
        $Version = $versionLine.Matches.Groups[1].Value
        Write-Host "[INFO] Tìm thấy Version: $Version" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] Không thể đọc phiên bản từ firmware.ino!" -ForegroundColor Red
        exit 1
    }
}

# 2. Biên dịch firmware
Write-Host "`n[1/3] Biên dịch Firmware..." -ForegroundColor Yellow
if (Test-Path $BUILD_DIR) {
    Remove-Item -Recurse -Force $BUILD_DIR
}

& $CLI_PATH compile --fqbn esp32:esp32:XIAO_ESP32S3 --build-path $BUILD_DIR $FIRMWARE_INO
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Biên dịch thất bại!" -ForegroundColor Red
    exit $LASTEXITCODE
}

$BIN_FILE = "$BUILD_DIR\firmware.ino.bin"
if (-Not (Test-Path $BIN_FILE)) {
    Write-Host "[ERROR] Không tìm thấy file BIN sau khi biên dịch!" -ForegroundColor Red
    exit 1
}

# 3. Đẩy lên Server
Write-Host "`n[2/3] Tải firmware ($BIN_FILE) lên Server..." -ForegroundColor Yellow
$URI = "http://${API_HOST}:${API_PORT}/api/projects/$PROJECT_ID/firmware"
Write-Host "URL: $URI"

try {
    # Tạo Form Data theo chuẩn multipart/form-data
    $Form = @{
        version = $Version
        notes = $Notes
        file = Get-Item -Path $BIN_FILE
    }
    
    $response = Invoke-WebRequest -Uri $URI -Method Post -Form $Form -UseBasicParsing
    
    if ($response.StatusCode -eq 200) {
        Write-Host "`n[3/3] PUBLISH THÀNH CÔNG!" -ForegroundColor Green
        Write-Host "Phiên bản $Version đã sẵn sàng trên OTA Server." -ForegroundColor Green
    } else {
        Write-Host "`n[ERROR] Server trả về mã lỗi: $($response.StatusCode)" -ForegroundColor Red
    }
} catch {
    Write-Host "`n[ERROR] Tải lên thất bại!" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    if ($_.Exception.Response) {
        $status = $_.Exception.Response.StatusCode
        Write-Host "HTTP Status: $status"
        if ($status -eq 403) {
            Write-Host "=> Bị từ chối (403 Forbidden). Bạn cần thêm Cookie xác thực vào tập lệnh này." -ForegroundColor Red
        }
    }
}

Write-Host "`nHoàn thành." -ForegroundColor Cyan
