#ifndef SMOOTH_OLED_H
#define SMOOTH_OLED_H

#include <Arduino.h>
#include <U8g2lib.h>
#include <Wire.h>

typedef void (*MenuCallback)();
typedef void (*SliderCallback)(int);
typedef void (*TextCallback)(const char*);
typedef void (*ListCallback)(int);

struct MenuItem {
    const char* title;
    const unsigned char* icon;
    MenuCallback on_enter;
};

enum AppState {
    STATE_CAROUSEL,
    STATE_POPUP,
    STATE_SLIDER,
    STATE_TEXT_INPUT,
    STATE_FULL_LIST,
    STATE_MODAL
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
    void setCarouselItems(const MenuItem* items, int count, const char* title = "");
    void setPopupListItems(const char** items, int count);
    void setSidePopupItems(const char** items, int count);

    // Tính năng Demo và Simulator
    void enableAutoDemo(bool enable);
    void enablePCViewer(bool enable);

    // --- Input API ---
    void up();
    void down();
    void left();
    void right();
    void openPopup();
    void openModal(const char* title, const char* text);
    void openSideList();
    void openSlider(const char* title, int current_val, int max_val, SliderCallback on_change = nullptr);
    void openTextInput(const char* title, TextCallback on_submit, const char* initial_text = nullptr);
    void openFullList(const char* title, const char** items, int count, ListCallback on_select = nullptr);
    void closeOverlay();
    bool backspace();
    void select();
    void inputChar(char c);

    // --- State API ---
    int getCarouselIndex() const { return _current_index; }
    bool isOverlayOpen() const { return _overlay_state != OVERLAY_NONE; }
    AppState getAppState() const { return _app_state; }
    int getSliderValue() const { return (int)(_target_slider_val + 0.5f); }
    const MenuItem* getCurrentMenuItem() const { return (_carousel_items != nullptr) ? &_carousel_items[_current_index] : nullptr; }
    int getPopupSelectedIndex() const { return _current_list_selection; }
    const char* getPopupSelectedItem() const { return (_popup_items != nullptr) ? _popup_items[_current_list_selection] : nullptr; }
    int getFullListSelectedIndex() const { return _list_selected_index; }
    void setFullListCount(int count) { _list_count = count; }

private:
    U8G2* _u8g2;
    Stream* _serial;

    AppState _app_state;
    OverlayState _overlay_state;
    AnimPhase _overlay_anim;
    AppState _prev_app_state;
    uint32_t _last_switch;
    uint32_t _last_tick;
    bool _auto_demo;
    bool _pc_viewer_enabled;

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
    const int MENU_BOX_X = 14;
    const int MENU_BOX_Y = 8;
    const int MENU_BOX_W = 100;
    const int MENU_BOX_H = 48;
    const int ITEM_START_Y = 18;
    const int LINE_HEIGHT = 12;

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
    // --- Biến Slider ---
    float _slider_val;
    float _target_slider_val;
    int _slider_max;
    const char* _slider_title;
    SliderCallback _slider_on_change;

    // --- Biến Text Input ---
    char _text_buffer[64];
    int _cursor_pos;
    int _current_char_idx;
    const char* _text_input_title;
    TextCallback _text_on_submit;

    // --- Biến Full List ---
    const char* _list_title;
    const char** _list_items;
    int _list_count;
    int _list_selected_index;
    float _list_camera_y;
    ListCallback _list_on_select;

    void flush_display();

    void update_physics();
    void draw_carousel_menu(int offset_x = 0);

    void update_list_physics();
    void draw_popup_menu();

    void update_side_physics();
    void draw_side_list_menu();

    void update_slider_physics();
    void draw_slider_menu(int offset_x = 0);
    
    void draw_text_input_menu(int offset_x = 0);
    void draw_full_list_menu(int offset_x = 0);
};

#endif
