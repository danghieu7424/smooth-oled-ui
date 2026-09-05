import urllib.request, time
req = urllib.request.urlopen('http://127.0.0.1:7424/api/firmware/007Rlq30Q2vU-esp32-tool?token=fc11b225f325609bb7309ad70f090a78')
print("Headers:", req.headers)
try:
    total = 0
    while True:
        chunk = req.read(4096)
        if not chunk: break
        total += len(chunk)
        if total % (4096 * 10) == 0:
            print(f"Downloaded {total} bytes...")
        time.sleep(0.01)
    print("Done. Total:", total)
except Exception as e:
    print("Failed:", e)
