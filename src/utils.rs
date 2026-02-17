pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn cookie(name: &str, value: &str) -> String {
    format!("{name}={value}; HttpOnly; Max-Age=3600; Secure; Path=/; SameSite=Strict")
}
