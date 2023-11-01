use crate::util::{
    bindings::console_log_str,
    cookie_management::{search_for_cookie, set_document_cookie, AUTH_TOKEN_COOKIE_NAME},
};
use gloo_timers::future::TimeoutFuture;
use leptos::{ReadSignal, WriteSignal};
use serde::{Deserialize, Serialize};

use super::cookie_management::unset_document_cookie;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct UserData {
    pub username: String,
    pub admin: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct UserRequestData {
    pub username: String,
}

pub type UserDataReadSignal = ReadSignal<Option<UserData>>;
pub type UserDataWriteSignal = WriteSignal<Option<UserData>>;

/// Fetch the current user data
///
/// Specifically, this will pretend to call the backend server to request the current
/// user-data, which may evaluate to 'None' if the user is not logged-in. Alternatively,
/// it could fail (e.g. if teh backend server is unreachable). Hence Result -> Option -> UserData  
pub async fn echo_user() -> anyhow::Result<Option<UserData>> {
    TimeoutFuture::new(250).await;

    let auth_cookie = search_for_cookie(AUTH_TOKEN_COOKIE_NAME)?;

    match auth_cookie {
        Some(value) => {
            if value == "invalid" {
                Ok(None)
            } else {
                Ok(Some(UserData {
                    username: value,
                    admin: false,
                }))
            }
        }
        None => Ok(None),
    }
}

/// Pretend to perform a logout action
///
/// This function doesn't return anything, unless an error is encountered.
/// E.g. if the server is unreachable. Hence Result<()>
pub async fn logout() -> anyhow::Result<()> {
    console_log_str("logging out".to_string());
    TimeoutFuture::new(250).await;
    unset_document_cookie(AUTH_TOKEN_COOKIE_NAME).expect("To be able to unset a cookie");

    Ok(())
}

/// Pretend to perform a login action against the backend server
pub async fn login() -> anyhow::Result<UserData> {
    console_log_str("logging in".to_string());
    // The below is a stand-in for calling a "login" endpoint, which in a real-world
    // context would set an auth-cookie
    TimeoutFuture::new(250).await;
    set_document_cookie(AUTH_TOKEN_COOKIE_NAME, "bananas")?;

    // After this point, an auth cookie would be available which means we can fetch the user data
    let user_data = match echo_user().await? {
        Some(val) => val,
        None => anyhow::bail!("User is not logged-in"),
    };

    Ok(user_data)
}
