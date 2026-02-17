mod dashboard;
mod question;
mod session;

pub use dashboard::{DashboardData, SessionResultData, dashboard, session_result};
pub use question::{AnswerData, QuestionData, answer, question};
pub use session::{StartPageData, session_name_error_page, start_page};

use rust_i18n::t;

pub(crate) fn selection_mode_label(mode: &str, locale: &str) -> String {
    match mode {
        "unanswered" => t!("mode.unanswered", locale = locale).to_string(),
        "incorrect" => t!("mode.incorrect", locale = locale).to_string(),
        "random" => t!("mode.random", locale = locale).to_string(),
        _ => mode.to_string(),
    }
}
