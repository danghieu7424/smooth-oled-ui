const fs = require('fs');
const content = fs.readFileSync('firmware.ino', 'utf8');
const lines = content.split('\n');

/****
 * Bảng sửa lỗi: Dòng -> nội dung comment đúng
 * CHỈ thay thế phần comment (sau //), KHÔNG đụng code C++
 ****/
const fixes = {
  // Comment-only lines
  120: '// --- KHAI BÁO CÁC HÀM XỬ LÝ SỰ KIỆN (CALLBACKS) ---',
  197: 'int current_brightness = 20; // Độ sáng hiện tại',
  208: '// Trạng thái kết nối WiFi',
  214: '// --- CÀI ĐẶT CÁC HÀM XỬ LÝ SỰ KIỆN ---',
  390: '  // 3. Khởi động UI',
  538: '        // 2. Logic Đồng hồ (Tick)',
  546: '        // 4. Delay để giữ 60FPS',
  588: '        // --- 2. XỬ LÝ QUÉT WIFI BẤT ĐỒNG BỘ ---',
  636: '        // --- 3. XỬ LÝ KẾT NỐI WIFI ---',
  655: '        // --- 5. ĐỒNG BỘ THỜI GIAN QUA API ---',
};

// Lines where comment is inline (after code), only replace the comment portion
const inlineFixes = {
  240: { after: '; //', comment: ' // Lệnh phần cứng đổi độ sáng OLED trực tiếp' },
  256: { full: '  // Chặn mở mật khẩu nếu đang Scanning, không có mạng, hoặc lỗi' },
  258: { full: '      // Nếu mạng này đang được kết nối rồi, báo luôn không cần nhập pass' },
  264: { full: '      // KIỂM TRA: Nếu mạng này TRÙNG với mạng đã lưu trong AT24C256' },
  269: { full: '              // Nếu không khớp SSID thì không dùng pwd này' },
  273: { full: '      // Mở ô nhập Pass và ĐIỀN SẴN mật khẩu cũ (như thẻ input type="text" có value)' },
  274: { full: '      // Người dùng chỉ cần ấn Enter để kết nối, hoặc ấn xóa để sửa' },
  289: { after: 'return; //', comment: ' // Đang quét thì không kích hoạt lại' },
  292: { full: '  // XÓA WiFi.disconnect() ở đây để không làm rớt mạng đang kết nối khi load lại menu' },
  294: { after: 'true); //', comment: ' // Quét bất đồng bộ (Async)' },
  304: { full: '  // [MỚI] Lưu Credentials vào AT24C256' },
  309: { full: '      // Ngắt kết nối cũ (nếu có)' },
  317: { full: '  // [MỚI] Cập nhật trạng thái Connecting... lên màn hình' },
  336: { full: '  // --- Khởi tạo và kiểm tra RTC & EEPROM (Core 1 sẽ dùng I2C1 / Wire1) ---' },
  337: { full: '  // Khởi tạo Bus 1 (Sensor) ở đây để các module gọi begin() thành công' },
  381: { full: '  // 1. Gán mảng dữ liệu vào thư viện UI' },
  393: { full: '  // 4. Cấu hình TimeSync' },
  405: { full: '  // Tạo Mutex cho các biến dùng chung' },
  408: { full: '  // Task UI trên Core 0' },
  419: { full: '  // Task Network trên Core 1' },
  432: { full: '    // Xóa task loop của Arduino để giải phóng tài nguyên' },
  441: { full: '        // 1. Xử lý Input từ Serial (Nút bấm mô phỏng)' },
  543: { full: '        // 3. Vẽ lên màn hình OLED (Render)' },
  558: { full: '        // --- 1. CẬP NHẬT TRẠNG THÁI WIFI ---' },
};

let count = 0;

for (const [lineNum, newContent] of Object.entries(fixes)) {
  const idx = parseInt(lineNum) - 1;
  if (idx < lines.length) {
    // Get leading whitespace from original line
    const origLine = lines[idx].replace('\r', '');
    const leadingWS = origLine.match(/^(\s*)/)[1];
    const fixedContent = newContent.trimStart();
    lines[idx] = leadingWS + fixedContent + '\r';
    count++;
  }
}

for (const [lineNum, fix] of Object.entries(inlineFixes)) {
  const idx = parseInt(lineNum) - 1;
  if (idx < lines.length) {
    const origLine = lines[idx].replace('\r', '');

    if (fix.full) {
      // Replace entire line but keep leading whitespace
      const leadingWS = origLine.match(/^(\s*)/)[1];
      lines[idx] = leadingWS + fix.full.trimStart() + '\r';
      count++;
    } else if (fix.after && fix.comment) {
      // Find the code portion, keep it, replace comment
      const commentIdx = origLine.indexOf('//');
      if (commentIdx !== -1) {
        const codePart = origLine.substring(0, commentIdx);
        lines[idx] = codePart + '//' + fix.comment.replace(/^\/\/\s?/, '') + '\r';
        count++;
      }
    }
  }
}

fs.writeFileSync('firmware.ino', lines.join('\n'), 'utf8');
console.log('Fixed ' + count + ' lines');
