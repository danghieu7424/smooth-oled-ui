#line 1 "D:\\all_projects\\rust\\rust\\display_oled\\SmoothOLED.cpp"
#include "SmoothOLED.h"

static bool s_advance_on_next_char = false;

SmoothOLED::SmoothOLED(U8G2* u8g2, Stream* serial) {
    _u8g2 = u8g2;
    _serial = serial;
    _app_state = STATE_CAROUSEL;
    _prev_app_state = STATE_CAROUSEL;
    _overlay_state = OVERLAY_NONE;
    _overlay_anim = PHASE_IDLE;
    _carousel_title = "< MAIN MENU >";
    _side_slide_x = 128.0f;
    _list_cam_target_idx = 0;
    _list_cam_y = 0.0f;
    _last_switch = 0;
    _last_tick = 0;
    _auto_demo = false;
    _pc_viewer_enabled = true;

    // --- Biến Carousel ---
    _carousel_items = nullptr;
    _carousel_count = 0;
    _current_index = 0;
    _cam_x = 0.0f;
    _target_cam_x = 0.0f;

    // --- Biến Popup List ---
    _popup_items = nullptr;
    _popup_count = 0;
    _current_list_selection = 0;
    _cursor_y = (float)(ITEM_START_Y - 8);
    _cursor_w = 56.0f;
    _target_cursor_y = (float)(ITEM_START_Y - 8);
    _target_cursor_w = 56.0f;

    _marquee_offset = 0;
    _marquee_last_time = 0;
    _marquee_delay = 30;
    _last_popup_idx = -1;

    // --- Biến Side Popup ---
    _side_items = nullptr;
    _side_count = 0;
    _side_selected_idx = 0;
    _side_parent_x = 32.0f;
    _side_slide_x = 128.0f;
    _side_list_cam_y = 0.0f;
    _side_cursor_w = 0.0f;

    _slider_val = 0.0f;
    _target_slider_val = 0.0f;
    _slider_max = 255;
    _slider_title = "Progress";
}

void SmoothOLED::begin() {
    _u8g2->setFont(u8g2_font_6x10_tf);
    _last_tick = millis();
    _last_switch = millis();
}

// Bảng ký tự xếp hạng ưu tiên (chữ thường -> số -> chữ hoa -> ký tự đặc biệt)
const char CHAR_MAP[] = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ!@#$%^&*()_+-=[]{}|;':\",./<>? ";
const int CHAR_MAP_LEN = sizeof(CHAR_MAP) - 1;

int getCharMapIndex(char c) {
    if (c == '\0') return 0;
    for (int i = 0; i < CHAR_MAP_LEN; i++) {
        if (CHAR_MAP[i] == c) return i;
    }
    return 0; // Mặc định là 'a'
}

// =================================================================================
// DATA SETUP
// =================================================================================

void SmoothOLED::setCarouselItems(const MenuItem* items, int count, const char* title) {
    _carousel_items = items;
    _carousel_count = count;
    _carousel_title = title;
    _current_index = 0;
    _target_cursor_w = 0;
    _cursor_w = 0;
}

void SmoothOLED::setPopupListItems(const char** items, int count) {
    _popup_items = items;
    _popup_count = count;
}

void SmoothOLED::setSidePopupItems(const MenuItem* items, int count) {
    _side_items = items;
    _side_count = count;
}

void SmoothOLED::enableAutoDemo(bool enable) {
    _auto_demo = enable;
}

void SmoothOLED::enablePCViewer(bool enable) {
    _pc_viewer_enabled = enable;
}

// =================================================================================
// INPUT API
// =================================================================================

void SmoothOLED::up() {
    if (_overlay_state == OVERLAY_SIDE_POPUP) {
        if (_side_selected_idx > 0) _side_selected_idx--;
        else _side_selected_idx = _side_count - 1;
    } else if (_app_state == STATE_POPUP && _overlay_state == OVERLAY_NONE) {
        if (_current_list_selection > 0) _current_list_selection--;
        else _current_list_selection = _popup_count - 1;
    } else if (_app_state == STATE_CAROUSEL && _overlay_state == OVERLAY_NONE) {
        if (_current_index > 0) _current_index--;
        else _current_index = _carousel_count - 1;
    } else if (_app_state == STATE_SLIDER) {
        _target_slider_val -= 5.0f;
        if (_target_slider_val < 0.0f) _target_slider_val = 0.0f;
    } else if (_app_state == STATE_TEXT_INPUT) {
        _current_char_idx++;
        if (_current_char_idx >= CHAR_MAP_LEN) _current_char_idx = 0;
        _text_buffer[_cursor_pos] = CHAR_MAP[_current_char_idx];
        s_advance_on_next_char = false;
        _last_tick = millis();
    } else if (_app_state == STATE_FULL_LIST) {
        if (_list_selected_index > 0) _list_selected_index--;
        else _list_selected_index = _list_count - 1;
    }
}

