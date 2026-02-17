mod dashboard;
mod question;
mod session;

pub use dashboard::dashboard;

use serde::Deserialize;
use warp::Filter;

use crate::{
    db::Db,
    is_authorized, is_htmx, names, with_locale, with_state,
};

/// Deserialize a value that may be either a JSON number or a string containing a number.
/// HTML forms via htmx json-enc always send values as strings.
fn deserialize_string_or_i32<'de, D: serde::Deserializer<'de>>(d: D) -> Result<i32, D::Error> {
    struct Vis;
    impl<'de> serde::de::Visitor<'de> for Vis {
        type Value = i32;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("number or numeric string")
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<i32, E> { Ok(v as i32) }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<i32, E> { Ok(v as i32) }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<i32, E> {
            v.parse().map_err(E::custom)
        }
    }
    d.deserialize_any(Vis)
}

#[derive(Deserialize)]
struct StartSessionBody {
    name: String,
    #[serde(default = "default_question_count", deserialize_with = "deserialize_string_or_i32")]
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
struct NavigateQuestionQuery {
    question_idx: i32,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    current_idx: Option<i32>,
}

pub fn route(
    conn: Db,
) -> impl warp::Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let quiz_dashboard = is_authorized(conn.clone())
        .and(is_htmx())
        .and(with_state(conn.clone()))
        .and(warp::get())
        .and(warp::path!("quiz" / i32 / "dashboard"))
        .and(with_locale())
        .and_then(dashboard::quiz_dashboard);

    let quiz_page = warp::get()
        .and(is_htmx())
        .and(with_state(conn.clone()))
        .and(warp::path!("quiz" / i32))
        .and(warp::cookie::optional(names::QUIZ_SESSION_COOKIE_NAME))
        .and(with_locale())
        .and_then(question::quiz_page);

    let start_session = warp::post()
        .and(with_state(conn.clone()))
        .and(warp::path!("start-session" / i32))
        .and(warp::body::json::<StartSessionBody>())
        .and(with_locale())
        .and_then(session::start_session);

    let submit_answer = warp::post()
        .and(with_state(conn.clone()))
        .and(warp::path!("submit-answer"))
        .and(warp::cookie(names::QUIZ_SESSION_COOKIE_NAME))
        .and(warp::body::bytes())
        .and(with_locale())
        .and_then(question::submit_answer_raw);

    let session_result = warp::get()
        .and(with_state(conn.clone()))
        .and(is_htmx())
        .and(warp::path!("results" / i32))
        .and(with_locale())
        .and_then(dashboard::session_result);

    let resume_session = warp::get()
        .and(with_state(conn.clone()))
        .and(warp::path!("resume-session" / i32 / String))
        .and(with_locale())
        .and_then(session::resume_session);

    let navigate_question = warp::get()
        .and(with_state(conn.clone()))
        .and(is_htmx())
        .and(warp::path!("question" / i32))
        .and(warp::query::<NavigateQuestionQuery>())
        .and(with_locale())
        .and_then(question::navigate_question);

    let retry_incorrect = warp::post()
        .and(with_state(conn.clone()))
        .and(warp::path!("retry-incorrect" / i32))
        .and(with_locale())
        .and_then(session::retry_incorrect);

    quiz_dashboard
        .or(quiz_page)
        .or(start_session)
        .or(submit_answer)
        .or(session_result)
        .or(resume_session)
        .or(navigate_question)
        .or(retry_incorrect)
}
