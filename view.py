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
print(" - Mũi tên TRÁI / PHẢI  : Trái (Left) / Phải (Right)")
print(" - Mũi tên LÊN / XUỐNG   : Lên (Up) / Xuống (Down)")
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

def on_key(event):
    if event.event_type == 'down':
        name = event.name
        if name == 'left': send_char('\x1BL')
        elif name == 'up': send_char('\x1BU')
        elif name == 'right': send_char('\x1BR')
        elif name == 'down': send_char('\x1BD')
        elif name == 'home': send_char('\x1BS')
        elif name == 'end': send_char('\x1BP')
        elif name == 'esc': send_char('\x1BC')
        elif name == 'backspace': send_char('\x1BB')
        elif name == 'enter': send_char('\x1BE')
        elif name == 'space': send_char(' ')
        elif len(name) == 1 and name.isprintable():
            send_char(name)

keyboard.hook(on_key)
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