void SmoothOLED::down() {
    if (_overlay_state == OVERLAY_SIDE_POPUP) {
        _side_selected_idx++;
        if (_side_selected_idx >= _side_count) _side_selected_idx = 0;
    } else if (_app_state == STATE_CAROUSEL) {
        _current_index++;
        if (_current_index >= _carousel_count) _current_index = 0;
    } else if (_app_state == STATE_POPUP) {
        _current_list_selection++;
        if (_current_list_selection >= _popup_count) _current_list_selection = 0;
    } else if (_app_state == STATE_SLIDER) {
        _target_slider_val += 5.0f;
        if (_target_slider_val > _slider_max) _target_slider_val = _slider_max;
    } else if (_app_state == STATE_TEXT_INPUT) {
        _current_char_idx--;
        if (_current_char_idx < 0) _current_char_idx = CHAR_MAP_LEN - 1;
        _text_buffer[_cursor_pos] = CHAR_MAP[_current_char_idx];
        s_advance_on_next_char = false;
        _last_tick = millis();
    } else if (_app_state == STATE_FULL_LIST) {
        if (_list_selected_index < _list_count - 1) _list_selected_index++;
        else _list_selected_index = 0;
    }
}

void SmoothOLED::left() {
    if (_app_state == STATE_TEXT_INPUT) {
        if (_cursor_pos > 0) {
            _cursor_pos--;
            _current_char_idx = getCharMapIndex(_text_buffer[_cursor_pos]);
        }
        s_advance_on_next_char = false;
        _last_tick = millis();
    } else {
        up(); // Delegate
    }
}

void SmoothOLED::right() {
    if (_app_state == STATE_TEXT_INPUT) {
        if (_cursor_pos < sizeof(_text_buffer) - 2) {
            // Nếu đã ở cuối cùng thì tự động thêm ký tự mới
            if (_text_buffer[_cursor_pos + 1] == '\0') {
                _cursor_pos++;
                _current_char_idx = 0; // 'a'
                _text_buffer[_cursor_pos] = CHAR_MAP[0];
                _text_buffer[_cursor_pos + 1] = '\0';
            } else {
                _cursor_pos++;
                _current_char_idx = getCharMapIndex(_text_buffer[_cursor_pos]);
            }
        }
        s_advance_on_next_char = false;
        _last_tick = millis();
    } else {
        down(); // Delegate
    }
}

void SmoothOLED::openPopup() {
    if (_overlay_state == OVERLAY_NONE && (_app_state == STATE_CAROUSEL || _app_state == STATE_SLIDER || _app_state == STATE_FULL_LIST)) {
        _prev_app_state = _app_state;
        _app_state = STATE_POPUP;
        _current_list_selection = 0;
        _list_cam_target_idx = 0;
        _list_cam_y = 0.0f;
        _cursor_y = (float)(ITEM_START_Y - 9);
        _cursor_w = 10.0f;
        _marquee_offset = 0;
    }
}

void SmoothOLED::openModal(const char* title, const char* text) {
    if (_overlay_state == OVERLAY_NONE) {
        if (_app_state != STATE_MODAL) {
            _prev_app_state = _app_state;
        }
        _app_state = STATE_MODAL;
        _modal_title = title;
        _modal_text = text;
        _marquee_offset = 0;
        _marquee_delay = 30;
    }
}

void SmoothOLED::openSideList() {
    if (_overlay_state == OVERLAY_NONE && (_app_state == STATE_CAROUSEL || _app_state == STATE_POPUP || _app_state == STATE_SLIDER || _app_state == STATE_FULL_LIST || _app_state == STATE_MODAL)) {
        if (_app_state == STATE_POPUP || _app_state == STATE_MODAL) {
            _app_state = _prev_app_state; // Đóng popup/modal nếu có, trả về màn hình gốc
        }
        _overlay_state = OVERLAY_SIDE_POPUP;
        _overlay_anim = PHASE_OPENING;
        _side_arc_radius = 0.0f;
        _side_slide_x = 128.0f;
        _side_list_cam_y = 0.0f;
        _side_selected_idx = 0;
    }
}

void SmoothOLED::openSlider(const char* title, int current_val, int max_val, SliderCallback on_change) {
    if (_overlay_state == OVERLAY_NONE && (_app_state == STATE_CAROUSEL || _app_state == STATE_POPUP)) {
        _prev_app_state = _app_state;
        _app_state = STATE_SLIDER;
        _slider_title = title;
        _slider_max = max_val;
        _slider_val = (float)current_val;
        _target_slider_val = (float)current_val;
        _slider_on_change = on_change;
        _last_switch = millis();
    }
}

void SmoothOLED::openTextInput(const char* title, TextCallback on_submit, const char* initial_text) {
    if (_overlay_state == OVERLAY_NONE && (_app_state == STATE_CAROUSEL || _app_state == STATE_POPUP || _app_state == STATE_FULL_LIST)) {
        _prev_app_state = _app_state;
        _app_state = STATE_TEXT_INPUT;
        _text_input_title = title;
        _text_on_submit = on_submit;
        memset(_text_buffer, 0, sizeof(_text_buffer));
        
        if (initial_text && strlen(initial_text) > 0) {
            strncpy(_text_buffer, initial_text, sizeof(_text_buffer) - 2);
            _cursor_pos = strlen(_text_buffer); // Trỏ vào ký tự null sau chuỗi
            _text_buffer[_cursor_pos] = CHAR_MAP[0]; // Ký tự mặc định tiếp theo
            _text_buffer[_cursor_pos + 1] = '\0';
            _current_char_idx = 0;
        } else {
            _cursor_pos = 0;
            _current_char_idx = 0;
            _text_buffer[0] = CHAR_MAP[0];
        }
        s_advance_on_next_char = false;
        _last_switch = millis();
    }
}

