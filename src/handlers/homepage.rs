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
        .route("/register", get(register_page).post(register_post))
        .route("/login", post(login_post))
        .route("/logout", post(logout_post))
        .route("/create-quiz", post(create_quiz))
        .route("/delete-quiz/{id}", delete(delete_quiz))
        .route("/set-locale", post(set_locale))
}

async fn register_page(Locale(locale): Locale) -> maud::Markup {
    views::page(
        "Register",
        homepage_views::register(homepage_views::RegisterState::NoError, &locale),
        &locale,
    )
}

async fn homepage(
    State(state): State<AppState>,
    jar: CookieJar,
    Locale(locale): Locale,
) -> Result<maud::Markup, AppError> {
    // Check new user_session cookie first
    if let Some(session_id) = jar
        .get(names::USER_SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
    {
        if let Ok(Some(user)) = state.db.get_user_by_session(&session_id).await {
            let quizzes = state
                .db
                .quizzes(user.id)
                .await
                .reject("could not get quizzes")?;
            return Ok(views::page_with_user(
                "Dashboard",
                homepage_views::dashboard(quizzes, &locale),
                &locale,
                Some(&user.display_name),
            ));
        }
    }

    // Fallback: check legacy admin_session cookie
    if let Some(admin_session) = jar
        .get(names::ADMIN_SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
    {
        let exists = state
            .db
            .admin_session_exists(admin_session)
            .await
            .unwrap_or(false);
        if exists {
            if let Ok(Some(user)) = state.db.find_user_by_email("admin@local").await {
                let quizzes = state
                    .db
                    .quizzes(user.id)
                    .await
                    .reject("could not get quizzes")?;
                return Ok(views::page_with_user(
                    "Dashboard",
                    homepage_views::dashboard(quizzes, &locale),
                    &locale,
                    Some(&user.display_name),
                ));
            }
        }
    }

    // Not logged in: show login page
    Ok(views::page(
        "Log In",
        homepage_views::login(homepage_views::LoginState::NoError, &locale),
        &locale,
    ))
}

#[derive(Deserialize)]
struct RegisterPost {
    email: String,
    display_name: String,
    password: String,
}

async fn register_post(
    State(state): State<AppState>,
    Locale(locale): Locale,
    Json(body): Json<RegisterPost>,
) -> Result<axum::response::Response, AppError> {
    // Validate inputs
    if body.email.is_empty() || body.password.is_empty() || body.display_name.is_empty() {
        return Ok(views::titled(
            "Register",
            homepage_views::register(homepage_views::RegisterState::EmptyFields, &locale),
        )
        .into_response());
    }

    // Check if email already exists
    let exists = state
        .db
        .email_exists(&body.email)
        .await
        .reject("could not check email")?;

    if exists {
        return Ok(views::titled(
            "Register",
            homepage_views::register(homepage_views::RegisterState::EmailTaken, &locale),
        )
        .into_response());
    }

    // Create user
    let user_id = state
        .db
        .create_user(&body.email, &body.password, &body.display_name)
        .await
        .reject("could not create user")?;

    // Create session
    let session = state
        .db
        .create_user_session(user_id)
        .await
        .reject("could not create session")?;

    let cookie = utils::cookie(
        names::USER_SESSION_COOKIE_NAME,
        &session,
        state.secure_cookies,
    );
    let quizzes = state
        .db
        .quizzes(user_id)
        .await
        .reject("could not get quizzes")?;

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    Ok((
        headers,
        views::titled("Dashboard", homepage_views::dashboard(quizzes, &locale)),
    )
        .into_response())
}

#[derive(Deserialize)]
struct LoginPost {
    email: String,
    password: String,
}

async fn login_post(
    State(state): State<AppState>,
    Locale(locale): Locale,
    Json(body): Json<LoginPost>,
) -> Result<axum::response::Response, AppError> {
    let verified = state
        .db
        .verify_user_password(&body.email, &body.password)
        .await
        .reject("could not verify password")?;

    if verified {
        let user = state
            .db
            .find_user_by_email(&body.email)
            .await
            .reject("could not find user")?
            .ok_or(AppError::Internal("user not found after verification"))?;

        let session = state
            .db
            .create_user_session(user.id)
            .await
            .reject("could not create session")?;

        let cookie = utils::cookie(
            names::USER_SESSION_COOKIE_NAME,
            &session,
            state.secure_cookies,
        );
        let quizzes = state
            .db
            .quizzes(user.id)
            .await
            .reject("could not get quizzes")?;

        let mut headers = HeaderMap::new();
        headers.insert(SET_COOKIE, cookie.parse().unwrap());

        Ok((
            headers,
            views::titled("Dashboard", homepage_views::dashboard(quizzes, &locale)),
        )
            .into_response())
    } else {
        Ok(views::titled(
            "Log In",
            homepage_views::login(homepage_views::LoginState::IncorrectPassword, &locale),
        )
        .into_response())
    }
}

async fn logout_post(jar: CookieJar, State(state): State<AppState>) -> impl IntoResponse {
    // Delete user session from DB
    if let Some(session_id) = jar
        .get(names::USER_SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
    {
        let _ = state.db.delete_user_session(&session_id).await;
    }

    // Clear both new and legacy session cookies
    let clear_user = utils::clear_cookie(names::USER_SESSION_COOKIE_NAME, state.secure_cookies);
    let clear_admin = utils::clear_cookie(names::ADMIN_SESSION_COOKIE_NAME, state.secure_cookies);
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, clear_user.parse().unwrap());
    headers.append(SET_COOKIE, clear_admin.parse().unwrap());
    headers.insert("HX-Redirect", "/".parse().unwrap());

    (headers, "")
}

async fn create_quiz(
    AuthGuard(user): AuthGuard,
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
        .load_quiz(quiz_name, questions, user.id)
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
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    axum::extract::Path(quiz_id): axum::extract::Path<i32>,
) -> Result<maud::Markup, AppError> {
    state
        .db
        .delete_quiz(quiz_id, user.id)
        .await
        .reject("failed to delete quiz")?;

    Ok(html!())
}

#[derive(Deserialize)]
struct SetLocaleBody {
    locale: String,
}

async fn set_locale(
    State(state): State<AppState>,
    Json(body): Json<SetLocaleBody>,
) -> Result<impl IntoResponse, AppError> {
    let locale = match body.locale.as_str() {
        "ja" => "ja",
        "zh-CN" => "zh-CN",
        "zh-TW" => "zh-TW",
        _ => "en",
    };
    let cookie = utils::cookie(names::LOCALE_COOKIE_NAME, locale, state.secure_cookies);
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    headers.insert("HX-Refresh", "true".parse().unwrap());

    Ok((headers, ""))
}
