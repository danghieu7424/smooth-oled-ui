use leptos::*;
use crate::shared::ui::checkbox::Checkbox;

#[component]
pub fn SubtitleSettingsModal(
    #[prop(into)] mode: Signal<u8>,
    #[prop(into)] set_mode: Callback<u8>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(optional, into)] available_langs: Option<Signal<Vec<String>>>,
    #[prop(optional, into)] active_lang: Option<Signal<String>>,
    #[prop(optional, into)] set_active_lang: Option<Callback<String>>,
) -> impl IntoView {
    
    let show_top = Signal::derive(move || mode.get() == 1 || mode.get() == 2);
    let show_bottom = Signal::derive(move || mode.get() == 1 || mode.get() == 3);

    let handle_top_change = move |checked: bool| {
        let current_mode = mode.get_untracked();
        let bottom = current_mode == 1 || current_mode == 3;
        if checked && bottom { set_mode.call(1); }
        else if checked && !bottom { set_mode.call(2); }
        else if !checked && bottom { set_mode.call(3); }
        else { set_mode.call(0); }
    };
    
    let handle_bottom_change = move |checked: bool| {
        let current_mode = mode.get_untracked();
        let top = current_mode == 1 || current_mode == 2;
        if top && checked { set_mode.call(1); }
        else if top && !checked { set_mode.call(2); }
        else if !top && checked { set_mode.call(3); }
        else { set_mode.call(0); }
    };

    let has_langs = Signal::derive(move || {
        available_langs.map(|l| !l.get().is_empty()).unwrap_or(false)
    });

    view! {
        <div class="sub-modal-overlay" on:click=move |_| on_close.call(())>
            <div class="sub-modal-content" on:click=move |ev| ev.stop_propagation()>
                <div class="modal-handle"></div>
                <div class="modal-header">
                    <span class="sub-title">"Phụ đề"</span>
                    <button class="close-btn" on:click=move |_| on_close.call(())>
                        <svg viewBox="0 0 24 24" fill="currentColor" width="24" height="24">
                            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"></path>
                        </svg>
                    </button>
                </div>
                
                <div class="sub-body">
                    <Checkbox checked=show_top on_change=handle_top_change label="Hiện dòng trên".to_string() />
                    <Checkbox checked=show_bottom on_change=handle_bottom_change label="Hiện dòng dưới".to_string() />
                    
                    <Show when=move || has_langs.get() fallback=|| ()>
                        <div class="lang-selection" style="margin-top: 12px; padding-top: 12px; border-top: 1px solid rgba(255, 255, 255, 0.1);">
                            <div style="font-size: 13px; color: #a1a1aa; margin-bottom: 8px;">"Ngôn ngữ"</div>
                            <div class="lang-options" style="display: flex; gap: 8px; flex-wrap: wrap;">
                                {move || {
                                    if let Some(langs_sig) = available_langs {
                                        let langs = langs_sig.get();
                                        langs.into_iter().map(|l| {
                                            let is_active = active_lang.map(|a| a.get() == l).unwrap_or(false);
                                            let l_clone = l.clone();
                                            view! {
                                                <button 
                                                    class="lang-btn" 
                                                    style=move || if is_active {
                                                        "background: #3b82f6; color: white; border: none; padding: 4px 12px; border-radius: 12px; font-size: 13px; cursor: pointer;"
                                                    } else {
                                                        "background: rgba(255,255,255,0.1); color: #e4e4e7; border: none; padding: 4px 12px; border-radius: 12px; font-size: 13px; cursor: pointer;"
                                                    }
                                                    on:click=move |_| {
                                                        if let Some(set_cb) = set_active_lang {
                                                            set_cb.call(l_clone.clone());
                                                        }
                                                    }
                                                >
                                                    {l.to_uppercase()}
                                                </button>
                                            }
                                        }).collect_view()
                                    } else {
                                        view! { <div></div> }.into_view()
                                    }
                                }}
                            </div>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}