void SmoothOLED::openFullList(const char* title, const char** items, int count, ListCallback on_select) {
    if (_overlay_state == OVERLAY_NONE && (_app_state == STATE_CAROUSEL || _app_state == STATE_POPUP)) {
        _prev_app_state = _app_state;
        _app_state = STATE_FULL_LIST;
        _list_title = title;
        _list_items = items;
        _list_count = count;
        _list_selected_index = 0;
        _list_camera_y = 0;
        _list_on_select = on_select;
        _last_switch = millis();
    }
}

void SmoothOLED::closeOverlay() {
    if (_overlay_state == OVERLAY_SIDE_POPUP) {
        _overlay_anim = PHASE_CLOSING;
    } else if (_app_state == STATE_FULL_LIST) {
        _app_state = STATE_CAROUSEL; // Trả về thẳng Carousel vì có thể prev_app_state bị ghi đè sau khi qua nhiều Modal
    } else if (_app_state == STATE_POPUP || _app_state == STATE_MODAL || _app_state == STATE_SLIDER || _app_state == STATE_TEXT_INPUT) {
        _app_state = _prev_app_state; // Trả về màn hình gốc
    }
}

bool SmoothOLED::backspace() {
    if (_app_state == STATE_TEXT_INPUT) {
        if (_cursor_pos > 0) {
            // Dịch chuỗi để xóa ký tự
            for (size_t i = _cursor_pos; i < sizeof(_text_buffer) - 1; i++) {
                _text_buffer[i] = _text_buffer[i + 1];
                if (_text_buffer[i] == '\0') break;
            }
            _cursor_pos--;
            _current_char_idx = getCharMapIndex(_text_buffer[_cursor_pos]);
            s_advance_on_next_char = false;
            return true;
        } else {
            // Hủy nhập liệu
            _app_state = _prev_app_state;
            return false; 
        }
    }
    return false;
}

void SmoothOLED::select() {
    if (_overlay_state == OVERLAY_SIDE_POPUP) {
        _overlay_anim = PHASE_CLOSING;
        if (_side_items[_side_selected_idx].on_enter) {
            _side_items[_side_selected_idx].on_enter();
        }
    } else if (_app_state == STATE_CAROUSEL) {
        _target_cursor_w = MENU_BOX_W + 10;
    } else if (_app_state == STATE_TEXT_INPUT) {
        // Đóng Text Input và khôi phục màn hình cũ TRƯỚC KHI gọi Callback
        // Nhờ vậy Callback có thể mở Popup đè lên màn hình cũ.
        _app_state = _prev_app_state;
        
        if (_text_on_submit) {
            _text_on_submit(_text_buffer);
        }
    } else if (_app_state == STATE_FULL_LIST) {
        if (_list_on_select) _list_on_select(_list_selected_index);
    } else if (_app_state == STATE_MODAL) {
        // Đóng Modal (Tương tự phím OK)
        _app_state = _prev_app_state;
    }
}

void SmoothOLED::inputChar(char c) {
    if (_app_state == STATE_TEXT_INPUT) {
        if (c >= 32 && c <= 126 && _cursor_pos < sizeof(_text_buffer) - 2) {
            // Nếu đã gõ xong 1 ký tự và tiếp tục gõ, ta đẩy con trỏ sang phải
            if (s_advance_on_next_char) {
                _cursor_pos++;
                if (_text_buffer[_cursor_pos] == '\0') {
                    _text_buffer[_cursor_pos + 1] = '\0'; // Đảm bảo chuỗi luôn kết thúc
                }
            }
            
            _text_buffer[_cursor_pos] = c;
            _current_char_idx = getCharMapIndex(c);
            
            // Đánh dấu để lần gõ phím máy tính tiếp theo sẽ đẩy con trỏ
            s_advance_on_next_char = true;
            _slider_title = "Progress";
        }
    } else if (_app_state == STATE_CLOCK) {
        _app_state = _prev_app_state;
    }
}

void SmoothOLED::flush_display() {
    _u8g2->sendBuffer();

    // Stream UART nếu có khai báo cổng Serial VÀ đang bật chế độ PC Viewer
    if (_serial && _pc_viewer_enabled) {
        uint8_t sync[] = {0xFE, 0xFE, 0xFE, 0xFE};
        _serial->write(sync, 4);
        _serial->write(_u8g2->getBufferPtr(), 1024);
    }
}

// =================================================================================
// CAROUSEL MENU ATOM
// =================================================================================

void SmoothOLED::update_physics() {
    _target_cam_x = (float)(_current_index * ITEM_SPACING);
    _cam_x += (_target_cam_x - _cam_x) * LERP_SPEED;
}

