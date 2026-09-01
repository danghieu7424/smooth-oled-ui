import serial
import cv2
import numpy as np
import time
import keyboard

PORT = 'COM14'
BAUD = 921600

try:
    ser = serial.Serial(PORT, BAUD, timeout=1)
except Exception as e:
    print(f"Không thể mở cổng {PORT}: {e}")
    exit(1)

print(f"Đang kết nối {PORT}...")
print("=============================================")
print(" ĐÃ TÍCH HỢP ĐIỀU KHIỂN BÀN PHÍM BLUETOOTH   ")
print("=============================================")
print(" - Mũi tên TRÁI / LÊN    : Lên (Up / Prev)")
print(" - Mũi tên PHẢI / XUỐNG  : Xuống (Down / Next)")
print(" - Phím HOME             : Mở Side List")
print(" - Phím END              : Mở Popup List")
print(" - Phím ESC              : Đóng các Menu")
print(" - Phím BACKSPACE        : Xóa ký tự (khi nhập pass)")
print(" - Phím ENTER            : Chọn / Xác nhận")
print("Bạn có thể tắt cửa sổ video bằng nút X trên thanh tiêu đề.")

# --- Thiết lập bắt phím ngầm (Global Hook) ---
last_press = 0
def send_char(c):
    global last_press
    now = time.time()
    if now - last_press > 0.15:
        try:
            ser.write(c.encode())
            last_press = now
        except:
            pass

keyboard.on_press_key('left', lambda _: send_char('U'))
keyboard.on_press_key('up', lambda _: send_char('U'))
keyboard.on_press_key('right', lambda _: send_char('D'))
keyboard.on_press_key('down', lambda _: send_char('D'))
keyboard.on_press_key('home', lambda _: send_char('S'))
keyboard.on_press_key('end', lambda _: send_char('P'))
keyboard.on_press_key('esc', lambda _: send_char('C'))
keyboard.on_press_key('backspace', lambda _: send_char('B'))
keyboard.on_press_key('enter', lambda _: send_char('E'))
# ---------------------------------------------

def read_exact(ser, size):
    buf = b''
    while len(buf) < size:
        data = ser.read(size - len(buf))
        if not data:
            return buf
        buf += data
    return buf

try:
    buffer = b''
    synced = False
    
    while True:
        if not synced:
            b = ser.read(1)
            if not b: continue
            buffer += b
            if len(buffer) > 4:
                buffer = buffer[1:]
            if buffer == b'\xfe\xfe\xfe\xfe':
                synced = True
                buffer = b''
        else:
            # Đọc 1024 bytes dữ liệu
            data = read_exact(ser, 1024)
            if len(data) != 1024:
                synced = False
                continue
                
            # Đọc 4 bytes tiếp theo xem có phải Header không
            next_header = read_exact(ser, 4)
            
            if next_header == b'\xfe\xfe\xfe\xfe':
                # Khung ảnh hợp lệ
                frame = np.zeros((64, 128), dtype=np.uint8)
                data_arr = np.frombuffer(data, dtype=np.uint8).reshape((8, 128))
                for page in range(8):
                    for bit in range(8):
                        frame[page*8 + bit, :] = ((data_arr[page] >> bit) & 1) * 255
                        
                frame_scaled = cv2.resize(frame, (512, 256), interpolation=cv2.INTER_NEAREST)
                frame_bgr = cv2.cvtColor(frame_scaled, cv2.COLOR_GRAY2BGR)
                
                cv2.imshow("OLED Viewer", frame_bgr)
                
                # Cho phép thoát bằng cách nhấn nút X của cửa sổ
                if cv2.waitKey(1) & 0xFF == ord('q') or cv2.getWindowProperty("OLED Viewer", cv2.WND_PROP_VISIBLE) < 1:
                    break
            else:
                buffer = next_header
                synced = False

except KeyboardInterrupt:
    pass
except Exception as e:
    print("Lỗi:", e)

ser.close()
cv2.destroyAllWindows()
