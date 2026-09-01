import serial, time
s = serial.Serial('COM14', 921600, timeout=1)
s.setDTR(False)
s.setRTS(True)
time.sleep(0.1)
s.setDTR(True)
s.setRTS(False)
time.sleep(0.1)
s.setRTS(True)
print('READING...')
end = time.time() + 10
while time.time() < end:
    data = s.read(1000).decode('utf-8', 'ignore')
    for line in data.splitlines():
        if '[' in line:
            print(line.strip())
