use std::collections::HashMap;

use axum::{
    extract::{Multipart, State},
    http::{header::SET_COOKIE, HeaderMap},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use maud::html;
use serde::Deserialize;

use crate::{
    extractors::{AuthGuard, Locale},
    handlers::quiz,
    models, names,
    rejections::{AppError, ResultExt},
    utils, views, AppState,
};

use crate::views::homepage as homepage_views;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(homepage))
        .route("/start", post(get_started_post))
        .route("/login", post(login_post))
        .route("/create-quiz", post(create_quiz))
        .route("/delete-quiz/{id}", delete(delete_quiz))
        .route("/set-locale", post(set_locale))
}

async fn homepage(
    State(state): State<AppState>,
    jar: CookieJar,
    Locale(locale): Locale,
) -> Result<maud::Markup, AppError> {
    let session = jar
        .get(names::ADMIN_SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string());

    let session_exists = match session {
        Some(s) => state.db.admin_session_exists(s).await.unwrap_or(false),
        None => false,
    };

    if session_exists {
        let quizzes = state.db.quizzes().await.reject("could not get quizzes")?;
        Ok(views::page(
            "Dashboard",
            homepage_views::dashboard(quizzes, &locale),
            &locale,
        ))
    } else {
        let admin_password = state
            .db
            .admin_password()
            .await
            .reject("could not get admin password")?;

        match admin_password {
            Some(_) => Ok(views::page(
                "Welcome Back",
                homepage_views::login(homepage_views::LoginState::NoError, &locale),
                &locale,
            )),
            None => Ok(views::page(
                "Get Started",
                homepage_views::get_started(&locale),
                &locale,
            )),
        }
    }
}

#[derive(Deserialize)]
struct GetStartedPost {
    admin_password: String,
}

async fn get_started_post(
    State(state): State<AppState>,
    Locale(locale): Locale,
    Json(body): Json<GetStartedPost>,
) -> Result<impl IntoResponse, AppError> {
    state
        .db
        .set_admin_password(body.admin_password)
        .await
        .reject("could not set admin password")?;

    let session = state
        .db
        .create_admin_session()
        .await
        .reject("could not create admin session")?;

    let cookie = utils::cookie(names::ADMIN_SESSION_COOKIE_NAME, &session);
    let quizzes = state.db.quizzes().await.reject("could not get quizzes")?;

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    Ok((
        headers,
        views::titled("Dashboard", homepage_views::dashboard(quizzes, &locale)),
    ))
}

#[derive(Deserialize)]
struct LoginPost {
    admin_password: String,
}

async fn login_post(
    State(state): State<AppState>,
    Locale(locale): Locale,
    Json(body): Json<LoginPost>,
) -> Result<axum::response::Response, AppError> {
    let admin_password = state
        .db
        .admin_password()
        .await
        .reject("could not get admin password")?;

    if admin_password == Some(body.admin_password) {
        let session = state
            .db
            .create_admin_session()
            .await
            .reject("could not create admin session")?;

        let cookie = utils::cookie(names::ADMIN_SESSION_COOKIE_NAME, &session);
        let quizzes = state.db.quizzes().await.reject("could not get quizzes")?;

        let mut headers = HeaderMap::new();
        headers.insert(SET_COOKIE, cookie.parse().unwrap());

        Ok((
            headers,
            views::titled("Dashboard", homepage_views::dashboard(quizzes, &locale)),
        )
            .into_response())
    } else {
        Ok(views::titled(
            "Welcome Back",
            homepage_views::login(homepage_views::LoginState::IncorrectPassword, &locale),
        )
        .into_response())
    }
}

async fn create_quiz(
    _guard: AuthGuard,
    State(state): State<AppState>,
    Locale(locale): Locale,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut field_names: HashMap<String, String> = HashMap::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("failed to read multipart field: {e}");
        AppError::Input("failed to read multipart field")
    })? {
        let name = field.name().unwrap_or_default().to_string();
        let text = field.text().await.map_err(|e| {
            tracing::error!("failed to read field data: {e}");
            AppError::Input("failed to read field data")
        })?;
        field_names.insert(name, text);
    }

    let quiz_name = field_names
        .remove("quiz_name")
        .ok_or(AppError::Input("missing quiz_name field"))?;

    let quiz_file = field_names
        .remove("quiz_file")
        .ok_or(AppError::Input("missing quiz_file field"))?;

    let questions = serde_json::from_str::<models::Questions>(&quiz_file)
        .reject_input("failed to decode quiz file")?;

    let quiz_id = state
        .db
        .load_quiz(quiz_name, questions)
        .await
        .reject_input("failed to load quiz")?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Replace-Url",
        names::quiz_dashboard_url(quiz_id).parse().unwrap(),
    );

    Ok((
        headers,
        views::titled(
            "Quiz Dashboard",
            quiz::dashboard(&state.db, quiz_id, &locale).await?,
        ),
    ))
}

async fn delete_quiz(
    _guard: AuthGuard,
    State(state): State<AppState>,
    axum::extract::Path(quiz_id): axum::extract::Path<i32>,
) -> Result<maud::Markup, AppError> {
    state
        .db
        .delete_quiz(quiz_id)
        .await
        .reject("failed to delete quiz")?;

    Ok(html!())
}

#[derive(Deserialize)]
struct SetLocaleBody {
    locale: String,
}

async fn set_locale(Json(body): Json<SetLocaleBody>) -> Result<impl IntoResponse, AppError> {
    let locale = match body.locale.as_str() {
        "ja" => "ja",
        "zh-CN" => "zh-CN",
        "zh-TW" => "zh-TW",
        _ => "en",
    };
    let cookie = utils::cookie(names::LOCALE_COOKIE_NAME, locale);
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    headers.insert("HX-Refresh", "true".parse().unwrap());

    Ok((headers, ""))
}
