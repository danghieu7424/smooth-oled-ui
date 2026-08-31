import serial
import cv2
import numpy as np

# Cấu hình cổng COM
PORT = 'COM14'
BAUD = 921600

try:
    ser = serial.Serial(PORT, BAUD, timeout=1)
except Exception as e:
    print(f"Không thể mở cổng {PORT}: {e}")
    exit(1)

out = cv2.VideoWriter('oled_capture.mp4', cv2.VideoWriter_fourcc(*'mp4v'), 60, (512, 256))

print(f"Đang kết nối {PORT}...")
print("Bấm phím 'q' trên cửa sổ video để dừng và lưu file MP4.")

buffer = b''
try:
    while True:
        b = ser.read(1)
        if not b:
            continue
        buffer += b
        if len(buffer) > 4:
            buffer = buffer[1:]
            
        if buffer == b'\xfe\xfe\xfe\xfe':
            data = ser.read(1024)
            if len(data) == 1024:
                frame = np.zeros((64, 128), dtype=np.uint8)
                data_arr = np.frombuffer(data, dtype=np.uint8).reshape((8, 128))
                for page in range(8):
                    for bit in range(8):
                        # Trích xuất bit từ LSB tới MSB của mỗi byte U8g2
                        frame[page*8 + bit, :] = ((data_arr[page] >> bit) & 1) * 255
                        
                frame_scaled = cv2.resize(frame, (512, 256), interpolation=cv2.INTER_NEAREST)
                frame_bgr = cv2.cvtColor(frame_scaled, cv2.COLOR_GRAY2BGR)
                
                cv2.imshow("OLED Render (Press Q to exit)", frame_bgr)
                out.write(frame_bgr)
                
                if cv2.waitKey(1) & 0xFF == ord('q'):
                    break
except KeyboardInterrupt:
    pass
except Exception as e:
    print("Lỗi:", e)

ser.close()
out.release()
cv2.destroyAllWindows()
print("Đã lưu video oled_capture.mp4!")
