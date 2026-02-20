const COOKIE_MAX_AGE_SECS: u32 = 3600;

pub fn cookie(name: &str, value: &str, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!("{name}={value}; HttpOnly; Max-Age={COOKIE_MAX_AGE_SECS}; Path=/; SameSite=Lax{secure_flag}")
}

pub fn clear_cookie(name: &str, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!("{name}=; HttpOnly; Max-Age=0; Path=/; SameSite=Strict{secure_flag}")
}
