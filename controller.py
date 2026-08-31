import serial
import keyboard
import sys
import time

try:
    # Sử dụng COM14 và tốc độ 921600 giống lúc flash
    ser = serial.Serial('COM14', 921600, timeout=0.1)
except Exception as e:
    print(f"Lỗi mở cổng COM14: {e}")
    sys.exit(1)

print("Đã kết nối thành công với ESP32 qua COM14!")
print("=============================================")
print(" BỘ ĐIỀU KHIỂN BẰNG BÀN PHÍM (BLUETOOTH PC)  ")
print("=============================================")
print("[Lưu ý] Script này chạy nền, bạn có thể click ra ngoài cửa sổ")
print(" - Mũi tên TRÁI / LÊN    : Lên (Up / Prev)")
print(" - Mũi tên PHẢI / XUỐNG  : Xuống (Down / Next)")
print(" - Phím HOME             : Mở Side List")
print(" - Phím WINDOWS (Win)    : Mở Popup List")
print(" - Phím ESC              : Đóng các Menu")
print(" - Phím ENTER            : Chọn (Dự phòng)")
print("\nNhấn Ctrl+C trong cửa sổ này để thoát chương trình.")

# Biến cờ chống dội phím (debounce) thủ công
last_press = 0

def send_char(c):
    global last_press
    now = time.time()
    if now - last_press > 0.15: # Chống dội 150ms
        try:
            ser.write(c.encode())
            last_press = now
        except:
            pass

# Khai báo sự kiện bắt phím
keyboard.on_press_key('left', lambda _: send_char('U'))
keyboard.on_press_key('up', lambda _: send_char('U'))
keyboard.on_press_key('right', lambda _: send_char('D'))
keyboard.on_press_key('down', lambda _: send_char('D'))
keyboard.on_press_key('home', lambda _: send_char('S'))
keyboard.on_press_key('left windows', lambda _: send_char('P'))
keyboard.on_press_key('right windows', lambda _: send_char('P'))
keyboard.on_press_key('esc', lambda _: send_char('C'))
keyboard.on_press_key('enter', lambda _: send_char('E'))

try:
    # Chạy vòng lặp vô tận giữ cho script sống
    while True:
        time.sleep(0.1)
except KeyboardInterrupt:
    print("\nĐã đóng cổng COM và ngắt kết nối.")
    ser.close()