void SmoothOLED::draw_carousel_menu(int offset_x) {
    _u8g2->setDrawColor(1); // Tránh rò rỉ màu từ frame trước
    
    if (_carousel_title != nullptr) {
        int title_w = _u8g2->getStrWidth(_carousel_title);
        _u8g2->drawStr(64 - (title_w / 2) + offset_x, 10, _carousel_title);
    }

    int screen_center_x = 64 + offset_x;
    int screen_center_y = 32;

    for (int i = 0; i < _carousel_count; i++) {
        int item_real_x = screen_center_x + (i * ITEM_SPACING) - (int)_cam_x - 12; 
        int item_real_y = screen_center_y - 14;

        if (item_real_x > -30 && item_real_x < 128) {
            _u8g2->drawXBMP(item_real_x, item_real_y, 24, 24, _carousel_items[i].icon);
        }
    }

    // --- Vẽ viền khung Sci-fi (Chamfered Bracket - Bo góc nhẹ 2px, Cạnh đứng ngắn) ---
    int bx1 = 48 + offset_x;
    int bx2 = 50 + offset_x;
    int bx3 = 78 + offset_x;
    int bx4 = 80 + offset_x;

    // Khung trên
    _u8g2->drawLine(bx2, 14, bx3, 14); // Ngang trên dài hơn
    _u8g2->drawLine(bx2, 14, bx1, 16); // Góc chéo trái
    _u8g2->drawLine(bx3, 14, bx4, 16); // Góc chéo phải
    _u8g2->drawLine(bx1, 16, bx1, 18); // Dọc trái xuống
    _u8g2->drawLine(bx4, 16, bx4, 18); // Dọc phải xuống

    // Khung dưới
    _u8g2->drawLine(bx2, 45, bx3, 45); // Ngang dưới dài hơn
    _u8g2->drawLine(bx2, 45, bx1, 43); // Góc chéo trái
    _u8g2->drawLine(bx3, 45, bx4, 43); // Góc chéo phải
    _u8g2->drawLine(bx1, 43, bx1, 41); // Dọc trái lên
    _u8g2->drawLine(bx4, 43, bx4, 41); // Dọc phải lên

    if (_carousel_items && _carousel_count > 0) {
        const char* label = _carousel_items[_current_index].title;
        int str_width = _u8g2->getStrWidth(label);
        _u8g2->drawStr(screen_center_x - (str_width / 2), 58, label);
    }
}

// =================================================================================
// POPUP LIST MENU ATOM
// =================================================================================

void SmoothOLED::update_list_physics() {
    if (_current_list_selection < _list_cam_target_idx) {
        _list_cam_target_idx = _current_list_selection;
    } else if (_current_list_selection > _list_cam_target_idx + 2) {
        _list_cam_target_idx = _current_list_selection - 2;
    }
    float target_cam_y = (float)(_list_cam_target_idx * LINE_HEIGHT);
    _list_cam_y += (target_cam_y - _list_cam_y) * LIST_LERP_FACTOR;

    _target_cursor_y = (float)(ITEM_START_Y + (_current_list_selection * LINE_HEIGHT) - 9);
    int text_pixel_width = 0;
    
    if (_popup_items && _popup_count > 0) {
        text_pixel_width = _u8g2->getStrWidth(_popup_items[_current_list_selection]);
        
        if (_current_list_selection != _last_popup_idx) {
            _last_popup_idx = _current_list_selection;
            _marquee_offset = 0;
            _marquee_delay = 30; 
        }
        
        int box_max_w = MENU_BOX_W - 4;
        _target_cursor_w = (float)(text_pixel_width + 4);
        if (_target_cursor_w > box_max_w) {
            _target_cursor_w = (float)box_max_w;
        }
    }
    
    _cursor_y += (_target_cursor_y - _cursor_y) * LIST_LERP_FACTOR;
    _cursor_w += (_target_cursor_w - _cursor_w) * LIST_LERP_FACTOR;
    
    uint32_t now = millis();
    if (now - _marquee_last_time >= 30) { 
        _marquee_last_time = now;
        int max_visible_w = MENU_BOX_W - 8;
        if (text_pixel_width > max_visible_w) {
            if (_marquee_delay > 0) {
                _marquee_delay--;
            } else {
                _marquee_offset += 0.5f;
                if (_marquee_offset > (text_pixel_width - max_visible_w + 6)) { 
                    _marquee_offset = 0;
                    _marquee_delay = 45; 
                }
            }
        } else {
            _marquee_offset = 0;
        }
    }
}

void SmoothOLED::draw_popup_menu() {
    // Xóa nền đen để che đi phần Main Menu bên dưới
    _u8g2->setDrawColor(0);
    _u8g2->drawRBox(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H, 2);
    _u8g2->setDrawColor(1);

    _u8g2->drawRFrame(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H, 2);
    
    _u8g2->setFont(u8g2_font_6x10_tf);
    
    _u8g2->setClipWindow(MENU_BOX_X + 2, MENU_BOX_Y + 2, MENU_BOX_X + MENU_BOX_W - 3, MENU_BOX_Y + MENU_BOX_H - 3);
    
    int cam_y_int = (int)(_list_cam_y + 0.5f);

    for (int i = 0; i < _popup_count; i++) {
        int text_w = _u8g2->getStrWidth(_popup_items[i]);
        int y_pos = ITEM_START_Y + (i * LINE_HEIGHT) - cam_y_int;
        int max_w = MENU_BOX_W - 8;
        
        if (i == _current_list_selection && text_w > max_w) {
            _u8g2->drawStr(MENU_BOX_X + 4 - (int)_marquee_offset, y_pos, _popup_items[i]);
        } else {
            _u8g2->drawStr(MENU_BOX_X + 4, y_pos, _popup_items[i]);
            
            if (text_w > max_w && i != _current_list_selection) {
                _u8g2->setDrawColor(0);
                _u8g2->drawBox(MENU_BOX_X + MENU_BOX_W - 12, y_pos - 8, 10, 10);
                _u8g2->setDrawColor(1);
                _u8g2->drawStr(MENU_BOX_X + MENU_BOX_W - 12, y_pos, "..");
            }
        }
    }
    
    int draw_cursor_y = (int)(_cursor_y + 0.5f) - cam_y_int;
    int draw_cursor_w = (int)(_cursor_w + 0.5f);

    _u8g2->setDrawColor(2);
    _u8g2->drawBox(MENU_BOX_X + 2, draw_cursor_y, draw_cursor_w, 13);
    _u8g2->setDrawColor(1);

    // Trả lại Clip Window SAU KHI vẽ Box XOR để Box không bị tràn viền
    _u8g2->setMaxClipWindow();
}

