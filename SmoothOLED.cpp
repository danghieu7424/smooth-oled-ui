#include "SmoothOLED.h"
#include <math.h>

SmoothOLED::SmoothOLED(U8G2* display) {
    u8g2 = display;
    app_state = STATE_CAROUSEL;
    demo_mode = false;
    
    last_tick = 0;
    last_switch = 0;

    carousel_items = nullptr;
    carousel_count = 0;
    current_index = 0;
    cam_x = 0.0f;
    target_cam_x = 0.0f;

    popup_items = nullptr;
    popup_count = 0;
    current_list_selection = 0;
    cursor_y = ITEM_START_Y - 8;
    cursor_w = 56.0f;
    target_cursor_y = ITEM_START_Y - 8;
    target_cursor_w = 56.0f;

    side_items = nullptr;
    side_count = 0;
    side_selected_idx = 0;
    side_parent_x = 32.0f;
    side_arc_radius = 0.0f;
    side_list_cam_y = 0.0f;
    side_cursor_w = 40.0f;
}

void SmoothOLED::setCarouselItems(const MenuItem* items, uint8_t count) {
    carousel_items = items;
    carousel_count = count;
}

void SmoothOLED::setPopupItems(const char** items, uint8_t count) {
    popup_items = items;
    popup_count = count;
}

void SmoothOLED::setSideMenuItems(const char** items, uint8_t count) {
    side_items = items;
    side_count = count;
}

void SmoothOLED::enableDemoMode(bool enable) {
    demo_mode = enable;
}

void SmoothOLED::changeState(AppState newState) {
    app_state = newState;
    if (app_state == STATE_SIDE_POPUP) {
        side_parent_x = 32.0f;
        side_arc_radius = 0.0f;
        side_list_cam_y = 0.0f;
        side_selected_idx = 0;
    }
}

void SmoothOLED::next() {
    if (app_state == STATE_CAROUSEL) {
        current_index++;
        if (current_index >= carousel_count) current_index = 0;
    } else if (app_state == STATE_POPUP) {
        current_list_selection++;
        if (current_list_selection >= popup_count) current_list_selection = 0;
    } else if (app_state == STATE_SIDE_POPUP) {
        side_selected_idx++;
        if (side_selected_idx >= side_count) side_selected_idx = 0;
    }
}

void SmoothOLED::prev() {
    if (app_state == STATE_CAROUSEL) {
        current_index--;
        if (current_index < 0) current_index = carousel_count - 1;
    } else if (app_state == STATE_POPUP) {
        current_list_selection--;
        if (current_list_selection < 0) current_list_selection = popup_count - 1;
    } else if (app_state == STATE_SIDE_POPUP) {
        side_selected_idx--;
        if (side_selected_idx < 0) side_selected_idx = side_count - 1;
    }
}

void SmoothOLED::select() {
    // Hàm này dự phòng cho việc bấm phím Enter vào mục chọn
}

void SmoothOLED::flush_display() {
    u8g2->sendBuffer();
    uint8_t sync[] = {0xFE, 0xFE, 0xFE, 0xFE};
    Serial.write(sync, 4);
    Serial.write(u8g2->getBufferPtr(), 1024);
}

void SmoothOLED::update(uint32_t currentMillis) {
    if (demo_mode) {
        if (app_state == STATE_CAROUSEL) {
            if (currentMillis - last_switch > 2000) {
                last_switch = currentMillis;
                current_index++;
                if (current_index >= carousel_count) {
                    current_index = 0;
                    changeState(STATE_POPUP);
                }
            }
        } else if (app_state == STATE_POPUP) {
            if (currentMillis - last_switch > 1500) {
                last_switch = currentMillis;
                current_list_selection++;
                if (current_list_selection >= popup_count) {
                    current_list_selection = 0;
                    changeState(STATE_SIDE_POPUP);
                }
            }
        } else if (app_state == STATE_SIDE_POPUP) {
            if (currentMillis - last_switch > 1800) {
                last_switch = currentMillis;
                side_selected_idx++;
                if (side_selected_idx >= side_count) {
                    side_selected_idx = 0;
                    changeState(STATE_CAROUSEL);
                }
            }
        }
    }

    if (currentMillis - last_tick >= 16) {
        last_tick = currentMillis;
        if (app_state == STATE_CAROUSEL) {
            update_carousel_physics();
            draw_carousel_menu();
        } else if (app_state == STATE_POPUP) {
            update_popup_physics();
            draw_popup_menu();
        } else if (app_state == STATE_SIDE_POPUP) {
            update_side_physics();
            draw_side_menu();
        }
    }
}

// --- PHYSICS & DRAWING METHODS ---
void SmoothOLED::update_carousel_physics() {
    target_cam_x = (float)(current_index * ITEM_SPACING);
    cam_x += (target_cam_x - cam_x) * LERP_SPEED;
}

