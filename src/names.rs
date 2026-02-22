pub const LOGIN_URL: &str = "/login";
pub const GET_STARTED_URL: &str = "/start";
pub const CREATE_QUIZ_URL: &str = "/create-quiz";
pub const SUBMIT_ANSWER_URL: &str = "/submit-answer";

pub const ADMIN_SESSION_COOKIE_NAME: &str = "admin_session"; // legacy, kept for migration
pub const USER_SESSION_COOKIE_NAME: &str = "user_session";
pub const QUIZ_SESSION_COOKIE_NAME: &str = "quiz_session";
pub const REGISTER_URL: &str = "/register";
pub const LOGOUT_URL: &str = "/logout";
pub const RESEND_VERIFICATION_URL: &str = "/resend-verification";
pub const FORGOT_PASSWORD_URL: &str = "/forgot-password";
pub const RESET_PASSWORD_URL: &str = "/reset-password";
pub const ACCOUNT_URL: &str = "/account";
pub const CHANGE_PASSWORD_URL: &str = "/change-password";

pub fn quiz_dashboard_url(public_id: &str) -> String {
    format!("/quiz/{public_id}/dashboard")
}

pub fn quiz_session_history_url(public_id: &str) -> String {
    format!("/quiz/{public_id}/sessions")
}

pub fn quiz_page_url(public_id: &str) -> String {
    format!("/quiz/{public_id}")
}

pub fn delete_quiz_url(public_id: &str) -> String {
    format!("/delete-quiz/{public_id}")
}

pub fn start_session_url(public_id: &str) -> String {
    format!("/start-session/{public_id}")
}

pub fn abandon_quiz_url(public_id: &str) -> String {
    format!("/quiz/{public_id}/abandon")
}

pub fn results_url(session_id: i32) -> String {
    format!("/results/{session_id}")
}

pub fn resume_session_url(session_id: i32, token: &str) -> String {
    format!("/resume-session/{session_id}/{token}")
}

pub fn delete_session_url(session_id: i32) -> String {
    format!("/session/{session_id}/delete")
}

pub fn rename_session_url(session_id: i32) -> String {
    format!("/session/{session_id}/rename")
}

// Quiz session defaults
pub const MIN_QUESTION_COUNT: i32 = 5;
pub const MAX_QUESTION_COUNT: i32 = 30;
pub const DEFAULT_QUESTION_COUNT: i32 = 10;
pub const DEFAULT_SELECTION_MODE: &str = "unanswered";
pub const SELECTION_MODES: &[&str] = &["unanswered", "incorrect", "random"];

// i18n
pub const LOCALE_COOKIE_NAME: &str = "lang";
pub const DEFAULT_LOCALE: &str = "en";
pub const SET_LOCALE_URL: &str = "/set-locale";
