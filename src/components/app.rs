use leptos::*;

use crate::{
    components::{control_area::ControlArea, display_area::DisplayArea},
    util::user_management::UserData,
};

#[component]
pub fn App() -> impl IntoView {
    // Create a user_data signal, and expose it as a context for use in child components
    let (user_data, set_user_data) = create_signal::<Option<UserData>>(None);
    provide_context(user_data);
    provide_context(set_user_data);

    // View generation
    view! {
        <div id="app-area">
            <ControlArea />
            <br />
            <DisplayArea />
        </div>
    }
}
