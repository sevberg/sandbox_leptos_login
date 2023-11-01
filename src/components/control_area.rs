use leptos::html::Div;
use leptos::*;

use crate::util::{
    bindings::console_log_str,
    cookie_management::{search_for_cookie, AUTH_TOKEN_COOKIE_NAME},
    user_management::{
        echo_user, login, logout, UserData, UserDataReadSignal, UserDataWriteSignal,
    },
};

#[derive(PartialEq, Clone)]
pub enum LoginAttemptStatus {
    None,
    NoUser,
    NeedsLogin,
    NeedsLogout,
    Failed(String),
    Succeeded(UserData),
}

impl std::fmt::Display for LoginAttemptStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginAttemptStatus::NoUser => write!(f, "Nothing to do"),
            LoginAttemptStatus::NeedsLogin => write!(f, "Waiting to trigger login..."),
            LoginAttemptStatus::NeedsLogout => write!(f, "Waiting to trigger logout..."),
            LoginAttemptStatus::Failed(err) => write!(f, "Failed with '{}'", err),
            LoginAttemptStatus::Succeeded(user) => write!(f, "Succeeded for '{}'", user.username),
            LoginAttemptStatus::None => write!(f, "No status? But why!?"),
        }
    }
}

impl LoginAttemptStatus {
    fn make_view(&self, setter: WriteSignal<LoginAttemptStatus>) -> HtmlElement<Div> {
        match self {
            LoginAttemptStatus::NoUser => self.make_view_no_user(setter),
            LoginAttemptStatus::NeedsLogin => self.make_view_needs_login(),
            LoginAttemptStatus::NeedsLogout => self.make_view_needs_logout(),
            LoginAttemptStatus::Failed(_) => self.make_view_failed(setter),
            LoginAttemptStatus::Succeeded(_) => self.make_view_succeeded(setter),
            LoginAttemptStatus::None => self.make_view_none(setter),
        }
    }

    fn make_view_no_user(&self, setter: WriteSignal<LoginAttemptStatus>) -> HtmlElement<Div> {
        view! {
            <div>
                <button type="button" on:click=move |_| setter.set(LoginAttemptStatus::NeedsLogin) >
                    "Log In (good)"
                </button>
                <button type="button" on:click=move |_| setter.set(LoginAttemptStatus::Failed("Bad username".to_string())) >
                    "Log In (fail)"
                </button>
            </div>
        }
    }

    fn make_view_needs_login(&self) -> HtmlElement<Div> {
        view! {
            <div>
                <button type="button" >
                    "Login in Progress..."
                </button>
            </div>
        }
    }

    fn make_view_needs_logout(&self) -> HtmlElement<Div> {
        view! {
            <div>
                <button type="button" >
                    "Logout in Progress..."
                </button>
            </div>
        }
    }

    fn make_view_failed(&self, setter: WriteSignal<LoginAttemptStatus>) -> HtmlElement<Div> {
        let reason = match self {
            LoginAttemptStatus::Failed(err) => err.to_owned(),
            _ => "Reason unknown".to_string(),
        };

        view! {
            <div>
                <button type="button" on:click=move |_| setter.set(LoginAttemptStatus::NoUser) >
                    "Reset"
                </button>
                <span>{format!("Failed: {reason}")}</span>
            </div>
        }
    }

    fn make_view_succeeded(&self, setter: WriteSignal<LoginAttemptStatus>) -> HtmlElement<Div> {
        let username = match self {
            LoginAttemptStatus::Succeeded(user_data) => user_data.username.to_owned(),
            _ => "Username unknown".to_string(),
        };

        view! {
            <div>
                <button type="button" on:click=move |_| setter.set(LoginAttemptStatus::NeedsLogout) >
                    "Log Out"
                </button>
                <span>{format!("Logged in as: {username}")}</span>
            </div>
        }
    }

    fn make_view_none(&self, setter: WriteSignal<LoginAttemptStatus>) -> HtmlElement<Div> {
        view! {
            <div>
                <button type="button" on:click=move |_| setter.set(LoginAttemptStatus::NoUser) >
                    "Reset"
                </button>
                <span>{"None"}</span>
            </div>
        }
    }
}

#[component]
pub fn ControlArea() -> impl IntoView {
    // Import the user-data setter context from the parent 'App'
    let set_user_data = use_context::<UserDataWriteSignal>()
        .expect("user-data setting context should be available");

    // Create a local signal which keeps track of login attempts (i.e. success, failure, in-progress, ect..)
    let (login_tracker, set_login_tracker) = create_signal(LoginAttemptStatus::NoUser);

    // Define a function which can handle Login attempts. Since this potentially talks to a backend
    // server, it will need to be wrapped in a 'create_local_resource'
    let try_next_login_attempt = |tracker: LoginAttemptStatus| async move {
        match tracker {
            LoginAttemptStatus::NeedsLogin => {
                let login_attempt_result = login().await;

                match login_attempt_result {
                    Ok(user_data) => LoginAttemptStatus::Succeeded(user_data),
                    Err(err) => LoginAttemptStatus::Failed(err.to_string()),
                }
            }
            LoginAttemptStatus::NeedsLogout => match logout().await {
                Ok(_) => LoginAttemptStatus::NoUser,
                Err(err) => {
                    console_log_str(format!("Logout Action failed -- {}", err.to_string()));
                    LoginAttemptStatus::NoUser
                }
            },
            other => other,
        }
    };

    let login_action = create_local_resource(move || login_tracker.get(), try_next_login_attempt);

    // In order to link the internal signal 'login_tracker' to the parent App's 'user-data' context, I need a way to
    // update the latter whenever the former changes. The only way I can get this to work is to use a `create_effect`.
    // However this is specifically AGAINST the create_effect docs, which state that one should not use create_effect
    // to perform any sort of setting within the Leptos reactive system. The docs suggest using a memo, or a derived
    // signal, but nether seem to work here
    create_effect(move |_| {
        let _login_action = login_action.get().unwrap_or(LoginAttemptStatus::None);
        console_log_str(format!("Calling login tracker effect -- {}", _login_action));
        match _login_action {
            LoginAttemptStatus::Succeeded(user_data) => set_user_data.set(Some(user_data)),
            _ => set_user_data.set(None),
        }
    });

    // Additionally, I want to check if there is already a logged-in user from a previous session. If so, I need to
    // update the login-tracker signal. This requires talking with the backend server, and so must be async (hence
    // another create_local_resource). Again, to link this back into the reactive system, I could only get it to
    // work with another create_effect
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
                Some(val) => {
                    console_log_str(format!(
                        "Setting user from previous session -- {}",
                        val.username
                    ));
                    set_login_tracker.set(LoginAttemptStatus::Succeeded(val));
                }
                None => console_log_str(format!("No user from previous session")),
            };
        }
        None => console_log_str(format!("User data cache has not finished loading")),
    });

    // View generation
    view! {
        <div id="control-area">
        {move || match login_action.get(){
            Some(status) => status.make_view(set_login_tracker),
            None => LoginAttemptStatus::None.make_view(set_login_tracker),
        }}
        </div>
    }
}
