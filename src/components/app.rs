use leptos::*;

use crate::{
    components::{control_area::ControlArea, display_area::DisplayArea},
    util::{
        bindings::console_log_str,
        cookie_management::{search_for_cookie, AUTH_TOKEN_COOKIE_NAME},
        user_management::{echo_user, UserData},
    },
};

#[component]
pub fn App() -> impl IntoView {
    // Create a user_data signal, and expose it as a context for use in child components
    let (user_data, set_user_data) = create_signal::<Option<UserData>>(None);
    provide_context(user_data);
    provide_context(set_user_data);

    // Check if is already logged-in from a previous session. If so, update the user-data signal. This
    // requires talking with the backend server, and so must be async (hence create_local_resource)
    let cached_user_data = create_local_resource(
        || (),
        |_| async move {
            console_log_str("Reading cookies".to_string());
            let auth_cookie = match search_for_cookie(AUTH_TOKEN_COOKIE_NAME) {
                Ok(search_result) => {
                    console_log_str("Found cookie".to_string());
                    search_result
                }
                Err(err) => {
                    console_log_str(format!("Error reading cookies -- {}", err.to_string()));
                    None
                }
            };

            let user_data = match auth_cookie {
                Some(_) => match echo_user().await {
                    Ok(val) => val,
                    Err(err) => {
                        console_log_str(format!("Error pinging user data -- {}", err.to_string()));
                        None
                    }
                },
                None => None,
            };

            user_data
        },
    );
    create_effect(move |_| match cached_user_data.get() {
        Some(user_data) => {
            match user_data.clone() {
                Some(val) => console_log_str(format!(
                    "Setting user from previous session -- {}",
                    val.username
                )),
                None => console_log_str(format!("No user from previous session")),
            };

            set_user_data.set(user_data);
        }
        None => console_log_str(format!("User data cache has not finished loading")),
    });

    // View generation
    view! {
        <div id="app-area">
            <ControlArea />
            <br />
            <DisplayArea />
        </div>
    }
}
