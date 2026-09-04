use leptos::html::Input; // Import kiểu thẻ Input

#[component]
fn FocusDemo() -> impl IntoView {
    // React: const inputRef = useRef(null);
    let input_element: NodeRef<Input> = create_node_ref();

    let on_click = move |_| {
        // Access vào DOM thật
        if let Some(input) = input_element.get() {
            input.focus().expect("Không focus được");
        }
    };

    view! {
        // Gắn ref vào thẻ (node_ref=...)
        <input type="text" node_ref=input_element placeholder="Focus me!" />
        <button on:click=on_click>"Click để Focus Input"</button>
    }
}