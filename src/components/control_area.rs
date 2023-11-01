use leptos::html::Div;
use leptos::*;

use crate::util::{
    bindings::console_log_str,
    cookie_management::{search_for_cookie, AUTH_TOKEN_COOKIE_NAME},
    user_management::{echo_user, login, logout, UserData, UserDataWriteSignal},
};

// I need a way to track the outcome of Login attempts. The solution I eventually landed on
// involved creating the following enum, which I can then use as a signal in the component
// below. Each iteration of this enum will have a distinct view in the component, so I
// also implemented a view generator function for each case.
#[derive(PartialEq, Clone)]
pub enum LoginAttemptStatus {
    Initial,
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
            LoginAttemptStatus::Initial => write!(f, "Initial status"),
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
            LoginAttemptStatus::Initial => self.make_view_none(setter),
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
    let (login_tracker, set_login_tracker) = create_signal(LoginAttemptStatus::Initial);

    // Define a function which can handle Login attempts. Since this potentially talks to a backend
    // server, it will need to be wrapped in a 'create_local_resource'
    let try_next_login_attempt = |tracker: LoginAttemptStatus| async move {
        match tracker {
            // On the initial load, we'll check if a cookie exists. If so, we'll try loading user data
            LoginAttemptStatus::Initial => {
                console_log_str("Reading cookies".to_string());
                match search_for_cookie(AUTH_TOKEN_COOKIE_NAME) {
                    Ok(search_result) => {
                        console_log_str("Found cookie".to_string());
                        match search_result {
                            Some(_) => match echo_user().await {
                                Ok(response) => match response {
                                    Some(user_data) => LoginAttemptStatus::Succeeded(user_data),
                                    None => LoginAttemptStatus::NoUser,
                                },
                                Err(err) => {
                                    console_log_str(format!(
                                        "Error pinging user data -- {}",
                                        err.to_string()
                                    ));
                                    LoginAttemptStatus::NoUser
                                }
                            },
                            None => LoginAttemptStatus::NoUser,
                        }
                    }
                    Err(err) => {
                        console_log_str(format!("Error reading cookies -- {}", err.to_string()));
                        return LoginAttemptStatus::NoUser;
                    }
                }
            }
            // If a LogIn action is requested, then we'll call the login function, and handle it's result
            LoginAttemptStatus::NeedsLogin => {
                let login_attempt_result = login().await;

                match login_attempt_result {
                    Ok(user_data) => LoginAttemptStatus::Succeeded(user_data),
                    Err(err) => LoginAttemptStatus::Failed(err.to_string()),
                }
            }
            // If a Logout action is requested, then we'll call the logout function, and handle it's result
            LoginAttemptStatus::NeedsLogout => match logout().await {
                Ok(_) => LoginAttemptStatus::NoUser,
                Err(err) => {
                    console_log_str(format!("Logout Action failed -- {}", err.to_string()));
                    LoginAttemptStatus::NoUser
                }
            },
            // Any other status is passed on as-is
            other => other,
        }
    };

    let login_action = create_local_resource(move || login_tracker.get(), try_next_login_attempt);
    let handle_login_action = create_memo(move |_| {
        let login_action = login_action.get().unwrap_or(LoginAttemptStatus::Initial);
        console_log_str(format!("Calling login tracker memo -- {}", login_action));
        match login_action.clone() {
            LoginAttemptStatus::Succeeded(user_data) => set_user_data.set(Some(user_data)),
            _ => set_user_data.set(None),
        };
        login_action
    });

    // View generation
    view! {
        <div id="control-area">
            {move || handle_login_action.get().make_view(set_login_tracker)}
        </div>
    }
}
