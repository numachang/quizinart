pub const LOGIN_URL: &str = "/login";
pub const GET_STARTED_URL: &str = "/start";
pub const CREATE_QUIZ_URL: &str = "/create-quiz";
pub const SUBMIT_ANSWER_URL: &str = "/submit-answer";

pub const ADMIN_SESSION_COOKIE_NAME: &str = "admin_session";
pub const QUIZ_SESSION_COOKIE_NAME: &str = "quiz_session";

pub fn quiz_dashboard_url(quiz_id: i32) -> String {
    format!("/quiz/{quiz_id}/dashboard")
}

pub fn quiz_page_url(quiz_id: i32) -> String {
    format!("/quiz/{quiz_id}")
}

pub fn delete_quiz_url(quiz_id: i32) -> String {
    format!("/delete-quiz/{quiz_id}")
}

pub fn start_session_url(quiz_id: i32) -> String {
    format!("/start-session/{quiz_id}")
}

pub fn results_url(session_id: i32) -> String {
    format!("/results/{session_id}")
}

pub fn resume_session_url(session_id: i32, token: &str) -> String {
    format!("/resume-session/{session_id}/{token}")
}
