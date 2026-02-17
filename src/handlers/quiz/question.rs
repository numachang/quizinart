use maud::Markup;
use warp::{
    http::{header::SET_COOKIE, Response},
    reject::Rejection,
};

use crate::{
    db::Db,
    names,
    rejections::{AppError, ResultExt},
    utils, views,
    views::quiz as quiz_views,
};
use super::{SubmitAnswerBody, NavigateQuestionQuery};

pub(crate) async fn quiz_page(
    is_htmx: bool,
    db: Db,
    quiz_id: i32,
    token: Option<String>,
    locale: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let content = match token {
        Some(token) => {
            let res = db.get_session(&token).await;

            match res {
                Ok(session) => {
                    let question_idx = db.current_question_index(session.id).await
                        .reject("could not get current question index")?;
                    question(&db, session.id, session.quiz_id, question_idx, false, &locale).await?
                }
                Err(e) => {
                    tracing::error!("could not get session for {token}: {e}");
                    super::session::page(&db, quiz_id, &locale).await?
                }
            }
        }
        None => super::session::page(&db, quiz_id, &locale).await?,
    };

    if is_htmx {
        Ok(views::titled("Quiz", content))
    } else {
        Ok(views::page("Quiz", content, &locale))
    }
}

pub(crate) async fn submit_answer_raw(
    db: Db,
    token: String,
    body_bytes: bytes::Bytes,
    locale: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let body_str = String::from_utf8(body_bytes.to_vec())
        .reject_input("failed to parse body as UTF-8")?;

    let mut option: Option<String> = None;
    let mut options: Vec<String> = Vec::new();

    for pair in body_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_value = urlencoding::decode(value)
                .map_err(|e| {
                    tracing::error!("failed to decode URL value: {e}");
                    warp::reject::custom(AppError::Input("failed to decode URL value"))
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
    submit_answer(db, token, body, &locale).await
}

async fn submit_answer(
    db: Db,
    token: String,
    body: SubmitAnswerBody,
    locale: &str,
) -> Result<impl warp::Reply, warp::Rejection> {
    let session = db.get_session(&token).await
        .reject("could not get session")?;

    let selected_ids: Vec<i32> = if !body.options.is_empty() {
        body.options
            .iter()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect()
    } else if let Some(option) = body.option {
        vec![option.parse::<i32>()
            .reject_input("failed to parse option id")?]
    } else {
        tracing::error!("no options provided");
        return Err(warp::reject::custom(AppError::Input("no options provided")));
    };

    let question_idx = db.current_question_index(session.id).await
        .reject("could not get current question index")?;

    let question_id = db.get_question_by_idx(session.id, question_idx).await
        .reject("could not get question id")?;

    let question_data = db.get_question(question_id).await
        .reject("could not get question")?;

    let is_correct = {
        let correct_ids = db.get_correct_option_ids(question_id).await
            .reject("could not get correct option ids")?;

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
            .reject("could not create answer")?;
    }

    db.update_question_result(session.id, question_id, is_correct)
        .await
        .reject("could not update question result")?;

    let questions_count = db.questions_count_for_session(session.id).await
        .reject("could not get question count")?;

    let is_final = question_idx + 1 == questions_count;

    let page = answer(
        &db,
        session.id,
        session.quiz_id,
        question_idx,
        selected_ids,
        None,
        None,
        locale,
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

pub(crate) async fn navigate_question(
    db: Db,
    _is_htmx: bool,
    session_id: i32,
    query: NavigateQuestionQuery,
    locale: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let session = db.get_session_by_id(session_id).await
        .reject("could not get session")?;

    let quiz_name = db.quiz_name(session.quiz_id).await
        .reject("could not get quiz name")?;

    let question_id = db.get_question_by_idx(session_id, query.question_idx).await
        .reject("could not get question id")?;

    let is_answered = db.is_question_answered(session_id, question_id).await
        .reject("could not check if question is answered")?;

    let page = if is_answered {
        let selected_answers = db.get_selected_answers(session_id, question_id).await
            .reject("could not get selected answers")?;

        answer(
            &db,
            session_id,
            session.quiz_id,
            query.question_idx,
            selected_answers,
            query.from.clone(),
            query.current_idx,
            &locale,
        )
        .await?
    } else {
        question(
            &db,
            session_id,
            session.quiz_id,
            query.question_idx,
            false,
            &locale,
        )
        .await?
    };

    Ok(views::titled(&quiz_name, page))
}

// --- Helper functions: DB queries + view delegation ---

pub async fn question(
    db: &Db,
    session_id: i32,
    quiz_id: i32,
    question_idx: i32,
    is_resuming: bool,
    locale: &str,
) -> Result<Markup, Rejection> {
    let quiz_name = db.quiz_name(quiz_id).await
        .reject("could not get quiz name")?;

    let question_id = db.get_question_by_idx(session_id, question_idx).await
        .reject("could not get question id")?;

    let question_data = db.get_question(question_id).await
        .reject("could not get question")?;

    let questions_count = db.questions_count_for_session(session_id).await
        .reject("could not get question count")?;

    let is_answered = db.is_question_answered(session_id, question_id).await
        .reject("could not check if question is answered")?;

    let selected_answers = db.get_selected_answers(session_id, question_id).await
        .reject("could not get selected answers")?;

    Ok(quiz_views::question(quiz_views::QuestionData {
        quiz_name,
        question: question_data,
        question_idx,
        questions_count,
        is_answered,
        selected_answers,
        is_resuming,
        session_id,
    }, locale))
}

pub async fn answer(
    db: &Db,
    session_id: i32,
    quiz_id: i32,
    question_idx: i32,
    selected: Vec<i32>,
    from_context: Option<String>,
    current_idx: Option<i32>,
    locale: &str,
) -> Result<Markup, Rejection> {
    let quiz_name = db.quiz_name(quiz_id).await
        .reject("could not get quiz name")?;

    let question_id = db.get_question_by_idx(session_id, question_idx).await
        .reject("could not get question id")?;

    let question_data = db.get_question(question_id).await
        .reject("could not get question")?;

    let questions_count = db.questions_count_for_session(session_id).await
        .reject("could not get question count")?;

    Ok(quiz_views::answer(quiz_views::AnswerData {
        quiz_name,
        question: question_data,
        question_idx,
        questions_count,
        session_id,
        quiz_id,
        selected,
        from_context,
        current_idx,
    }, locale))
}
