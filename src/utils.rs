pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const COOKIE_MAX_AGE_SECS: u32 = 3600;

pub fn cookie(name: &str, value: &str) -> String {
    format!("{name}={value}; HttpOnly; Max-Age={COOKIE_MAX_AGE_SECS}; Secure; Path=/; SameSite=Strict")
}
