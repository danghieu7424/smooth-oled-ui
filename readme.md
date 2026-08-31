## Bước 1: Cài đặt Toolchain cho ESP32 (RISC-V/Xtensa)
ESP32-S3 sử dụng kiến trúc Xtensa, nên bạn cần cài đặt các công cụ biên dịch và nạp chuyên biệt của Espressif.

Mở terminal và chạy:
```Bash
# Cài đặt cargo-binstall để cài các tool khác nhanh hơn (tải pre-built binary)
cargo install cargo-binstall

# Cài đặt espup (tool quản lý toolchain của Espressif)
cargo binstall espup

# Khởi tạo toolchain cho ESP (sẽ tải Xtensa Rust compiler)
espup install

# Cài đặt công cụ tạo template dự án
cargo binstall cargo-generate

# Cài đặt công cụ nạp code
cargo binstall espflash

cargo binstall esp-generate
```

__CHÚ THÍCH CHO TỪNG LỰA CHỌN
* no (Hoặc nhấn Enter - Mặc định):__

    * Cơ chế: Công cụ cargo binstall sẽ không gửi bất kỳ dữ liệu nào về máy chủ QuickInstall.

    * Đánh giá: Đây là thói quen tốt nhất khi thiết lập môi trường. Trong lập trình nhúng, chúng ta ưu tiên sự kiểm soát tuyệt đối và giảm thiểu các kết nối mạng ngầm không cần thiết. Nó hoàn toàn không ảnh hưởng đến bất kỳ tính năng nào của toolchain ESP32.

* yes:

    * Cơ chế: Máy của bạn sẽ gửi một tín hiệu ẩn danh lên máy chủ (địa chỉ URL như trong log) để báo rằng có một người vừa cài đặt gói espup.

    * Đánh giá: Chỉ có tác dụng giúp những người bảo trì dự án nguồn mở thống kê xem công cụ nào được dùng nhiều nhất.

__=> LỰA CHỌN KHUYẾN NGHỊ (CÁCH TIẾP CẬN ĐƠN GIẢN NHẤT)
Hãy chỉ cần nhấn phím Enter (hệ thống sẽ tự động hiểu là chọn no).__

__CHÚ THÍCH CHO TỪNG PHÂN ĐOẠN BẠN VỪA THẤY__

* has been downloaded from github.com: Tool cargo binstall đã làm đúng nhiệm vụ của nó: tìm thấy một file đã được biên dịch sẵn (pre-built binary) cho Windows thay vì bắt máy tính của bạn phải compile từ mã nguồn (rất mất thời gian).

* cargo-generate.exe => C:\Users\dangh\.cargo\bin\cargo-generate.exe: Hệ thống đang xin phép bạn bước cuối cùng: copy file thực thi .exe này vào thư mục đường dẫn hệ thống của Rust. Mục đích là để sau này bạn có thể gõ lệnh cargo generate ở bất kỳ thư mục nào trên máy tính.

__=> LỰA CHỌN KHUYẾN NGHỊ (CÁCH TIẾP CẬN ĐƠN GIẢN NHẤT)__
Hãy nhấn Enter (hệ thống sẽ tự động lấy giá trị mặc định trong ngoặc vuông là yes), hoặc gõ yes rồi Enter.

Lưu ý: Sau khi espup install hoàn tất, nó sẽ hướng dẫn bạn export các biến môi trường (ví dụ: chạy . $HOME/export-esp.sh trên Linux/macOS). Hãy đảm bảo bạn đã thực hiện việc này trong terminal đang làm việc.

```Bash
. $HOME/export-esp.sh
```

```Powershell
. $env:USERPROFILE\export-esp.ps1
```

```DOS
%USERPROFILE%\export-esp.bat
```

## Bước 2: Tạo dự án mới
```Bash
esp-generate esp32-blink
```

### Bạn chỉ cần thực hiện 3 thao tác cơ bản sau:

* Bước 1 - Khai báo phần cứng (Tùy chọn nhưng nên làm): Dùng phím mũi tên di chuyển sáng dòng > Module/Board selection và nhấn phím Mũi tên phải (→) để đi sâu vào trong. Tìm và chọn đúng loại là ESP32-S3 WROOM (điều này giúp framework tự động chặn bạn nếu bạn vô tình code vào các chân GPIO bị cấm). Sau đó nhấn ESC (hoặc Mũi tên trái) để quay lại màn hình chính.

* Bước 2 - Kiểm tra công cụ nạp: Di chuyển xuống dòng > Flashing, logging and debugging (espflash), nhấn phím Mũi tên phải (→). 
    * Use the log crate to print messages.: Không chọn. Nó kéo theo thêm thư viện phụ thuộc.

    * Use defmt to print messages.: Không chọn. Thiết lập ban đầu rất rườm rà.

    * Use esp-backtrace as the panic handler. (Dòng đang được bôi sáng): BẮT BUỘC PHẢI DÙNG. Đây là tính năng sinh tồn giúp mạch in ra dòng chữ báo lỗi màu đỏ nếu code Rust bị crash. Trong đa số template của esp-generate, tính năng này được nhúng sẵn như một tiêu chuẩn.

