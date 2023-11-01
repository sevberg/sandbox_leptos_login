use crate::util::bindings::console_log_str;
use crate::wasm_bindgen::JsCast;
use web_sys::{window, HtmlDocument};

pub const AUTH_TOKEN_COOKIE_NAME: &str = "AUTH_TOKEN";

fn process_key_value_str(key_value_str: &str) -> Option<(&str, &str)> {
    match key_value_str.split_once('=') {
        Some((key, value)) => Some((key.trim(), value.trim())),
        None => None,
    }
}

// Get the current HtmlDocument
fn get_document() -> anyhow::Result<HtmlDocument> {
    let window = match window() {
        Some(val) => val,
        None => anyhow::bail!("Window is not available"),
    };

    let document = match window.document() {
        Some(val) => val,
        None => anyhow::bail!("Document is not available"),
    };

    match document.dyn_into::<HtmlDocument>() {
        Ok(val) => Ok(val),
        Err(_) => anyhow::bail!("Could not cast Document into an HtmlDocument"),
    }
}

// Get all cookies in the current HtmlDocument
pub fn get_document_cookies() -> anyhow::Result<String> {
    let cookies = get_document()?
        .cookie()
        .expect("To be able to extract cookies from an HtmlDocument");

    Ok(cookies)
}

// Set a cookie with the given `key` to the given `value`
pub fn set_document_cookie(key: &str, value: &str) -> anyhow::Result<()> {
    let document = get_document()?;
    document
        .set_cookie(format!("{}={}", key, value).as_str())
        .expect("To be able to set a cookie");

    Ok(())
}

// Set a cookie with the given `key`` to "invalid"
pub fn unset_document_cookie(key: &str) -> anyhow::Result<()> {
    set_document_cookie(key, "invalid")
}

// Search the current HtmlDocument for a specific cookie, and return it's value
pub fn search_for_cookie(key: &str) -> anyhow::Result<Option<String>> {
    let all_cookies = get_document_cookies()?;
    let expected_cookie = all_cookies.split(":").find_map(|key_value_str| {
        match process_key_value_str(key_value_str) {
            Some((observed_key, observed_value)) => {
                if observed_key == key {
                    match urlencoding::decode(observed_value) {
                        Ok(val) => Some(val.into_owned()),
                        Err(_) => {
                            console_log_str(format!(
                                "Could not decode cookie value for 'key={}'",
                                key
                            ));
                            None
                        }
                    }
                } else {
                    None
                }
            }

            None => None,
        }
    });

    Ok(expected_cookie)
}
