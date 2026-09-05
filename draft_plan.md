# kế hoạch: tạo server cung cấp bản update firmware cho esp.

# triển khai: 

- người dùng đăng nhập bằng tài khoản Google khi đăng nhập lưu các thông tin cơ bản về người dùng.
- mỗi người có thể tải nhiều dự án. mỗi dự án có nhiều phiên bản. mỗi dự án có một mã id để khi esp truy cập api kiểm tra xem có phải phiên bản mới không sẽ bắt tay và tải về rồi cập nhật.
- esp tạo một thư viện cho server với name.begin("ID").

# mô hình dự án:

```Plaintext
storages/
├── user_id_01/
│   ├── project_id_01/
│   │   ├── version_firmware_v1.0.0
│   │   ├── version_firmware_v1.0.1
│   │   └── ...
│   ├── project_id_02/
│   │   ├── version_firmware_v1.0.0
│   │   ├── version_firmware_v2.0.0
│   │   └── ...
│   └── ...
├── user_id_02/
│   ├── project_id_03/
│   │   ├── version_firmware_v1.0.0
│   │   ├── version_firmware_v1.1.0
│   │   └── ...
│   ├── project_id_04/
│   │   ├── version_firmware_v0.1.0
│   │   ├── version_firmware_v0.2.0
│   │   └── ...
│   └── ...
└── ...
```

```graph TD 

    Root["storages"]
    
    %% Nhánh User
    U1["user_id_1"]
    U2["user_id_2"]
    U_More["... (user_id_n)"]
    
    Root --> U1
    Root --> U2
    Root -.-> U_More
    
    %% Nhánh Projects của user_id_1
    P1["project_id_1"]
    P2["project_id_2"]
    P_More1["... (project_id_n)"]
    
    U1 --> P1
    U1 --> P2
    U1 -.-> P_More1
    
    %% Nhánh Projects của user_id_2
    P3["project_id_3"]
    P4["project_id_4"]
    P_More2["... (project_id_n)"]
    
    U2 --> P3
    U2 --> P4
    U2 -.-> P_More2
    
    %% Nhánh Firmware của project_id_1
    V1["version_firmware_v1.0"]
    V2["version_firmware_v1.1"]
    V_More1["... (version_firmware_n)"]
    
    P1 --> V1
    P1 --> V2
    P1 -.-> V_More1

    %% Nhánh Firmware của project_id_2
    V3["version_firmware_v2.0"]
    V4["version_firmware_v2.1"]
    V_More2["... (version_firmware_n)"]
    
    P2 --> V3
    P2 --> V4
    P2 -.-> V_More2
```

# ui

- trang như firebase với các dự án kèm theo biểu đồ tổng thể của cả dự án để xem số lượng cập nhật của các thiết bị yêu cầu.
- bên trong dự án cũng hiện tổng thể và thêm phần upload firmware mới. mỗi firmware có thể xem các thiết bị đã yêu cầu cập nhật của riêng bản đó.
- có thể xóa các bản cũ nếu người dùng yêu cầu.

# server & data

do mô hình khá nhỏ tôi phân vân có nên dùng các cơ sở dữ liệu bên ngoài không hoặc dùng chính utils trong server của tôi cũng khá ổn.

> **[Gợi ý từ AI (Antigravity)] Về Cơ sở dữ liệu:**
> Tránh dùng file JSON (utils) thuần túy vì khi có nhiều ESP truy cập đồng thời (concurrency) để check update, ghi/đọc JSON thủ công rất dễ lỗi file hoặc thắt nút cổ chai. 
> **Đề xuất cực mạnh:** Dùng **SQLite**. Nó là database nhưng nằm gọn trong 1 file tĩnh duy nhất (như JSON), không cần cài đặt server (không cần MySQL/MongoDB), tốc độ siêu nhanh và quản lý quan hệ User -> Project -> Version cực kỳ chuẩn chỉ.

>> ok đấy nhưng tôi không muốn dùng ứng dụng thứ 3 vì tôi khi build chỉ muốn có 1 file chạy duy nhất.

> **[Câu hỏi phản biện từ AI] Về Bảo mật Firmware:**
> Hiện tại ESP chỉ dùng `project_id` để tải firmware. Như vậy ai vô tình biết `project_id` cũng có thể lấy cắp file `.bin` của bạn hoặc spam request làm sập server. Bạn có muốn thêm cơ chế **Tạo Token/API Key** riêng cho từng thiết bị, hoặc cho phép gán danh sách địa chỉ MAC hợp lệ không?

>> cái này ok đấy thư viện trong firmware/src cũng thêm một thư viện để khi khởi tạo chỉ cần nhập id dự án và token để xác nhận dự án.
>> cơ chế token cho 1 project và cho copy ở dưới id trong dashboarth

> **[Câu hỏi phản biện từ AI] Về Phiên bản (Versioning):**
> ESP32 sẽ so sánh phiên bản như thế nào? Chúng ta nên quy định dùng *Semantic Versioning* (ví dụ: `1.0.1`) kết hợp chuẩn thư viện `HTTPUpdate` (Gửi version hiện tại qua Header, server trả về file `.bin` nếu có bản cao hơn, hoặc trả về mã `304 Not Modified` nếu đã mới nhất). Bạn thấy cơ chế này ổn chứ?

>> ok nhưng làm thế nào cũng được miên là tuân thủ quy tắc các cong ty lớn họ làm ví dụ như tplink, xiaomi họ có phần kiểm tra phiên bản cập nhật bên trong các router

# yêu cầu thêm

## xóa toàn bộ các phần không liên quan và giữ lại các phần như:

- backend:
  - utils
  - tính năng cờ, chạy ngầm, logs
  - xóa các rowter của dự án cũ
- frontend
  - global state
  - light-dark
  - các atom đã tạo trong shared

# MQTT Broker (Khuyến nghị số 1 cho IoT)

- Cách hoạt động: ESP giữ 1 kết nối TCP nhẹ qua MQTT Broker và subscribe vào topic devices/{device_id}/ota. Khi upload firmware mới lên Web Server (Axum), server publish payload JSON chứa URL tải file và mã SHA-256 vào topic này.

- Ưu điểm: Độ trễ tính bằng mili-giây, cực nhẹ (heartbeat Keep-Alive chỉ 2 bytes), hỗ trợ hàng trăm nghìn thiết bị đồng thời.