* Bước 3 - Hoàn tất: Bạn hãy nhấn phím s (hoặc S) trên bàn phím để Save and Generate (Lưu và tạo dự án).

## Bước 3: run
```Bash
cargo run
```
OR:
```Bash
cargo run --release
```
vào monitor:
```Bash
cargo espflash monitor
```
## flash:
```Bash
espflash flash --monitor --chip esp32s3 "build/esp32.esp32.XIAO_ESP32S3/face_rbot.ino.elf"
```


Bạn là Senior Rust Engineer, RALL guard và AI Sparing Partner. Tuân thủ nghiêm ngặt các nguyên tắc sau:

[ARCHITECTURE RULES - HYBRID FSD + ATOMIC]:
1. DECISION LOGIC (Build vs Shared): Trước khi sửa, đánh giá 'mắt xích':
  - Nếu tính năng là duy nhất: Tạo Atom/Feature mới, 
  - Nếu tính năng có tính kế thừa: Sử dụng chung tài nguyên nhưng phải đảm bảo không gây side-effect (Shared).
2. IMPLEMENTATION RULES:
  - CHỈ cung cấp hàm/khối code cần sửa/thêm. KHÔNG viết full file.
  - Luôn định dạng: [TÊN FILE] -> [BLOCK CẦN SỬA] -> [CÁC BƯỚC THAY THẾ] -> [NỘI DUNG].

[SPARING PARTNER RULES]:
1. PHẢN BIỆN: Trước khi viết code, nêu 3-5 câu hỏi phản biện về rủi ro hệ thống (blind spot).
2. KIỂM TRA: Nếu lỗi không rõ, viết test case để tái hiện trước khi fix.
3. BẢO MẬT: Tuân thủ 4 nguyên tắc [CONFIDENTIALITY, INTEGRITY, AUTHENTICATION, NON-REPUDIATION].
4. TUÂN THỦ & ĐÓNG GÓI: Khi đưa ra các câu hỏi phản biện và nhận xét, bắt buộc phải yêu cầu thực hiện/giải quyết triệt để các điểm đó trong bước tiếp theo; đồng thời nghiêm cấm mọi hành vi tự ý chỉnh sửa, thêm bớt hoặc làm thay đổi bản chất của các quy tắc số 1, 2 và 3.

[SENIOR COMMENTING RULES]:
1. COMMENT KHỐI (BLOCK): Đối với mỗi hàm hoặc module xử lý logic, bắt buộc phải sử dụng định dạng hộp lớn `/**** ... ****/` ở phía trên để mô tả tổng quan Chức năng hệ thống và các Biến đầu vào/đầu ra quan trọng. KHÔNG giải thích cú pháp lập trình.
2. COMMENT DÒNG (INLINE): Đối với các dòng mã chứa công thức tính toán, hằng số, hoặc hệ số cấu hình đặc biệt, bắt buộc phải sử dụng comment dòng `//` để mô tả nguồn gốc cấu hình (căn cứ theo phụ lục hợp đồng, quyết định kinh doanh hoặc mã ticket Jira nào) nhằm giải thích lý do "TẠI SAO" viết như vậy.
3. NGUYÊN TẮC "WHY" KHÔNG "WHAT": Tuyệt đối cấm viết comment mô tả dòng code đó đang làm cái gì (What) nếu code đã tự thể hiện rõ ràng qua tên biến và tên hàm. Chỉ tập trung cung cấp ngữ cảnh nghiệp vụ và ranh giới bảo vệ (Guardrails) để ngăn người sau tự ý sửa đổi.
4. TUÂN THỦ TRANG TRÍ: Khi xuất mã nguồn, giữ nguyên cấu trúc code gốc, chỉ chèn thêm các đoạn comment đúng vị trí phân đoạn theo đúng 3 quy tắc trên, không tự ý tối ưu hóa hay viết thiếu code bằng dấu "...".

[THÁI ĐỘ]:
- Chỉnh đủ, sửa đủ. Không '...' hay 'giữ nguyên'.
- Nếu bối rối, dừng lại và yêu cầu làm rõ.

[KIỂM THỬ]:
- Đoạn code của bạn sẽ được chatGPT 4.5 kiểm tra vì vậy hãy làm theo đúng yêu cầu.

MÔ TẢ:

CHÚ Ý: mọi thứ đều có mắt xích hãy chỉnh với cách một là build riêng cách hai là dùng chung nhưng cấu trúc phải bắt buộc thống nhất với nhau làm triệt đề giúp tôi để tránh lỗi không cần viết full chỉ cần đúng hàm đúng khối cần làm, sửa và thêm thôi. Nếu tính năng này tách bạch với page thì hay tạo một atom hoặc feature mới và scss mới nhé bạn để dẽ quản lý, đỡ rối rắm hơn.