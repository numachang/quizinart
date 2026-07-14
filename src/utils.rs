use axum::http::{header::InvalidHeaderValue, HeaderValue};

const COOKIE_MAX_AGE_SECS: u32 = 86400; // 1 day
const OAUTH_STATE_MAX_AGE_SECS: u32 = 600; // 10 minutes: covers one trip to Google's consent screen

pub fn cookie(name: &str, value: &str, secure: bool) -> Result<HeaderValue, InvalidHeaderValue> {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!("{name}={value}; HttpOnly; Max-Age={COOKIE_MAX_AGE_SECS}; Path=/; SameSite=Lax{secure_flag}")
        .parse()
}

pub fn oauth_state_cookie(value: &str, secure: bool) -> Result<HeaderValue, InvalidHeaderValue> {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{}={value}; HttpOnly; Max-Age={OAUTH_STATE_MAX_AGE_SECS}; Path=/; SameSite=Lax{secure_flag}",
        crate::names::OAUTH_STATE_COOKIE_NAME
    )
    .parse()
}

pub fn clear_cookie(name: &str, secure: bool) -> Result<HeaderValue, InvalidHeaderValue> {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!("{name}=; HttpOnly; Max-Age=0; Path=/; SameSite=Strict{secure_flag}").parse()
}