// =================================================================================
// SIDE POPUP MENU ATOM
// =================================================================================

void SmoothOLED::update_side_physics() {
    // Khi Opening, arc tăng dần từ 0 -> TARGET_ARC. Khi Closing, arc giữ nguyên
    float target_arc = (_overlay_anim == PHASE_CLOSING) ? TARGET_ARC_RADIUS : TARGET_ARC_RADIUS;
    _side_arc_radius += (target_arc - _side_arc_radius) * SIDE_LERP_FACTOR;

    // Khi Closing, slide sang phải 128px
    float target_slide = (_overlay_anim == PHASE_CLOSING) ? 128.0f : 0.0f;
    _side_slide_x += (target_slide - _side_slide_x) * SIDE_LERP_FACTOR;

    float target_cam_y = (float)(_side_selected_idx * SIDE_LINE_SPACING);
    _side_list_cam_y += (target_cam_y - _side_list_cam_y) * SIDE_LERP_FACTOR;

    if (_side_items && _side_count > 0) {
        _u8g2->setFont(u8g2_font_6x10_tf);
        int target_w = _u8g2->getStrWidth(_side_items[_side_selected_idx].title) + 4;
        _side_cursor_w += (target_w - _side_cursor_w) * SIDE_LERP_FACTOR;
    }

    // Đóng hoàn toàn khi đã trượt ra ngoài đủ xa
    if (_overlay_anim == PHASE_CLOSING && _side_slide_x > 126.0f) {
        _overlay_state = OVERLAY_NONE;
        _overlay_anim = PHASE_IDLE;
        _last_switch = millis();
    }
}

void SmoothOLED::draw_side_list_menu() {
    int slide_offset = (int)_side_slide_x;

    // Không tự vẽ đè Icon cũ nữa, để Carousel tự dịch chuyển mượt mà làm nền
    _u8g2->setDrawColor(0);
    _u8g2->drawDisc(78 + slide_offset, 32, (int)_side_arc_radius);

    _u8g2->setDrawColor(1);
    _u8g2->drawCircle(78 + slide_offset, 32, (int)_side_arc_radius, U8G2_DRAW_UPPER_LEFT | U8G2_DRAW_LOWER_LEFT);

    _u8g2->setFont(u8g2_font_6x10_tf);
    
    int list_y_start = 37 - (int)_side_list_cam_y;

    for (int i = 0; i < _side_count; i++) {
        int item_y = list_y_start + (i * SIDE_LINE_SPACING);

        if (item_y > 0 && item_y < 74) {
            float dy = (float)(item_y - 5 - 32); 
            if (dy < 0) dy = -dy;
            
            float x_offset = 0;
            if (_side_arc_radius > dy) {
                x_offset = _side_arc_radius - sqrt(_side_arc_radius * _side_arc_radius - dy * dy);
            } else {
                x_offset = _side_arc_radius; 
            }

            int text_x = 37 + (int)x_offset + slide_offset;

            if (i == _side_selected_idx) {
                _u8g2->setDrawColor(1);
                _u8g2->drawRBox(text_x - 2, item_y - 8, (int)_side_cursor_w, 10, 2);
                
                _u8g2->setDrawColor(0);
                _u8g2->drawStr(text_x, item_y, _side_items[i].title);
            } else {
                _u8g2->setDrawColor(1);
                _u8g2->drawStr(text_x, item_y, _side_items[i].title);
            }
        }
    }
    
    _u8g2->setDrawColor(1); // Reset lại màu chuẩn cho các frame tiếp theo
}

// =================================================================================
// SLIDER PROGRESS ATOM
// =================================================================================

void SmoothOLED::update_slider_physics() {
    int old_val = (int)(_slider_val + 0.5f);
    _slider_val += (_target_slider_val - _slider_val) * 0.3f;
    int new_val = (int)(_slider_val + 0.5f);
    
    // Gọi callback khi giá trị hiển thị bị thay đổi trong quá trình trượt
    if (_slider_on_change && old_val != new_val) {
        _slider_on_change(new_val);
    }
}

void SmoothOLED::draw_slider_menu(int offset_x) {
    _u8g2->setDrawColor(1);
    
    // Vẽ tiêu đề
    _u8g2->setFont(u8g2_font_6x10_tf);
    int str_width = _u8g2->getStrWidth(_slider_title);
    _u8g2->drawStr(64 - (str_width / 2) + offset_x, 18, _slider_title);

    // Vẽ khung ngoài
    _u8g2->drawRFrame(14 + offset_x, 28, 100, 14, 3);

    // Tính toán độ dài thanh bar
    float ratio = _slider_val / (float)_slider_max;
    if (ratio < 0.0f) ratio = 0.0f;
    if (ratio > 1.0f) ratio = 1.0f;
    int fill_w = (int)(ratio * 96.0f);
    
    if (fill_w > 0) {
        // Fix lỗi thư viện U8g2 vẽ tia dài khi width nhỏ hơn 2*radius
        if (fill_w < 5) {
            _u8g2->drawBox(16 + offset_x, 30, fill_w, 10);
        } else {
            _u8g2->drawRBox(16 + offset_x, 30, fill_w, 10, 2);
        }
    }

    // Hiển thị phần trăm (%) thay vì số thô
    char val_str[10];
    int percent = (int)((_target_slider_val / (float)_slider_max) * 100.0f + 0.5f);
    snprintf(val_str, sizeof(val_str), "%d%%", percent);
    int val_w = _u8g2->getStrWidth(val_str);
    _u8g2->drawStr(64 - (val_w / 2) + offset_x, 56, val_str);
}

