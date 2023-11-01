use leptos::html::Div;
use leptos::*;

use crate::util::{
    bindings::console_log_str,
    user_management::{login, logout, UserData, UserDataReadSignal, UserDataWriteSignal},
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

    let user_data =
        use_context::<UserDataReadSignal>().expect("user-data reading context should be available");
    let set_user_data = use_context::<UserDataWriteSignal>()
        .expect("user-data setting context should be available");

    let initial_status = match user_data.get_untracked() {
        Some(user_data) => LoginAttemptStatus::Succeeded(user_data),
        None => LoginAttemptStatus::NoUser,
    };

    let (login_tracker, set_login_tracker) = create_signal(initial_status);
    let login_action = create_local_resource(move || login_tracker.get(), try_next_login_attempt);

    create_effect(move |_| {
        let _login_action = login_action.get().unwrap_or(LoginAttemptStatus::None);
        console_log_str(format!("Calling login tracker effect -- {}", _login_action));
        match _login_action {
            LoginAttemptStatus::Succeeded(user_data) => set_user_data.set(Some(user_data)),
            _ => set_user_data.set(None),
        }
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
