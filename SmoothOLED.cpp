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
    _u8g2->clearBuffer();
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

    _u8g2->drawFrame(48, 14, 32, 32);
    _u8g2->drawBox(46, 16, 2, 28); 
    _u8g2->drawBox(80, 16, 2, 28);

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
    if (_popup_items && _popup_count > 0) {
        int text_pixel_width = _u8g2->getStrWidth(_popup_items[_current_list_selection]);
        _target_cursor_w = (float)(text_pixel_width + 4);
    }
    _cursor_y += (_target_cursor_y - _cursor_y) * LIST_LERP_FACTOR;
    _cursor_w += (_target_cursor_w - _cursor_w) * LIST_LERP_FACTOR;
}

void SmoothOLED::draw_popup_menu() {
    _u8g2->clearBuffer();

    _u8g2->setDrawColor(1);
    _u8g2->setFont(u8g2_font_courB18_tf);
    _u8g2->drawStr(5, 42, "12");
    _u8g2->drawStr(100, 42, "18");
    _u8g2->setFont(u8g2_font_5x7_tf);
    _u8g2->drawStr(24, 8, "Tue 01 Oct 2024");
    _u8g2->drawStr(88, 60, "33.1C");

    _u8g2->setDrawColor(0);
    _u8g2->drawBox(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H);

    _u8g2->setDrawColor(1);
    _u8g2->drawFrame(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H);

    _u8g2->setFont(u8g2_font_6x10_tf);
    for (int i = 0; i < _popup_count; i++) {
        _u8g2->drawStr(MENU_BOX_X + 4, ITEM_START_Y + (i * LINE_HEIGHT), _popup_items[i]);
    }

    _u8g2->setDrawColor(2);
    _u8g2->drawBox(MENU_BOX_X + 2, (int)_cursor_y, (int)_cursor_w, 11);
    _u8g2->setDrawColor(1);

    flush_display();
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
    _u8g2->clearBuffer();

    _u8g2->setDrawColor(1);
    _u8g2->drawStr((int)_side_parent_x - 4, 10, "MAIN MENU");
    
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

        if (item_y > 10 && item_y < 70) {
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

    flush_display();
}

// =================================================================================
// MAIN LOOP & STATE MACHINE
// =================================================================================

void SmoothOLED::update() {
    uint32_t now = millis();

    if (_app_state == STATE_CAROUSEL) {
        if (_auto_demo && now - _last_switch > 2000) {
            _last_switch = now;
            _current_index++;
            if (_current_index >= _carousel_count) {
                _current_index = 0;
                _app_state = STATE_POPUP;
            }
        }

        if (now - _last_tick >= 16) {
            _last_tick = now;
            update_physics();
            draw_carousel_menu();
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

        if (now - _last_tick >= 16) {
            _last_tick = now;
            update_list_physics();
            draw_popup_menu();
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

        if (now - _last_tick >= 16) {
            _last_tick = now;
            update_side_physics();
            draw_side_list_menu();
        }
    }
}
