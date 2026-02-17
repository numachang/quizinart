use maud::{html, Markup};
use serde::Deserialize;
use warp::{
    http::{header::SET_COOKIE, Response},
    reject::Rejection,
    Filter,
};

use crate::{
    db::Db,
    is_authorized, is_htmx, names,
    rejections::{InputError, InternalServerError},
    utils, views, with_state,
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
    10
}

fn default_selection_mode() -> String {
    "unanswered".to_string()
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
        .and_then(quiz_dashboard);

    let quiz_page = warp::get()
        .and(is_htmx())
        .and(with_state(conn.clone()))
        .and(warp::path!("quiz" / i32))
        .and(warp::cookie::optional(names::QUIZ_SESSION_COOKIE_NAME))
        .and_then(quiz_page);

    let start_session = warp::post()
        .and(with_state(conn.clone()))
        .and(warp::path!("start-session" / i32))
        .and(warp::body::json::<StartSessionBody>())
        .and_then(start_session);

    let submit_answer = warp::post()
        .and(with_state(conn.clone()))
        .and(warp::path!("submit-answer"))
        .and(warp::cookie(names::QUIZ_SESSION_COOKIE_NAME))
        .and(warp::body::bytes())
        .and_then(submit_answer_raw);

    let session_result = warp::get()
        .and(with_state(conn.clone()))
        .and(is_htmx())
        .and(warp::path!("results" / i32))
        .and_then(session_result);

    let resume_session = warp::get()
        .and(with_state(conn.clone()))
        .and(warp::path!("resume-session" / i32 / String))
        .and_then(resume_session);

    let navigate_question = warp::get()
        .and(with_state(conn.clone()))
        .and(is_htmx())
        .and(warp::path!("question" / i32))
        .and(warp::query::<NavigateQuestionQuery>())
        .and_then(navigate_question);

    let retry_incorrect = warp::post()
        .and(with_state(conn.clone()))
        .and(warp::path!("retry-incorrect" / i32))
        .and_then(retry_incorrect);

    quiz_dashboard
        .or(quiz_page)
        .or(start_session)
        .or(submit_answer)
        .or(session_result)
        .or(resume_session)
        .or(navigate_question)
        .or(retry_incorrect)
}

async fn quiz_dashboard(
    _: (),
    is_htmx: bool,
    db: Db,
    quiz_id: i32,
) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(if is_htmx {
        views::titled("Quiz Dashboard", dashboard(&db, quiz_id).await?)
    } else {
        views::page("Quiz Dashboard", dashboard(&db, quiz_id).await?)
    })
}

