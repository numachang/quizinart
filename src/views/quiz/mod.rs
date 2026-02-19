mod dashboard;
mod question;
mod session;

pub use dashboard::{
    dashboard, session_history, session_result, DashboardData, SessionHistoryData,
    SessionResultData,
};
pub use question::{answer, bookmark_button, question, AnswerData, QuestionData};
pub use session::{session_name_error_page, start_page, StartPageData};

use rust_i18n::t;

pub(crate) fn selection_mode_label(mode: &str, locale: &str) -> String {
    match mode {
        "unanswered" => t!("mode.unanswered", locale = locale).to_string(),
        "incorrect" => t!("mode.incorrect", locale = locale).to_string(),
        "random" => t!("mode.random", locale = locale).to_string(),
        "bookmarked" => t!("mode.bookmarked", locale = locale).to_string(),
        _ => mode.to_string(),
    }
}
