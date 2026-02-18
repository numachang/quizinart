mod dashboard;
mod question;
mod session;

pub use dashboard::dashboard;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use serde::Deserialize;

use crate::{names, AppState};

/// Deserialize a value that may be either a JSON number or a string containing a number.
/// HTML forms via htmx json-enc always send values as strings.
fn deserialize_string_or_i32<'de, D: serde::Deserializer<'de>>(d: D) -> Result<i32, D::Error> {
    struct Vis;
    impl<'de> serde::de::Visitor<'de> for Vis {
        type Value = i32;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("number or numeric string")
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<i32, E> {
            i32::try_from(v).map_err(E::custom)
        }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<i32, E> {
            i32::try_from(v).map_err(E::custom)
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<i32, E> {
            v.parse().map_err(E::custom)
        }
    }
    d.deserialize_any(Vis)
}

#[derive(Deserialize)]
struct StartSessionBody {
    name: String,
    #[serde(
        default = "default_question_count",
        deserialize_with = "deserialize_string_or_i32"
    )]
    question_count: i32,
    #[serde(default = "default_selection_mode")]
    selection_mode: String,
}

fn default_question_count() -> i32 {
    names::DEFAULT_QUESTION_COUNT
}

fn default_selection_mode() -> String {
    names::DEFAULT_SELECTION_MODE.to_string()
}

#[derive(Deserialize)]
struct SubmitAnswerBody {
    #[serde(default)]
    option: Option<String>,
    #[serde(default)]
    options: Vec<String>,
}

#[derive(Deserialize)]
struct RenameSessionBody {
    name: String,
}

#[derive(Deserialize)]
struct NavigateQuestionQuery {
    question_idx: i32,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    current_idx: Option<i32>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/quiz/{id}/dashboard", get(dashboard::quiz_dashboard))
        .route("/quiz/{id}", get(question::quiz_page))
        .route("/start-session/{id}", post(session::start_session))
        .route("/submit-answer", post(question::submit_answer_raw))
        .route("/results/{id}", get(dashboard::session_result))
        .route("/resume-session/{id}/{token}", get(session::resume_session))
        .route("/question/{id}", get(question::navigate_question))
        .route("/retry-incorrect/{id}", post(session::retry_incorrect))
        .route("/retry-bookmarked/{id}", post(session::retry_bookmarked))
        .route(
            "/toggle-bookmark/{session_id}/{question_id}",
            post(question::toggle_bookmark),
        )
        .route("/session/{id}/delete", delete(session::delete_session))
        .route("/session/{id}/rename", patch(session::rename_session))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_session_body_accepts_numeric_string() {
        let body: StartSessionBody = serde_json::from_str(
            r#"{"name":"alice","question_count":"10","selection_mode":"random"}"#,
        )
        .expect("should parse numeric string");

        assert_eq!(body.question_count, 10);
    }

    #[test]
    fn start_session_body_rejects_out_of_range_i64() {
        let result = serde_json::from_str::<StartSessionBody>(
            r#"{"name":"alice","question_count":2147483648,"selection_mode":"random"}"#,
        );

        assert!(result.is_err());
    }

    #[test]
    fn start_session_body_rejects_out_of_range_u64() {
        let result = serde_json::from_str::<StartSessionBody>(
            r#"{"name":"alice","question_count":9223372036854775808,"selection_mode":"random"}"#,
        );

        assert!(result.is_err());
    }
}