async fn start_session(
    db: Db,
    quiz_id: i32,
    body: StartSessionBody,
) -> Result<impl warp::Reply, warp::Rejection> {
    let question_count = body.question_count.clamp(5, 30);

    let selection_mode = match body.selection_mode.as_str() {
        "unanswered" | "incorrect" | "random" => body.selection_mode.as_str(),
        _ => "unanswered",
    };

    let session_token = match db
        .create_session(&body.name, quiz_id, question_count, selection_mode)
        .await
    {
        Ok(token) => {
            tracing::info!("Created new session for '{}'", body.name);
            token
        }
        Err(e) if e.to_string().contains("already in use") => {
            tracing::warn!("Duplicate session name attempted: {}", body.name);

            let quiz_name = db.quiz_name(quiz_id).await.map_err(|e| {
                tracing::error!("could not get quiz name: {e}");
                warp::reject::custom(InternalServerError)
            })?;

            let error_html = views::titled(
                &quiz_name,
                views::quiz::session_name_error_page(&body.name, quiz_id),
            );

            return Ok(Response::builder()
                .status(200)
                .body(error_html.into_string())
                .unwrap());
        }
        Err(e) => {
            tracing::error!("could not create session for quiz={quiz_id}: {e}");
            return Err(warp::reject::custom(InternalServerError));
        }
    };

    let session = db.get_session(&session_token).await.map_err(|e| {
        tracing::error!("could not get session: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let quiz_name = db.quiz_name(quiz_id).await.map_err(|e| {
        tracing::error!("could not get quiz name for quiz={quiz_id}: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let question_idx = db.current_question_index(session.id).await.map_err(|e| {
        tracing::error!("could not get current question index: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let page = views::titled(
        &quiz_name,
        question(&db, session.id, quiz_id, question_idx, false).await?,
    );
    let cookie = utils::cookie(names::QUIZ_SESSION_COOKIE_NAME, &session_token);
    let resp = Response::builder()
        .header(SET_COOKIE, cookie)
        .body(page.into_string())
        .unwrap();

    Ok(resp)
}

async fn quiz_page(
    is_htmx: bool,
    db: Db,
    quiz_id: i32,
    token: Option<String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let content = match token {
        Some(token) => {
            let res = db.get_session(&token).await;

            match res {
                Ok(session) => {
                    let question_idx =
                        db.current_question_index(session.id).await.map_err(|e| {
                            tracing::error!("could not get current question index: {e}");
                            warp::reject::custom(InternalServerError)
                        })?;
                    // Normal quiz flow (Next button) - never show resume message
                    question(&db, session.id, session.quiz_id, question_idx, false).await?
                }
                Err(e) => {
                    tracing::error!("could not get session for {token}: {e}");
                    page(&db, quiz_id).await?
                }
            }
        }
        None => page(&db, quiz_id).await?,
    };

    if is_htmx {
        Ok(views::titled("Quiz", content))
    } else {
        Ok(views::page("Quiz", content))
    }
}

async fn submit_answer_raw(
    db: Db,
    token: String,
    body_bytes: bytes::Bytes,
) -> Result<impl warp::Reply, warp::Rejection> {
    let body_str = String::from_utf8(body_bytes.to_vec()).map_err(|e| {
        tracing::error!("failed to parse body as UTF-8: {e}");
        warp::reject::custom(InputError)
    })?;

    let mut option: Option<String> = None;
    let mut options: Vec<String> = Vec::new();

    for pair in body_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_value = urlencoding::decode(value)
                .map_err(|e| {
                    tracing::error!("failed to decode URL value: {e}");
                    warp::reject::custom(InputError)
                })?
                .to_string();

            match key {
                "option" => option = Some(decoded_value),
                "options" => options.push(decoded_value),
                _ => {}
            }
        }
    }

    tracing::info!("Received body: option={:?}, options={:?}", option, options);

    let body = SubmitAnswerBody { option, options };
    submit_answer(db, token, body).await
}

async fn submit_answer(
    db: Db,
    token: String,
    body: SubmitAnswerBody,
) -> Result<impl warp::Reply, warp::Rejection> {
    let session = db.get_session(&token).await.map_err(|e| {
        tracing::error!("could not get session for {token}: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let selected_ids: Vec<i32> = if !body.options.is_empty() {
        body.options
            .iter()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect()
    } else if let Some(option) = body.option {
        vec![option.parse::<i32>().map_err(|e| {
            tracing::error!("failed to parse option id: {e}");
            warp::reject::custom(InputError)
        })?]
    } else {
        tracing::error!("no options provided");
        return Err(warp::reject::custom(InputError));
    };

    let question_idx = db.current_question_index(session.id).await.map_err(|e| {
        tracing::error!("could not get current question index: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let question_id = db
        .get_question_by_idx(session.id, question_idx)
        .await
        .map_err(|e| {
            tracing::error!("could not get question id: {e}");
            warp::reject::custom(InternalServerError)
        })?;

    let question_data = db.get_question(question_id).await.map_err(|e| {
        tracing::error!("could not get question: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let is_correct = {
        let correct_ids = db.get_correct_option_ids(question_id).await.map_err(|e| {
            tracing::error!("could not get correct option ids: {e}");
            warp::reject::custom(InternalServerError)
        })?;

        if question_data.is_multiple_choice {
            let mut selected_sorted = selected_ids.clone();
            selected_sorted.sort();
            let mut correct_sorted = correct_ids.clone();
            correct_sorted.sort();

            tracing::info!(
                "Multiple choice validation: selected={:?}, correct={:?}, match={}",
                selected_sorted,
                correct_sorted,
                selected_sorted == correct_sorted
            );

            selected_sorted == correct_sorted
        } else {
            correct_ids.contains(&selected_ids[0])
        }
    };

    for option_id in &selected_ids {
        db.create_answer(session.id, question_id, *option_id, is_correct)
            .await
            .map_err(|e| {
                tracing::error!("could not create answer for {token}: {e}");
                warp::reject::custom(InternalServerError)
            })?;
    }

    db.update_question_result(session.id, question_id, is_correct)
        .await
        .map_err(|e| {
            tracing::error!("could not update question result: {e}");
            warp::reject::custom(InternalServerError)
        })?;

    let questions_count = db
        .questions_count_for_session(session.id)
        .await
        .map_err(|e| {
            tracing::error!(
                "could not get question count for session={}: {e}",
                session.id
            );
            warp::reject::custom(InternalServerError)
        })?;

    let is_final = question_idx + 1 == questions_count;

    let page = answer(
        &db,
        session.id,
        session.quiz_id,
        question_idx,
        selected_ids,
        None,
        None,
    )
    .await?;

    let resp = if is_final {
        let cookie = utils::cookie(names::QUIZ_SESSION_COOKIE_NAME, "");
        Response::builder()
            .header(SET_COOKIE, cookie)
            .body(page.into_string())
            .unwrap()
    } else {
        Response::builder().body(page.into_string()).unwrap()
    };

    Ok(resp)
}

async fn session_result(
    db: Db,
    is_htmx: bool,
    session_id: i32,
) -> Result<impl warp::Reply, warp::Rejection> {
    let session = db.get_session_by_id(session_id).await.map_err(|e| {
        tracing::error!("could not get session with id {session_id}: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let questions_count = db
        .questions_count_for_session(session.id)
        .await
        .map_err(|e| {
            tracing::error!(
                "could not get question count for session {}: {e}",
                session.id
            );
            warp::reject::custom(InternalServerError)
        })?;

    let current_idx = db.current_question_index(session.id).await.map_err(|e| {
        tracing::error!("could not get current question index: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let is_complete = current_idx >= questions_count;
    let answered_count = current_idx;

    let page = {
        let correct_answers = db.correct_answers(session.id).await.map_err(|e| {
            tracing::error!(
                "could not get correct answer count for session with id {}: {e}",
                session.id
            );
            warp::reject::custom(InternalServerError)
        })?;

        let answers = db.get_answers(session.id).await.map_err(|e| {
            tracing::error!(
                "could not get answers for session with id {}: {e}",
                session.id
            );
            warp::reject::custom(InternalServerError)
        })?;

        let quiz_name = db.quiz_name(session.quiz_id).await.map_err(|e| {
            tracing::error!("could not quiz name with id {}: {e}", session.quiz_id);
            warp::reject::custom(InternalServerError)
        })?;

        let category_stats = db.get_category_stats(session.id).await.map_err(|e| {
            tracing::error!(
                "could not get category stats for session {}: {e}",
                session.id
            );
            warp::reject::custom(InternalServerError)
        })?;

        let incorrect_count = answers.iter().filter(|a| !a.is_correct).count();

        let mode_label = selection_mode_label(
            session.selection_mode.as_deref().unwrap_or("random")
        );

        let percentage = if answered_count > 0 {
            correct_answers as f64 * 100.0 / answered_count as f64
        } else {
            0.0
        };

        html! {
            h5 { mark { (quiz_name) } }
            p style="color: #666; font-size: 0.9rem;" {
                "Mode: " strong { (mode_label) }
                " / Questions: " strong { (questions_count) }
            }

            @if !is_complete {
                article style="background-color: #fff3cd; border: 2px solid #f0ad4e; padding: 1rem; border-radius: 8px;" {
                    h4 { "Quiz In Progress" }
                    p { "You have answered " mark { (answered_count) } " out of " mark { (questions_count) } " questions." }
                    p { "Below are the results for the questions you have answered so far." }
                }
            }

            h1 {
                mark { (session.name) }
                @if !is_complete {
                    " - Progress Report"
                }
            }

            article {
                h4 { "Score" }
                p {
                    "Correct: " mark { (correct_answers) }
                    " / Answered: " mark { (answered_count) }
                    @if is_complete {
                        " / Total: " mark { (questions_count) }
                    }
                    " (" mark { (format!("{:.0}%", percentage)) } ")"
                }
            }

            @if incorrect_count > 0 && is_complete {
                article style="width: fit-content;" {
                    h4 { "Retry Incorrect" }
                    p { "You got " (incorrect_count) " questions wrong." }
                    button hx-post=(format!("/retry-incorrect/{}", session_id))
                           hx-target="main"
                           hx-swap="innerHTML"
                           style="width: fit-content; background-color: #dc3545; color: white; font-weight: 500;" {
                        "Retry " (incorrect_count) " Incorrect Questions"
                    }
                }
            }

            @if !category_stats.is_empty() {
                article {
                    h4 { "Category Performance" }
                    table {
                        thead { tr {
                            th { "Category" }
                            th { "Correct / Total" }
                            th { "Accuracy" }
                        } }
                        tbody {
                            @for stat in &category_stats {
                                tr {
                                    td { (stat.category) }
                                    td { (stat.correct) " / " (stat.total) }
                                    td { (format!("{:.1}%", stat.accuracy)) }
                                }
                            }
                        }
                    }
                }
            }

            article {
                h4 { "All Questions" }
                table {
                    thead { tr {
                        th { "#" }
                        th { "Question" }
                        th { "Correct" }
                    } }
                    tbody {
                        @for a in &answers {
                            @let url = if is_complete {
                                format!("/question/{}?question_idx={}&from=report", session_id, a.question_idx)
                            } else {
                                format!("/question/{}?question_idx={}&from=report&current_idx={}", session_id, a.question_idx, current_idx)
                            };
                            tr style="cursor: pointer;"
                               hx-get=(url)
                               hx-push-url="true"
                               hx-target="main" {
                                td { (a.question_idx + 1) }
                                td { (a.question) }
                                td { (if a.is_correct { "🟢" } else { "🔴" }) }
                             }
                        }
                    }
                }
            }

            div style="margin-top: 2rem;" {
                button hx-get=(names::quiz_dashboard_url(session.quiz_id))
                       hx-push-url="true"
                       hx-target="main"
                       style="width: fit-content;" {
                    "Back to Dashboard"
                }
            }
        }
    };

    Ok(if is_htmx {
        views::titled("Results", page)
    } else {
        views::page("Results", page)
    })
}

async fn resume_session(
    db: Db,
    session_id: i32,
    token: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    tracing::info!(
        "Resuming session {} with token {}",
        session_id,
        token
    );

    let session = db.get_session(&token).await.map_err(|e| {
        tracing::error!("could not get session: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let quiz_name = db.quiz_name(session.quiz_id).await.map_err(|e| {
        tracing::error!("could not get quiz name for quiz={}: {e}", session.quiz_id);
        warp::reject::custom(InternalServerError)
    })?;

    let question_idx = db.current_question_index(session.id).await.map_err(|e| {
        tracing::error!("could not get current question index: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    // Only show resume message from explicit dashboard resume
    let is_resuming = question_idx > 0;
    let page = views::titled(
        &quiz_name,
        question(&db, session.id, session.quiz_id, question_idx, is_resuming).await?,
    );
    let cookie_header = utils::cookie(names::QUIZ_SESSION_COOKIE_NAME, &token);

    let resp = Response::builder()
        .header(SET_COOKIE, cookie_header)
        .body(page.into_string())
        .unwrap();

    Ok(resp)
}

async fn retry_incorrect(
    db: Db,
    session_id: i32,
) -> Result<impl warp::Reply, warp::Rejection> {
    let session = db.get_session_by_id(session_id).await.map_err(|e| {
        tracing::error!("could not get session: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let incorrect_ids = db.get_incorrect_questions(session_id).await.map_err(|e| {
        tracing::error!("could not get incorrect questions: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    if incorrect_ids.is_empty() {
        let page = views::titled("Results", html! {
            p { "No incorrect questions to retry." }
            button hx-get=(names::results_url(session_id))
                   hx-push-url="true"
                   hx-target="main" {
                "Back to Results"
            }
        });
        return Ok(Response::builder()
            .body(page.into_string())
            .unwrap());
    }

    let suffix = &ulid::Ulid::new().to_string()[..6];
    let retry_name = format!("{}-retry-{}", session.name, suffix.to_lowercase());

    let token = db
        .create_session_with_questions(&retry_name, session.quiz_id, &incorrect_ids, "incorrect")
        .await
        .map_err(|e| {
            tracing::error!("could not create retry session: {e}");
            warp::reject::custom(InternalServerError)
        })?;

    let new_session = db.get_session(&token).await.map_err(|e| {
        tracing::error!("could not get new session: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let quiz_name = db.quiz_name(session.quiz_id).await.map_err(|e| {
        tracing::error!("could not get quiz name: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let page = views::titled(
        &quiz_name,
        question(&db, new_session.id, session.quiz_id, 0, false).await?,
    );
    let cookie = utils::cookie(names::QUIZ_SESSION_COOKIE_NAME, &token);

    Ok(Response::builder()
        .header(SET_COOKIE, cookie)
        .body(page.into_string())
        .unwrap())
}

async fn navigate_question(
    db: Db,
    _is_htmx: bool,
    session_id: i32,
    query: NavigateQuestionQuery,
) -> Result<impl warp::Reply, warp::Rejection> {
    let session = db.get_session_by_id(session_id).await.map_err(|e| {
        tracing::error!("could not get session: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let quiz_name = db.quiz_name(session.quiz_id).await.map_err(|e| {
        tracing::error!("could not get quiz name for quiz={}: {e}", session.quiz_id);
        warp::reject::custom(InternalServerError)
    })?;

    let question_id = db
        .get_question_by_idx(session_id, query.question_idx)
        .await
        .map_err(|e| {
            tracing::error!("could not get question id: {e}");
            warp::reject::custom(InternalServerError)
        })?;

    let is_answered = db
        .is_question_answered(session_id, question_id)
        .await
        .map_err(|e| {
            tracing::error!("could not check if question is answered: {e}");
            warp::reject::custom(InternalServerError)
        })?;

    let page = if is_answered {
        let selected_answers = db
            .get_selected_answers(session_id, question_id)
            .await
            .map_err(|e| {
                tracing::error!("could not get selected answers: {e}");
                warp::reject::custom(InternalServerError)
            })?;

        answer(
            &db,
            session_id,
            session.quiz_id,
            query.question_idx,
            selected_answers,
            query.from.clone(),
            query.current_idx,
        )
        .await?
    } else {
        question(
            &db,
            session_id,
            session.quiz_id,
            query.question_idx,
            false,
        )
        .await?
    };

    Ok(views::titled(&quiz_name, page))
}

pub async fn question(
    db: &Db,
    session_id: i32,
    quiz_id: i32,
    question_idx: i32,
    is_resuming: bool,
) -> Result<Markup, Rejection> {
    let quiz_name = db.quiz_name(quiz_id).await.map_err(|e| {
        tracing::error!("could not get quiz name for quiz={quiz_id}: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let question_id = db
        .get_question_by_idx(session_id, question_idx)
        .await
        .map_err(|e| {
            tracing::error!(
                "could not get question id for question_idx={question_idx} on session={session_id}: {e}"
            );
            warp::reject::custom(InternalServerError)
        })?;

    let question_data = db.get_question(question_id).await.map_err(|e| {
        tracing::error!("could not get question with id={question_id}: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let questions_count = db
        .questions_count_for_session(session_id)
        .await
        .map_err(|e| {
            tracing::error!("could not get question count for session={session_id}: {e}");
            warp::reject::custom(InternalServerError)
        })?;

    let is_answered = db
        .is_question_answered(session_id, question_id)
        .await
        .map_err(|e| {
            tracing::error!("could not check if question is answered: {e}");
            warp::reject::custom(InternalServerError)
        })?;

    let selected_answers = db
        .get_selected_answers(session_id, question_id)
        .await
        .map_err(|e| {
            tracing::error!("could not get selected answers: {e}");
            warp::reject::custom(InternalServerError)
        })?;

    Ok(html! {
        p { "You are doing the quiz " mark { (quiz_name) } "." }
        article style="width: fit-content;" {
            p style="color: #666; font-size: 0.9rem; margin-bottom: 0.5rem;" {
                "Question " strong { (question_idx + 1) } " of " (questions_count)
            }

            @if is_resuming {
                p style="color: #28a745; font-weight: 500; background-color: #d4edda; padding: 0.5rem; border-radius: 4px;" {
                    "Resuming from where you left off."
                }
            }

            h3 { (question_data.question) }

            @if question_data.is_multiple_choice {
                p style="color: #0066cc; font-weight: 500;" { "Multiple choice - select all that apply" }
            }

            form hx-post=(names::SUBMIT_ANSWER_URL)
                 hx-target="main"
                 hx-swap="innerHTML"
                 id="question-form" {
                fieldset {
                    @for opt in question_data.options {
                        label {
                            @if question_data.is_multiple_choice {
                                @if selected_answers.contains(&opt.id) {
                                    input type="checkbox" name="options" value=(opt.id) onchange="enableNextButton()" checked;
                                } @else {
                                    input type="checkbox" name="options" value=(opt.id) onchange="enableNextButton()";
                                }
                            } @else {
                                @if selected_answers.contains(&opt.id) {
                                    input type="radio" name="option" value=(opt.id) onchange="enableNextButton()" checked;
                                } @else {
                                    input type="radio" name="option" value=(opt.id) onchange="enableNextButton()";
                                }
                            }
                            (opt.option)
                        }
                    }
                }
                div style="display: flex; gap: 1rem; margin-top: 1rem; align-items: center;" {
                    @if question_idx > 0 {
                        button type="button" class="nav-btn"
                               hx-get=(format!("/question/{}?question_idx={}", session_id, question_idx - 1))
                               hx-target="main"
                               hx-swap="innerHTML" {
                            "Previous"
                        }
                    }
                    input type="submit" id="submit-btn" class="nav-btn" value="Submit Answer" disabled[!is_answered];
                }
            }
            script {
                "function enableNextButton() { document.getElementById('submit-btn').disabled = false; }"
            }
        }
    })
}

pub async fn answer(
    db: &Db,
    session_id: i32,
    quiz_id: i32,
    question_idx: i32,
    selected: Vec<i32>,
    from_context: Option<String>,
    current_idx: Option<i32>,
) -> Result<Markup, Rejection> {
    let quiz_name = db.quiz_name(quiz_id).await.map_err(|e| {
        tracing::error!("could not get quiz name for quiz={quiz_id}: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let question_id = db
        .get_question_by_idx(session_id, question_idx)
        .await
        .map_err(|e| {
            tracing::error!(
                "could not get question id for question_idx={question_idx} on session={session_id}: {e}"
            );
            warp::reject::custom(InternalServerError)
        })?;

    let question_data = db.get_question(question_id).await.map_err(|e| {
        tracing::error!("could not get question with id={question_id}: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let questions_count = db
        .questions_count_for_session(session_id)
        .await
        .map_err(|e| {
            tracing::error!("could not get question count for session={session_id}: {e}");
            warp::reject::custom(InternalServerError)
        })?;

    let is_final = question_idx + 1 == questions_count;

    Ok(html! {
        p { "You are doing the quiz " mark { (quiz_name) } "." }
        article style="width: fit-content;" {
            p style="color: #666; font-size: 0.9rem; margin-bottom: 0.5rem;" {
                "Question " strong { (question_idx + 1) } " of " (questions_count)
            }
            h3 { (question_data.question) }

            form {
                fieldset disabled="true" {
                    @for opt in question_data.options {
                        @let is_selected = selected.contains(&opt.id);
                        @let css_class = if opt.is_answer {
                            "option-correct"
                        } else if is_selected {
                            "option-incorrect"
                        } else {
                            "option-neutral"
                        };

                        div class=(css_class) {
                            label {
                                @if question_data.is_multiple_choice {
                                    @if is_selected {
                                        input type="checkbox" name="options[]" value=(opt.id) checked;
                                    } @else {
                                        input type="checkbox" name="options[]" value=(opt.id);
                                    }
                                } @else {
                                    @if is_selected {
                                        input type="radio" name="option" value=(opt.id) checked;
                                    } @else {
                                        input type="radio" name="option" value=(opt.id);
                                    }
                                }
                                (opt.option)
                                @if opt.is_answer {
                                    span class="badge-correct" { "Correct" }
                                } @else if is_selected {
                                    span class="badge-incorrect" { "Incorrect" }
                                }
                            }
                            @if let Some(explanation) = &opt.explanation {
                                div class="explanation" {
                                    (explanation)
                                }
                            }
                        }
                    }
                }
            }

            @if from_context.as_deref() == Some("report") {
                div style="display: flex; gap: 1rem; margin-top: 1rem; align-items: center;" {
                    button class="nav-btn"
                           hx-get=(names::results_url(session_id))
                           hx-push-url="true"
                           hx-target="main"
                           hx-disabled-elt="this" {
                        "Back to Results"
                    }
                    @if let Some(current) = current_idx {
                        button class="nav-btn"
                               hx-get=(format!("/question/{}?question_idx={}", session_id, current))
                               hx-push-url="true"
                               hx-target="main"
                               hx-disabled-elt="this"
                               style="background-color: #007bff; color: white;" {
                            "Return to Current Question"
                        }
                    }
                }
            } @else {
                div style="display: flex; gap: 1rem; margin-top: 1rem; align-items: center;" {
                    @if question_idx > 0 {
                        button type="button" class="nav-btn"
                               hx-get=(format!("/question/{}?question_idx={}", session_id, question_idx - 1))
                               hx-target="main"
                               hx-swap="innerHTML" {
                            "Previous"
                        }
                    }
                    @if is_final {
                        button class="nav-btn"
                               hx-get=(names::results_url(session_id))
                               hx-push-url="true"
                               hx-target="main" hx-disabled-elt="this" { "See Results" }
                    } @else {
                        button class="nav-btn"
                               hx-get=(names::quiz_page_url(quiz_id))
                               hx-target="main" hx-disabled-elt="this" { "Next" }
                    }
                }
            }
        }
    })
}

pub async fn page(db: &Db, quiz_id: i32) -> Result<Markup, Rejection> {
    let quiz_name = db.quiz_name(quiz_id).await.map_err(|e| {
        tracing::error!("could not get quiz name for {quiz_id}: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let total_questions = db.questions_count(quiz_id).await.map_err(|e| {
        tracing::error!("could not get question count for quiz={quiz_id}: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    Ok(html! {
        h1 { "Welcome to Quizinart!" }
        p {
            "You are going to be doing the quiz "
            mark { (quiz_name) }
            " (" (total_questions) " questions)."
        }
        article style="width: fit-content;" {
            form hx-post=(names::start_session_url(quiz_id))
                 hx-ext="json-enc"
                 hx-target="main"
                 hx-disabled-elt="find input[type='text'], find input[type='submit']"
                 hx-swap="innerHTML" {
                label {
                    "Session Name"
                    input name="name"
                          id="session-name"
                          type="text"
                          autocomplete="off"
                          aria-describedby="name-helper"
                          aria-label="Session Name"
                          pattern="[a-zA-Z0-9_\\-]+"
                          title="Only letters, numbers, underscores, and hyphens are allowed"
                          required;
                    small id="name-helper" style="display: block; margin-top: 0.5rem; color: #666;" {
                        "Use the auto-generated name or enter your own."
                    }
                }
                script {
                    (maud::PreEscaped("(function(){var d=new Date(),y=d.getFullYear(),m=String(d.getMonth()+1).padStart(2,'0'),dd=String(d.getDate()).padStart(2,'0'),r=Math.random().toString(36).substring(2,8);document.getElementById('session-name').value=y+'-'+m+'-'+dd+'-'+r;})();"))
                }
                label {
                    "Number of Questions"
                    input name="question_count"
                          type="number"
                          min="5"
                          max="30"
                          value="10"
                          aria-label="Question Count"
                          required;
                    small style="display: block; margin-top: 0.5rem; color: #666;" {
                        "Choose between 5 and 30 questions (default: 10)."
                    }
                }
                fieldset {
                    legend { "Selection Mode" }
                    label {
                        input type="radio" name="selection_mode" value="unanswered" checked;
                        "Unanswered questions (default)"
                    }
                    label {
                        input type="radio" name="selection_mode" value="incorrect";
                        "Previously incorrect questions"
                    }
                    label {
                        input type="radio" name="selection_mode" value="random";
                        "Random"
                    }
                }
                input type="submit" value="Start";
            }
        }
    })
}

pub async fn dashboard(db: &Db, quiz_id: i32) -> Result<Markup, Rejection> {
    let quiz_name = db.quiz_name(quiz_id).await.map_err(|e| {
        tracing::error!("could not get quiz name from database: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let sessions_count = db.sessions_count(quiz_id).await.map_err(|e| {
        tracing::error!("could not get sessions count for quiz_id={quiz_id}: {e}",);
        warp::reject::custom(InternalServerError)
    })?;

    let sessions = db.get_sessions_report(quiz_id).await.map_err(|e| {
        tracing::error!("could not get sessions report for quiz_id={quiz_id}: {e}",);
        warp::reject::custom(InternalServerError)
    })?;

    let overall = db.get_quiz_overall_stats(quiz_id).await.map_err(|e| {
        tracing::error!("could not get quiz overall stats for quiz_id={quiz_id}: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let cat_stats = db.get_quiz_category_stats(quiz_id).await.map_err(|e| {
        tracing::error!("could not get quiz category stats for quiz_id={quiz_id}: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let overall_accuracy = if overall.total_answered > 0 {
        overall.total_correct as f64 * 100.0 / overall.total_answered as f64
    } else {
        0.0
    };

    Ok(html! {
        h1 { (quiz_name) }

        article {
            h4 { "Overall Statistics" }
            table {
                tbody {
                    tr {
                        td { "Total Questions" }
                        td { strong { (overall.total_questions) } }
                    }
                    tr {
                        td { "Questions Asked (unique)" }
                        td { strong { (overall.unique_asked) } " / " (overall.total_questions) }
                    }
                    tr {
                        td { "Total Answers" }
                        td { strong { (overall.total_answered) } }
                    }
                    tr {
                        td { "Accuracy" }
                        td { strong { (format!("{:.1}%", overall_accuracy)) }
                            " (" (overall.total_correct) " / " (overall.total_answered) ")"
                        }
                    }
                    tr {
                        td { "Sessions" }
                        td { strong { (sessions_count) } }
                    }
                }
            }
        }

        @if !cat_stats.is_empty() {
            article {
                h4 { "Category Statistics" }
                table {
                    thead { tr {
                        th { "Category" }
                        th { "Questions" }
                        th { "Asked" }
                        th { "Accuracy" }
                    } }
                    tbody {
                        @for c in &cat_stats {
                            @let acc = if c.total_answered > 0 {
                                c.total_correct as f64 * 100.0 / c.total_answered as f64
                            } else {
                                0.0
                            };
                            tr {
                                td { (c.category) }
                                td { (c.total_in_category) }
                                td { (c.unique_asked) " / " (c.total_in_category) }
                                td {
                                    (format!("{:.1}%", acc))
                                    @if c.total_answered > 0 {
                                        small style="color: #666;" {
                                            " (" (c.total_correct) "/" (c.total_answered) ")"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        div style="margin-bottom: 1rem;" {
            button hx-get=(names::quiz_page_url(quiz_id))
                   hx-push-url="true"
                   hx-target="main"
                   style="width: fit-content; background-color: #007bff; color: white; font-weight: 500;" {
                "Start New Session"
            }
        }

        @if !sessions.is_empty() {
            article {
                h4 { "Session History" }
                table {
                    thead { tr {
                        th { "Name" }
                        th { "Mode" }
                        th { "Progress" }
                        th { "Score" }
                        th { "Status" }
                    } }
                    tbody {
                        @for s in sessions {
                            tr {
                                td { (s.name) }
                                td {
                                    (selection_mode_label(s.selection_mode.as_deref().unwrap_or("random")))
                                }
                                td {
                                    @if s.is_complete {
                                        a hx-get=(names::results_url(s.id))
                                           hx-push-url="true"
                                           hx-target="main"
                                           href="#" {
                                            (s.answered_questions) "/" (s.total_questions)
                                        }
                                    } @else {
                                        a hx-get=(names::resume_session_url(s.id, &s.session_token))
                                           hx-push-url="true"
                                           hx-target="main"
                                           href="#" {
                                            (s.answered_questions) "/" (s.total_questions)
                                        }
                                    }
                                }
                                td {
                                    a hx-get=(names::results_url(s.id))
                                       hx-push-url="true"
                                       hx-target="main"
                                       href="#" {
                                        (s.score) "/" (s.answered_questions)
                                    }
                                }
                                td {
                                    @if s.is_complete {
                                        span style="color: #28a745; font-weight: 500;" { "Complete" }
                                    } @else {
                                        span style="color: #6c757d; font-weight: 500;" { "In Progress" }
                                    }
                                }
                             }
                        }
                    }
                }
            }
        }
    })
}

fn selection_mode_label(mode: &str) -> &str {
    match mode {
        "unanswered" => "Unanswered",
        "incorrect" => "Incorrect",
        "random" => "Random",
        _ => mode,
    }
}
