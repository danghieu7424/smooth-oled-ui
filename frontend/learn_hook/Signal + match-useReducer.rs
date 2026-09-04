// 1. Định nghĩa các hành động (Actions)
enum CounterAction {
    Increment,
    Decrement,
    Reset,
    Add(i32), // Payload
}

#[component]
fn CounterReducer() -> impl IntoView {
    // State gốc
    let (state, set_state) = create_signal(0);

    // 2. Tạo hàm dispatch (Reducer Logic)
    // Thay vì switch/case của JS, ta dùng match của Rust (mạnh hơn nhiều)
    let dispatch = move |action: CounterAction| {
        set_state.update(|count| match action {
            CounterAction::Increment => *count += 1,
            CounterAction::Decrement => *count -= 1,
            CounterAction::Reset => *count = 0,
            CounterAction::Add(n) => *count += n,
        });
    };

    view! {
        <div>
            <h1>"Count: " {state}</h1>
            <button on:click=move |_| dispatch(CounterAction::Decrement)>"-1"</button>
            <button on:click=move |_| dispatch(CounterAction::Increment)>"+1"</button>
            <button on:click=move |_| dispatch(CounterAction::Add(5))  >"+5"</button>
            <button on:click=move |_| dispatch(CounterAction::Reset)>"Reset"</button>
        </div>
    }
}