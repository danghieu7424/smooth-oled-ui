#include "SmoothOLED.h"

SmoothOLED::SmoothOLED(U8G2* u8g2, Stream* serial) {
    _u8g2 = u8g2;
    _serial = serial;

    _app_state = STATE_CAROUSEL;
    _last_switch = 0;
    _last_tick = 0;
    _auto_demo = false;

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
    _side_arc_radius = 0.0f;
    _side_list_cam_y = 0.0f;
    _side_cursor_w = 40.0f;
}

void SmoothOLED::begin() {
    _u8g2->setFont(u8g2_font_6x10_tf);
    _last_tick = millis();
    _last_switch = millis();
}

void SmoothOLED::setCarouselItems(const MenuItem* items, int count) {
    _carousel_items = items;
    _carousel_count = count;
}

void SmoothOLED::setPopupListItems(const char** items, int count) {
    _popup_items = items;
    _popup_count = count;
}

void SmoothOLED::setSidePopupItems(const char** items, int count) {
    _side_items = items;
    _side_count = count;
}

void SmoothOLED::enableAutoDemo(bool enable) {
    _auto_demo = enable;
}

void SmoothOLED::flush_display() {
    _u8g2->sendBuffer();

    // Stream UART nếu có khai báo cổng Serial
    if (_serial) {
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

void SmoothOLED::draw_carousel_menu() {
    _u8g2->drawStr(30, 10, "< MAIN MENU >");

    int screen_center_x = 64;
    int screen_center_y = 32;

    for (int i = 0; i < _carousel_count; i++) {
        int item_real_x = screen_center_x + (i * ITEM_SPACING) - (int)_cam_x - 12; 
        int item_real_y = screen_center_y - 14;

        if (item_real_x > -30 && item_real_x < 128) {
            _u8g2->drawXBMP(item_real_x, item_real_y, 24, 24, _carousel_items[i].icon);
        }
    }

    // --- Vẽ viền khung Sci-fi (Chamfered Bracket - Bo góc nhẹ 2px, Cạnh đứng ngắn) ---
    // Khung trên
    _u8g2->drawLine(50, 14, 78, 14); // Ngang trên dài hơn
    _u8g2->drawLine(50, 14, 48, 16); // Góc chéo trái
    _u8g2->drawLine(78, 14, 80, 16); // Góc chéo phải
    _u8g2->drawLine(48, 16, 48, 18); // Dọc trái xuống (ngắn 2px)
    _u8g2->drawLine(80, 16, 80, 18); // Dọc phải xuống (ngắn 2px)

    // Khung dưới
    _u8g2->drawLine(50, 45, 78, 45); // Ngang dưới dài hơn
    _u8g2->drawLine(50, 45, 48, 43); // Góc chéo trái
    _u8g2->drawLine(78, 45, 80, 43); // Góc chéo phải
    _u8g2->drawLine(48, 43, 48, 41); // Dọc trái lên (ngắn 2px)
    _u8g2->drawLine(80, 43, 80, 41); // Dọc phải lên (ngắn 2px)

    if (_carousel_items && _carousel_count > 0) {
        const char* label = _carousel_items[_current_index].title;
        int str_width = _u8g2->getStrWidth(label);
        _u8g2->drawStr((128 - str_width) / 2, 58, label);
    }

    flush_display();
}

// =================================================================================
// POPUP LIST MENU ATOM
// =================================================================================

void SmoothOLED::update_list_physics() {
    _target_cursor_y = (float)(ITEM_START_Y + (_current_list_selection * LINE_HEIGHT) - 8);
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
                _marquee_offset += 1.0f;
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
    int MENU_BOX_X = 34;
    int MENU_BOX_Y = 16;
    int MENU_BOX_W = 64;
    int MENU_BOX_H = 32;

    // Xóa nền đen để che đi phần Main Menu bên dưới
    _u8g2->setDrawColor(0);
    _u8g2->drawBox(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H);
    _u8g2->setDrawColor(1);

    _u8g2->drawFrame(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H);

    _u8g2->setFont(u8g2_font_6x10_tf);
    
    _u8g2->setClipWindow(MENU_BOX_X + 2, MENU_BOX_Y + 1, MENU_BOX_X + MENU_BOX_W - 2, MENU_BOX_Y + MENU_BOX_H - 1);
    
    for (int i = 0; i < _popup_count; i++) {
        int text_w = _u8g2->getStrWidth(_popup_items[i]);
        int y_pos = ITEM_START_Y + (i * LINE_HEIGHT);
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
    
    _u8g2->setMaxClipWindow();

    _u8g2->setDrawColor(2);
    _u8g2->drawBox(MENU_BOX_X + 2, (int)_cursor_y, (int)_cursor_w, 11);
    _u8g2->setDrawColor(1);
}

// =================================================================================
// SIDE POPUP MENU ATOM
// =================================================================================

void SmoothOLED::update_side_physics() {
    _side_parent_x += (TARGET_PARENT_X - _side_parent_x) * SIDE_LERP_FACTOR;
    _side_arc_radius += (TARGET_ARC_RADIUS - _side_arc_radius) * SIDE_LERP_FACTOR;

    float target_cam_y = (float)(_side_selected_idx * SIDE_LINE_SPACING);
    _side_list_cam_y += (target_cam_y - _side_list_cam_y) * SIDE_LERP_FACTOR;

    if (_side_items && _side_count > 0) {
        int target_w = _u8g2->getStrWidth(_side_items[_side_selected_idx]) + 4;
        _side_cursor_w += (target_w - _side_cursor_w) * SIDE_LERP_FACTOR;
    }
}

void SmoothOLED::draw_side_list_menu() {
    _u8g2->drawStr(24, 10, "MAIN MENU");

    _u8g2->setDrawColor(1);
    
    // Fallback: draw a basic box if icon isn't accessible directly, or just assume first carousel icon is settings
    if (_carousel_items && _carousel_count > 2) {
        _u8g2->drawXBMP((int)_side_parent_x, 24, 24, 24, _carousel_items[2].icon); // icon_settings
    }

    _u8g2->setDrawColor(0);
    _u8g2->drawDisc(78, 32, (int)_side_arc_radius);

    _u8g2->setDrawColor(1);
    _u8g2->drawCircle(78, 32, (int)_side_arc_radius, U8G2_DRAW_UPPER_LEFT | U8G2_DRAW_LOWER_LEFT);

    _u8g2->setFont(u8g2_font_6x10_tf);
    int base_y = 37;
    for (int i = 0; i < _side_count; i++) {
        int item_y = base_y + (i * SIDE_LINE_SPACING) - (int)_side_list_cam_y;

        if (item_y > 0 && item_y < 74) {
            float dy = (float)(item_y - 5 - 32); 
            if (dy < 0) dy = -dy;
            
            float x_offset = 0;
            if (_side_arc_radius > dy) {
                x_offset = _side_arc_radius - sqrt(_side_arc_radius * _side_arc_radius - dy * dy);
            } else {
                x_offset = _side_arc_radius;
            }
            
            int text_x = 37 + (int)x_offset;
            _u8g2->drawStr(text_x, item_y, _side_items[i]);
        }
    }

    _u8g2->setDrawColor(2);
    _u8g2->drawBox(35, base_y - 8, (int)_side_cursor_w, 11);
    _u8g2->setDrawColor(1);
}

// =================================================================================
// MAIN LOOP & STATE MACHINE
// =================================================================================

void SmoothOLED::update() {
    uint32_t now = millis();

    // --- AUTO DEMO LOGIC ---
    if (_app_state == STATE_CAROUSEL) {
        if (_auto_demo && now - _last_switch > 2000) {
            _last_switch = now;
            _current_index++;
            if (_current_index >= _carousel_count) {
                _current_index = 0;
                _app_state = STATE_POPUP;
            }
        }
    } else if (_app_state == STATE_POPUP) {
        if (_auto_demo && now - _last_switch > 1500) {
            _last_switch = now;
            _current_list_selection++;
            if (_current_list_selection >= _popup_count) {
                _current_list_selection = 0;
                _app_state = STATE_SIDE_POPUP;
                
                _side_parent_x = 32.0f;
                _side_arc_radius = 0.0f;
                _side_list_cam_y = 0.0f;
                _side_selected_idx = 0;
            }
        }
    } else if (_app_state == STATE_SIDE_POPUP) {
        if (_auto_demo && now - _last_switch > 1800) {
            _last_switch = now;
            _side_selected_idx++;
            if (_side_selected_idx >= _side_count) {
                _side_selected_idx = 0;
                _current_index = 0; 
                _app_state = STATE_CAROUSEL;
            }
        }
    }

    // --- PHYSICS & RENDER TICK (60FPS) ---
    if (now - _last_tick >= 16) {
        _last_tick = now;
        
        _u8g2->clearBuffer();

        if (_app_state == STATE_CAROUSEL) {
            update_physics();
            draw_carousel_menu();
        } else if (_app_state == STATE_POPUP) {
            update_list_physics();
            draw_carousel_menu(); // Vẽ nền Main Menu phía sau
            draw_popup_menu();
        } else if (_app_state == STATE_SIDE_POPUP) {
            update_side_physics();
            draw_side_list_menu();
        }

        flush_display();
    }
}
