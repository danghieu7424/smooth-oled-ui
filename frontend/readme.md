# chuẩn bị môi trường (có thì bỏ qua)

1. [Rust](https://rust-lang.org/tools/install/)

2. WebAssembly target:
```Bash
rustup target add wasm32-unknown-unknown
```

3. Cài Trunk (Công cụ build & hot reload)
```Bash
cargo install --locked trunk
```

4. Cài SASS/SCSS Compiler (Trunk cần cái này để dịch file .scss):
- Windows (dùng Chocolatey): `choco install sass` kiểm tra: `sass --version`
    - cài choco với terminal Administrator
```Bash
Set-ExecutionPolicy Bypass -Scope Process -Force; `
[System.Net.ServicePointManager]::SecurityProtocol = `
[System.Net.ServicePointManager]::SecurityProtocol -bor 3072; `
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
```
- macOS (Homebrew): `brew install sass/sass/sass`
- Linux: `sudo apt install sass` hoặc cài qua npm `npm install -g sass`

# Method
HTTP Method,Hàm trong Rust (gloo_net),Ý nghĩa
GET,"Request::get(""url"")",Lấy dữ liệu
POST,"Request::post(""url"")",Tạo mới dữ liệu (Gửi kèm Body)
PUT,"Request::put(""url"")",Cập nhật toàn bộ (Ghi đè)
PATCH,"Request::patch(""url"")",Cập nhật một phần
DELETE,"Request::delete(""url"")",Xóa dữ liệu
HEAD,"Request::head(""url"")","Chỉ lấy Header, không lấy Body"
Khác,"Request::new(""url"").method(...)",Các method tùy chỉnh