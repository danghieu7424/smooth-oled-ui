#ifndef SMOOTH_OLED_H
#define SMOOTH_OLED_H

#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>

struct MenuItem {
    const char* title;
    const unsigned char* icon;
};

enum AppState {
    STATE_CAROUSEL,
    STATE_POPUP
};

enum OverlayState {
    OVERLAY_NONE,
    OVERLAY_SIDE_POPUP
};

enum AnimPhase {
    PHASE_IDLE,
    PHASE_OPENING,
    PHASE_CLOSING
};

class SmoothOLED {
public:
    SmoothOLED(U8G2* u8g2, Stream* serial = nullptr);
    
    void begin();
    void update(); // Gọi liên tục trong loop()

    // Cài đặt dữ liệu các màn hình
    void setCarouselItems(const MenuItem* items, int count, const char* title = "< MAIN MENU >");
    void setPopupListItems(const char** items, int count);
    void setSidePopupItems(const char** items, int count);

    // Bật chế độ tự động chuyển mục để chạy Demo Loop
    void enableAutoDemo(bool enable);

    // --- Input API ---
    void up();
    void down();
    void openPopup();
    void openSideList();
    void closeOverlay();
    void select();

private:
    U8G2* _u8g2;
    Stream* _serial;

    AppState _app_state;
    OverlayState _overlay_state;
    AnimPhase _overlay_anim;
    uint32_t _last_switch;
    uint32_t _last_tick;
    bool _auto_demo;

    // --- Biến Carousel ---
    const MenuItem* _carousel_items;
    int _carousel_count;
    const char* _carousel_title;
    int _current_index;
    float _cam_x;
    float _target_cam_x;
    const float LERP_SPEED = 0.18f;
    const int ITEM_SPACING = 45;

    // --- Biến Popup List ---
    const char** _popup_items;
    int _popup_count;
    int _current_list_selection;
    float _cursor_y;
    float _cursor_w;
    float _target_cursor_y;
    float _target_cursor_w;
    int _list_cam_target_idx;
    float _list_cam_y;
    const float LIST_LERP_FACTOR = 0.22f;
    const int MENU_BOX_X = 32;
    const int MENU_BOX_Y = 16;
    const int MENU_BOX_W = 64;
    const int MENU_BOX_H = 36;
    const int ITEM_START_Y = 26;
    const int LINE_HEIGHT = 10;

    // --- Biến Marquee ---
    float _marquee_offset;
    uint32_t _marquee_last_time;
    int _marquee_delay;
    int _last_popup_idx;

    // --- Biến Side Popup ---
    const char** _side_items;
    int _side_count;
    int _side_selected_idx;
    float _side_parent_x;
    float _side_arc_radius;
    float _side_slide_x;
    float _side_list_cam_y;
    float _side_cursor_w;
    const float TARGET_PARENT_X = 18.0f;
    const float TARGET_ARC_RADIUS = 52.0f;
    const float SIDE_LERP_FACTOR = 0.20f;
    const int SIDE_LINE_SPACING = 14;

    void flush_display();

    void update_physics();
    void draw_carousel_menu(int offset_x = 0);

    void update_list_physics();
    void draw_popup_menu();

    void update_side_physics();
    void draw_side_list_menu();
};

#endif