void SmoothOLED::draw_carousel_menu() {
    u8g2->clearBuffer();
    u8g2->drawStr(30, 10, "< MAIN MENU >");

    int screen_center_x = 64;
    int screen_center_y = 32;

    for (int i = 0; i < carousel_count; i++) {
        float item_target_x = (i * ITEM_SPACING) - cam_x;
        int item_screen_x = screen_center_x + (int)item_target_x;
        
        int draw_x = item_screen_x - 12;
        int draw_y = screen_center_y - 12;

        // Chỉ vẽ Icon nếu nằm trong khung hiển thị
        if (draw_x > -24 && draw_x < 128) {
            if (i == current_index) {
                u8g2->drawXBM(draw_x, draw_y, 24, 24, carousel_items[i].icon);
            } else {
                int scaled_w = 18, scaled_h = 18;
                int scaled_x = item_screen_x - 9;
                int scaled_y = screen_center_y - 9;
                // Render Icon nhỏ cho mục không được chọn
                u8g2->drawXBM(scaled_x, scaled_y, scaled_w, scaled_h, carousel_items[i].icon);
            }
        }
    }

    const char* label = carousel_items[current_index].title;
    int str_width = u8g2->getStrWidth(label);
    u8g2->drawStr((128 - str_width) / 2, 58, label);

    flush_display();
}

void SmoothOLED::update_popup_physics() {
    target_cursor_y = (float)(ITEM_START_Y + (current_list_selection * LINE_HEIGHT) - 8);
    if (popup_items && popup_count > 0) {
        int text_pixel_width = u8g2->getStrWidth(popup_items[current_list_selection]);
        target_cursor_w = (float)(text_pixel_width + 4);
    }
    cursor_y += (target_cursor_y - cursor_y) * LIST_LERP_FACTOR;
    cursor_w += (target_cursor_w - cursor_w) * LIST_LERP_FACTOR;
}

void SmoothOLED::draw_popup_menu() {
    u8g2->clearBuffer();

    // Lớp nền phía sau
    u8g2->setDrawColor(1);
    u8g2->setFont(u8g2_font_courB18_tf);
    u8g2->drawStr(5, 42, "12");
    u8g2->drawStr(100, 42, "18");
    u8g2->setFont(u8g2_font_5x7_tf);
    u8g2->drawStr(24, 8, "Tue 01 Oct 2024");
    u8g2->drawStr(88, 60, "33.1C");

    // Xóa nền đen để đè hộp thoại
    u8g2->setDrawColor(0);
    u8g2->drawBox(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H);

    // Vẽ viền
    u8g2->setDrawColor(1);
    u8g2->drawFrame(MENU_BOX_X, MENU_BOX_Y, MENU_BOX_W, MENU_BOX_H);
    u8g2->drawFrame(MENU_BOX_X + 2, MENU_BOX_Y + 2, MENU_BOX_W - 4, MENU_BOX_H - 4);

    // Chữ danh sách
    u8g2->setFont(u8g2_font_6x10_tf);
    for (int i = 0; i < popup_count; i++) {
        int item_y = ITEM_START_Y + (i * LINE_HEIGHT);
        u8g2->drawStr(MENU_BOX_X + 4, item_y, popup_items[i]);
    }

    // Thanh con trỏ (XOR)
    u8g2->setDrawColor(2);
    u8g2->drawBox(MENU_BOX_X + 2, (int)cursor_y, (int)cursor_w, 11);
    u8g2->setDrawColor(1);

    flush_display();
}

void SmoothOLED::update_side_physics() {
    float target_parent_x = 0.0f;
    float target_arc_radius = 50.0f;
    float target_list_cam_y = (float)(side_selected_idx * SIDE_LINE_SPACING);

    if (side_items && side_count > 0) {
        int text_w = u8g2->getStrWidth(side_items[side_selected_idx]);
        float target_w = (float)(text_w + 6);
        side_cursor_w += (target_w - side_cursor_w) * SIDE_PHYSICS_SPEED;
    }

    side_parent_x += (target_parent_x - side_parent_x) * SIDE_PHYSICS_SPEED;
    side_arc_radius += (target_arc_radius - side_arc_radius) * SIDE_PHYSICS_SPEED;
    side_list_cam_y += (target_list_cam_y - side_list_cam_y) * SIDE_PHYSICS_SPEED;
}

void SmoothOLED::draw_side_menu() {
    u8g2->clearBuffer();

    int pr_x = (int)side_parent_x;
    u8g2->drawXBM(pr_x - 12, 32 - 12, 24, 24, carousel_items[current_index].icon);
    
    int str_width = u8g2->getStrWidth(carousel_items[current_index].title);
    u8g2->drawStr(pr_x - (str_width/2), 58, carousel_items[current_index].title);

    // Mặt nạ tròn
    u8g2->setDrawColor(0);
    u8g2->drawDisc(128, 32, (int)side_arc_radius);
    
    u8g2->setDrawColor(1);
    u8g2->drawCircle(128, 32, (int)side_arc_radius);
    u8g2->drawCircle(128, 32, (int)side_arc_radius - 2);

    u8g2->setFont(u8g2_font_6x10_tf);
    int base_y = 37;
    for (int i = 0; i < side_count; i++) {
        int item_y = base_y + (i * SIDE_LINE_SPACING) - (int)side_list_cam_y;

        if (item_y > 10 && item_y < 70) {
            float dy = (float)(item_y - 5 - 32); 
            if (dy < 0) dy = -dy;
            
            float x_offset = 0;
            if (side_arc_radius > dy) {
                x_offset = side_arc_radius - sqrt(side_arc_radius * side_arc_radius - dy * dy);
            } else {
                x_offset = side_arc_radius; 
            }
            
            int text_x = 41 + (int)x_offset;
            u8g2->drawStr(text_x, item_y, side_items[i]);
        }
    }

    u8g2->setDrawColor(2);
    u8g2->drawBox(39, base_y - 8, (int)side_cursor_w, 11);
    u8g2->setDrawColor(1);

    flush_display();
}