void SmoothOLED::draw_text_input_menu(int offset_x) {
    // Xóa nền khu vực Popup để đè lên danh sách bên dưới
    _u8g2->setDrawColor(0);
    _u8g2->drawBox(2 + offset_x, 2, 124, 44);
    
    _u8g2->setDrawColor(1);
    _u8g2->drawRFrame(2 + offset_x, 2, 124, 44, 3); // Khung viền bo góc

    // Vẽ tiêu đề
    _u8g2->setFont(u8g2_font_6x10_tf);
    int str_width = _u8g2->getStrWidth(_text_input_title);
    _u8g2->drawStr(64 - (str_width / 2) + offset_x, 13, _text_input_title);
    
    // Gạch ngang phân cách tiêu đề
    _u8g2->drawLine(2 + offset_x, 16, 126 + offset_x, 16);

    // Vẽ khung nhập liệu
    _u8g2->drawRFrame(6 + offset_x, 24, 116, 16, 2);

    // Hiển thị text đã nhập (hoặc đang nhập)
    _u8g2->setCursor(8 + offset_x, 35);
    _u8g2->print(_text_buffer);

    // Tính vị trí con trỏ hiện tại trên màn hình
    char temp[64];
    strncpy(temp, _text_buffer, _cursor_pos);
    temp[_cursor_pos] = '\0';
    int cursor_x = 8 + _u8g2->getStrWidth(temp);

    // Vẽ gạch dưới con trỏ mỏng (1px) không chạm chữ và không chạm khung
    _u8g2->drawLine(cursor_x + offset_x, 37, cursor_x + 5 + offset_x, 37);
    // Mũi tên trên chỉ Lên (2 lớp: đỉnh 2px, đáy 4px) nằm cách viền khung (y=24) 1px
    _u8g2->drawLine(cursor_x + 2 + offset_x, 21, cursor_x + 3 + offset_x, 21);
    _u8g2->drawLine(cursor_x + 1 + offset_x, 22, cursor_x + 4 + offset_x, 22);
    
    // Mũi tên dưới chỉ Xuống (2 lớp: đáy 4px, đỉnh 2px) nằm cách viền khung (y=39) 1px
    _u8g2->drawLine(cursor_x + 1 + offset_x, 41, cursor_x + 4 + offset_x, 41);
    _u8g2->drawLine(cursor_x + 2 + offset_x, 42, cursor_x + 3 + offset_x, 42);
}

void SmoothOLED::draw_full_list_menu(int offset_x) {
    // Tiêu đề
    _u8g2->setFont(u8g2_font_6x10_tf);
    _u8g2->setDrawColor(1);
    
    int title_w = _u8g2->getStrWidth(_list_title);
    _u8g2->drawStr(64 - title_w/2 + offset_x, 10, _list_title);
    _u8g2->drawLine(0 + offset_x, 13, 128 + offset_x, 13);
    
    // Physics cho camera cuộn
    float target_y = _list_selected_index * 12;
    _list_camera_y += (target_y - _list_camera_y) * 0.3f;
    
    int start_y = 28 - (int)_list_camera_y;
    
    // Vẽ danh sách
    for (int i = 0; i < _list_count; i++) {
        int y = start_y + i * 12;
        if (y > 22 && y < 70) {
            if (i == _list_selected_index) {
                // Highlight box
                _u8g2->setDrawColor(1);
                _u8g2->drawBox(4 + offset_x, y - 9, 120, 12);
                _u8g2->setDrawColor(0);
            } else {
                _u8g2->setDrawColor(1);
            }
            _u8g2->drawStr(8 + offset_x, y, _list_items[i]);
        }
    }
    _u8g2->setDrawColor(1);
}

// =================================================================================
// MODAL DIALOG ATOM
// =================================================================================

void SmoothOLED::update_modal_physics() {
    int text_pixel_width = 0;
    if (_modal_text) {
        text_pixel_width = _u8g2->getStrWidth(_modal_text);
    }
    
    uint32_t now = millis();
    if (now - _marquee_last_time >= 30) { 
        _marquee_last_time = now;
        int max_visible_w = MENU_BOX_W - 8;
        if (text_pixel_width > max_visible_w) {
            if (_marquee_delay > 0) {
                _marquee_delay--;
            } else {
                _marquee_offset += 0.5f;
                if (_marquee_offset > (text_pixel_width - max_visible_w + 6)) { 
                    _marquee_offset = 0;
                    _marquee_delay = 45; 
                }
            }
        } else {
            _marquee_offset = 0;
        }
    }
}

