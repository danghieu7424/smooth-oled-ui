use leptos::*;

/****
 * Component hiển thị chế độ Preview (Text/Markdown) trước khi Edit.
 * INPUT:
 * - content: Nội dung của file.
 * - ext: Phần mở rộng (ví dụ: md, txt, rs) để quyết định render Markdown hay Text thuần.
 ****/
#[component]
pub fn TextPreview(
    #[prop(into)] content: Signal<String>,
    #[prop(into)] ext: Signal<String>,
) -> impl IntoView {
    create_effect(move |_| {
        if ext.get().to_lowercase() == "md" {
            let _ = content.get(); // Trigger update
            leptos::set_timeout(move || {
                let _ = js_sys::eval(r#"
                    document.querySelectorAll('.markdown-body pre').forEach(pre => {
                        if (pre.parentNode.classList.contains('code-block-wrapper')) return;
                        
                        const code = pre.querySelector('code');
                        let language = 'Mã code';
                        if (code && code.className) {
                            const match = code.className.match(/language-(\w+)/);
                            if (match) {
                                language = match[1].charAt(0).toUpperCase() + match[1].slice(1).toLowerCase();
                            }
                        }
                        
                        const wrapper = document.createElement('div');
                        wrapper.className = 'code-block-wrapper';
                        
                        const header = document.createElement('div');
                        header.className = 'code-block-header';
                        
                        const langLabel = document.createElement('span');
                        langLabel.className = 'code-block-lang';
                        langLabel.innerText = language;
                        
                        const btn = document.createElement('button');
                        btn.className = 'copy-btn';
                        btn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg> Copy';
                        btn.onclick = function() {
                            const text = code ? code.innerText : pre.innerText;
                            navigator.clipboard.writeText(text).then(() => {
                                btn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"><path d="M20 6L9 17l-5-5"></path></svg> Copied!';
                                setTimeout(() => btn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg> Copy', 2000);
                            });
                        };
                        
                        header.appendChild(langLabel);
                        header.appendChild(btn);
                        
                        pre.parentNode.insertBefore(wrapper, pre);
                        wrapper.appendChild(header);
                        wrapper.appendChild(pre);

                        if (code && typeof hljs !== 'undefined') {
                            hljs.highlightElement(code);
                        }
                    });

                    document.querySelectorAll('.markdown-body input[type="checkbox"]').forEach(cb => {
                        // Keep disabled state but ensure it's not clickable
                        cb.onclick = function() { return false; };
                    });
                "#);
            }, std::time::Duration::from_millis(50));
        }
    });

    view! {
        <div class="text-preview-container">
            {move || {
                let e = ext.get().to_lowercase();
                let text = content.get();
                if e == "md" {
                    use pulldown_cmark::{Parser, html, Options};
                    let mut options = Options::empty();
                    options.insert(Options::ENABLE_TABLES);
                    options.insert(Options::ENABLE_STRIKETHROUGH);
                    options.insert(Options::ENABLE_TASKLISTS);
                    options.insert(Options::ENABLE_SMART_PUNCTUATION);
                    
                    let parser = Parser::new_ext(&text, options);
                    let mut html_output = String::new();
                    html::push_html(&mut html_output, parser);
                    view! { <div class="markdown-body" inner_html=html_output></div> }.into_view()
                } else {
                    // Hiển thị text thuần túy
                    view! { <pre class="plain-text-body">{text}</pre> }.into_view()
                }
            }}
        </div>
    }
}
