#ifndef SMOOTHOLED_H
#define SMOOTHOLED_H

#include <Arduino.h>
#include <U8g2lib.h>

// Tại sao (Why): Phân tách trạng thái để chạy độc lập các Atom trên cùng 1 vòng lặp
enum AppState {
    STATE_CAROUSEL,
    STATE_POPUP,
    STATE_SIDE_POPUP
};

// Cấu trúc dữ liệu cho một mục Menu
struct MenuItem {
    const char* title;
    const unsigned char* icon;
};

/****
 * [LOGIC] Lớp điều khiển chính của thư viện Smooth OLED UI
 ****/
class SmoothOLED {
public:
    SmoothOLED(U8G2* display);

    // Cấu hình dữ liệu
    void setCarouselItems(const MenuItem* items, uint8_t count);
    void setPopupItems(const char** items, uint8_t count);
    void setSideMenuItems(const char** items, uint8_t count);

    // Bật/tắt chế độ tự động trình diễn
    void enableDemoMode(bool enable);

    // Gọi liên tục trong hàm loop()
    void update(uint32_t currentMillis);

    // Thay đổi giao diện
    void changeState(AppState newState);
    void next();
    void prev();
    void select();

private:
    U8G2* u8g2;
    
    AppState app_state;
    bool demo_mode;
    uint32_t last_tick;
    uint32_t last_switch;

    // --- Carousel Data ---
    const MenuItem* carousel_items;
    uint8_t carousel_count;
    int current_index;
    float cam_x;
    float target_cam_x;
    const float LERP_SPEED = 0.18f;
    const int ITEM_SPACING = 45;

    // --- Popup List Data ---
    const char** popup_items;
    uint8_t popup_count;
    int current_list_selection;
    float cursor_y;
    float cursor_w;
    float target_cursor_y;
    float target_cursor_w;
    const float LIST_LERP_FACTOR = 0.22f;
    const int MENU_BOX_X = 32;
    const int MENU_BOX_Y = 16;
    const int MENU_BOX_W = 64;
    const int MENU_BOX_H = 36;
    const int ITEM_START_Y = 26; 
    const int LINE_HEIGHT = 10;

    // --- Side Popup Data ---
    const char** side_items;
    uint8_t side_count;
    int side_selected_idx;
    float side_parent_x;
    float side_arc_radius;
    float side_list_cam_y;
    float side_cursor_w;
    const int SIDE_LINE_SPACING = 14;
    const float SIDE_PHYSICS_SPEED = 0.2f;

    // --- Nội bộ ---
    void update_carousel_physics();
    void draw_carousel_menu();

    void update_popup_physics();
    void draw_popup_menu();

    void update_side_physics();
    void draw_side_menu();

    void flush_display();
};

#endif // SMOOTHOLED_H