void SmoothOLED::draw_modal_dialog() {
    _u8g2->setDrawColor(0);
    _u8g2->drawRBox(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H, 2);
    _u8g2->setDrawColor(1);
    _u8g2->drawRFrame(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H, 2);

    int button_bar_y = MENU_BOX_Y + MENU_BOX_H - 18;
    _u8g2->drawLine(MENU_BOX_X, button_bar_y, MENU_BOX_X + MENU_BOX_W - 1, button_bar_y);
    _u8g2->drawLine(MENU_BOX_X + MENU_BOX_W / 2, button_bar_y, MENU_BOX_X + MENU_BOX_W / 2, MENU_BOX_Y + MENU_BOX_H - 1);

    _u8g2->setFont(u8g2_font_5x7_tf);
    
    // Left button
    _u8g2->drawStr(MENU_BOX_X + 11, button_bar_y + 8, "cancel");
    _u8g2->drawStr(MENU_BOX_X + 14, button_bar_y + 16, "[ESC]");

    // Right button
    _u8g2->drawStr(MENU_BOX_X + MENU_BOX_W / 2 + 20, button_bar_y + 8, "ok");
    _u8g2->drawStr(MENU_BOX_X + MENU_BOX_W / 2 + 13, button_bar_y + 16, "[ENT]");

    // Content
    _u8g2->setFont(u8g2_font_6x10_tf);
    
    _u8g2->setDrawColor(1);
    _u8g2->drawStr(MENU_BOX_X + 4, MENU_BOX_Y + 11, _modal_title);
    
    _u8g2->setClipWindow(MENU_BOX_X + 2, MENU_BOX_Y + 13, MENU_BOX_X + MENU_BOX_W - 2, button_bar_y - 1);
    int offset_int = (int)(_marquee_offset + 0.5f);
    _u8g2->drawStr(MENU_BOX_X + 4 - offset_int, MENU_BOX_Y + 25, _modal_text);
    _u8g2->setMaxClipWindow();
}

// =================================================================================
// GAME LOOP (Cập nhật 60FPS)
// =================================================================================

void SmoothOLED::update() {
    uint32_t now = millis();

    // --- AUTO DEMO LOGIC ---
    if (_auto_demo) {
        if (_overlay_state == OVERLAY_NONE) {
            if (_app_state == STATE_CAROUSEL) {
                if (now - _last_switch > 2000) {
                    _last_switch = now;
                    _current_index++;
                    if (_current_index >= _carousel_count) {
                        _current_index = 0;
                        _app_state = STATE_POPUP;
                        _target_cursor_w = 0;
                        _cursor_w = 0;
                    }
                }
            } else if (_app_state == STATE_POPUP) {
                if (now - _last_switch > 1500) {
                    _last_switch = now;
                    _current_list_selection++;
                    if (_current_list_selection >= _popup_count) {
                        _current_list_selection = 0;
                        
                        // Đóng Popup, mở Side List
                        _app_state = STATE_CAROUSEL; 
                        _overlay_state = OVERLAY_SIDE_POPUP;
                        _overlay_anim = PHASE_OPENING;
                        _side_arc_radius = 0.0f;
                        _side_list_cam_y = 0.0f;
                        _side_selected_idx = 0;
                    }
                }
            }
        } else if (_overlay_state == OVERLAY_SIDE_POPUP) {
            if (now - _last_switch > 1800) {
                _last_switch = now;
                _side_selected_idx++;
                if (_side_selected_idx >= _side_count) {
                    _side_selected_idx = 0;
                    _overlay_anim = PHASE_CLOSING;
                }
            }
        }
    }

    // --- PHYSICS & RENDER TICK (60FPS) ---
    if (now - _last_tick >= 16) {
        _last_tick = now;
        _u8g2->clearBuffer();

        int background_offset_x = 0;

        // 1. Overlay Physics
        if (_overlay_state == OVERLAY_SIDE_POPUP) {
            bool was_closing = (_overlay_anim == PHASE_CLOSING);
            update_side_physics();
            
            // Đẩy màn hình nền sang trái dựa trên hoạt ảnh của Side Popup
            // Nhưng kết hợp cả _side_slide_x khi đóng
            float combined_push = (_side_arc_radius / TARGET_ARC_RADIUS) * 46.0f;
            if (was_closing) {
                combined_push = (1.0f - (_side_slide_x / 128.0f)) * 46.0f;
            }
            background_offset_x = -(int)combined_push;
        }

        // 2. Draw Background
        if (_app_state == STATE_CAROUSEL) {
            update_physics();
            draw_carousel_menu(background_offset_x);
        } else if (_app_state == STATE_POPUP) {
            update_list_physics();
            // Tự động render lại màn hình nền dựa trên _prev_app_state
            if (_prev_app_state == STATE_CAROUSEL) {
                draw_carousel_menu(background_offset_x);
            } else if (_prev_app_state == STATE_SLIDER) {
                draw_slider_menu(background_offset_x);
            }
            // Nếu có Side List đè lên thì không vẽ box Popup nữa để tránh lỗi UI
            if (_overlay_state == OVERLAY_NONE) {
                draw_popup_menu();
            }
        } else if (_app_state == STATE_SLIDER) {
            update_slider_physics();
            draw_slider_menu(background_offset_x);
        } else if (_app_state == STATE_TEXT_INPUT) {
            // Tự động render lại màn hình nền dựa trên _prev_app_state
            if (_prev_app_state == STATE_CAROUSEL) {
                draw_carousel_menu(background_offset_x);
            } else if (_prev_app_state == STATE_POPUP) {
                // Tình huống: Popup đè lên Carousel, TextInput đè lên Popup
                draw_carousel_menu(background_offset_x); 
                draw_popup_menu();
            } else if (_prev_app_state == STATE_FULL_LIST) {
                draw_full_list_menu(background_offset_x);
            }
            if (_overlay_state == OVERLAY_NONE) {
                draw_text_input_menu();
            }
        } else if (_app_state == STATE_FULL_LIST) {
            if (_overlay_state == OVERLAY_NONE) {
                draw_full_list_menu(background_offset_x);
            }
        } else if (_app_state == STATE_MODAL) {
            update_modal_physics();
            if (_prev_app_state == STATE_CAROUSEL) {
                draw_carousel_menu(background_offset_x);
            } else if (_prev_app_state == STATE_POPUP) {
                draw_carousel_menu(background_offset_x); 
                draw_popup_menu();
            } else if (_prev_app_state == STATE_FULL_LIST) {
                draw_full_list_menu(background_offset_x);
            }
            if (_overlay_state == OVERLAY_NONE) {
                draw_modal_dialog();
            }
        } else if (_app_state == STATE_CLOCK) {
            update_clock_physics();
            draw_clock_menu(background_offset_x);
        }

        // 3. Draw Overlay
        if (_overlay_state == OVERLAY_SIDE_POPUP) {
            draw_side_list_menu();
        }

        flush_display();
    }
}

