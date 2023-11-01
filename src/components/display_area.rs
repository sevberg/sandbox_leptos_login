use leptos::*;

use crate::util::user_management::UserDataReadSignal;

#[component]
pub fn DisplayArea() -> impl IntoView {
    let user_data =
        use_context::<UserDataReadSignal>().expect("user reader context should be available");

    let generate_display_text = move || match user_data.get() {
        Some(user) => format!("Hello there, {}!", user.username),
        None => "You're a mystery. Please log-in".to_string(),
    };

    // View generation
    view! {
        <div id="display-area">
            <span>{
                move || generate_display_text()}</span>
        </div>
    }
}
