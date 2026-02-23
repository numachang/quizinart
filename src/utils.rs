use axum::http::{header::InvalidHeaderValue, HeaderValue};

const COOKIE_MAX_AGE_SECS: u32 = 86400; // 1 day

pub fn cookie(name: &str, value: &str, secure: bool) -> Result<HeaderValue, InvalidHeaderValue> {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!("{name}={value}; HttpOnly; Max-Age={COOKIE_MAX_AGE_SECS}; Path=/; SameSite=Lax{secure_flag}")
        .parse()
}

pub fn clear_cookie(name: &str, secure: bool) -> Result<HeaderValue, InvalidHeaderValue> {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!("{name}=; HttpOnly; Max-Age=0; Path=/; SameSite=Strict{secure_flag}").parse()
}