// =================================================================================
// CLOCK ATOM
// =================================================================================

void SmoothOLED::openClock() {
    if (_overlay_state == OVERLAY_NONE) {
        _prev_app_state = _app_state;
        _app_state = STATE_CLOCK;
        _clock_h1.current_val = _clock_h1.next_val = 0;
        _clock_h2.current_val = _clock_h2.next_val = 0;
        _clock_m1.current_val = _clock_m1.next_val = 0;
        _clock_m2.current_val = _clock_m2.next_val = 0;
        _clock_s1.current_val = _clock_s1.next_val = 0;
        _clock_s2.current_val = _clock_s2.next_val = 0;
        _clock_h1.anim_y = _clock_h2.anim_y = _clock_m1.anim_y = _clock_m2.anim_y = _clock_s1.anim_y = _clock_s2.anim_y = 0.0f;
        memset(_clock_solar, 0, sizeof(_clock_solar));
        memset(_clock_lunar, 0, sizeof(_clock_lunar));
    }
}

void SmoothOLED::updateClock(int h, int m, int s, const char* solar_date, const char* lunar_date) {
    auto updateDigit = [](ClockDigit& d, int new_val) {
        if (d.next_val != new_val) {
            d.current_val = d.next_val; 
            d.next_val = new_val;
            d.anim_y = 0.0f;
        }
    };

    updateDigit(_clock_h1, h / 10);
    updateDigit(_clock_h2, h % 10);
    updateDigit(_clock_m1, m / 10);
    updateDigit(_clock_m2, m % 10);
    updateDigit(_clock_s1, s / 10);
    updateDigit(_clock_s2, s % 10);

    if (solar_date) strncpy(_clock_solar, solar_date, 31);
    if (lunar_date) strncpy(_clock_lunar, lunar_date, 63);
}

void SmoothOLED::update_clock_physics() {
    auto physDigit = [](ClockDigit& d) {
        if (d.current_val != d.next_val) {
            d.anim_y += 0.1f; // Animation speed
            if (d.anim_y >= 1.0f) {
                d.anim_y = 1.0f;
                d.current_val = d.next_val;
            }
        } else {
            d.anim_y = 0.0f;
        }
    };

    physDigit(_clock_h1);
    physDigit(_clock_h2);
    physDigit(_clock_m1);
    physDigit(_clock_m2);
    physDigit(_clock_s1);
    physDigit(_clock_s2);
}

void SmoothOLED::draw_clock_menu(int offset_x) {
    // 1. Draw Solar Date
    _u8g2->setFont(u8g2_font_profont12_tf);
    int sw = _u8g2->getStrWidth(_clock_solar);
    _u8g2->drawStr(offset_x + (128 - sw) / 2, 10, _clock_solar);

    // 2. Draw Lunar Date
    sw = _u8g2->getStrWidth(_clock_lunar);
    _u8g2->drawStr(offset_x + (128 - sw) / 2, 62, _clock_lunar);

    auto drawDigit = [&](ClockDigit& d, int x, bool is_small) {
        if (is_small) _u8g2->setFont(u8g2_font_logisoso24_tn);
        else _u8g2->setFont(u8g2_font_logisoso32_tn);

        char buf[2];
        buf[1] = '\0';
        int h = is_small ? 24 : 32;
        int y = 45; // Baseline

        if (d.current_val == d.next_val) {
            buf[0] = '0' + d.current_val;
            _u8g2->drawStr(offset_x + x, y, buf);
        } else {
            int offset = (int)(d.anim_y * h);
            
            // Old digit sliding UP
            buf[0] = '0' + d.current_val;
            _u8g2->drawStr(offset_x + x, y - offset, buf);
            
            // New digit sliding UP from bottom
            buf[0] = '0' + d.next_val;
            _u8g2->drawStr(offset_x + x, y + h - offset, buf);
        }
    };

    // Clip window ensures digits don't overwrite the dates
    _u8g2->setClipWindow(offset_x, 13, offset_x + 128, 50);

    drawDigit(_clock_h1, 0, false);
    drawDigit(_clock_h2, 21, false);

    _u8g2->setFont(u8g2_font_logisoso32_tn);
    _u8g2->drawStr(offset_x + 42, 42, ":"); // ":" is slightly higher

    drawDigit(_clock_m1, 52, false);
    drawDigit(_clock_m2, 73, false);

    drawDigit(_clock_s1, 98, true);
    drawDigit(_clock_s2, 114, true);

    _u8g2->setMaxClipWindow();
}
