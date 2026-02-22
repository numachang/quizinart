use std::collections::HashMap;

use axum::{
    extract::{Form, Multipart, State},
    http::{
        header::{LOCATION, SET_COOKIE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use maud::html;
use serde::Deserialize;

use crate::{
    extractors::{AuthGuard, IsHtmx, Locale},
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
        .route("/login", get(login_page).post(login_post))
        .route("/logout", post(logout_post))
        .route("/verify-email/{token}", get(verify_email))
        .route("/resend-verification", post(resend_verification))
        .route(
            "/forgot-password",
            get(forgot_password_page).post(forgot_password_post),
        )
        .route("/reset-password/{token}", get(reset_password_page))
        .route("/reset-password", post(reset_password_post))
        .route("/create-quiz", post(create_quiz))
        .route("/delete-quiz/{id}", delete(delete_quiz))
        .route("/set-locale", post(set_locale))
}

async fn register_page(IsHtmx(is_htmx): IsHtmx, Locale(locale): Locale) -> maud::Markup {
    views::render(
        is_htmx,
        "Register",
        homepage_views::register(homepage_views::RegisterState::NoError, &locale),
        &locale,
        None,
    )
}

async fn homepage(
    State(state): State<AppState>,
    jar: CookieJar,
    IsHtmx(is_htmx): IsHtmx,
    Locale(locale): Locale,
) -> Result<axum::response::Response, AppError> {
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
            return Ok(views::render(
                is_htmx,
                "My Quizzes",
                homepage_views::quiz_list(quizzes, &locale),
                &locale,
                Some(&user.display_name),
            )
            .into_response());
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
                return Ok(views::render(
                    is_htmx,
                    "My Quizzes",
                    homepage_views::quiz_list(quizzes, &locale),
                    &locale,
                    Some(&user.display_name),
                )
                .into_response());
            }
        }
    }

    // Not logged in: redirect to login page
    Ok(Redirect::to(names::LOGIN_URL).into_response())
}

async fn login_page(IsHtmx(is_htmx): IsHtmx, Locale(locale): Locale) -> maud::Markup {
    views::render(
        is_htmx,
        "Log In",
        homepage_views::login(homepage_views::LoginState::NoError, &locale),
        &locale,
        None,
    )
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
    Form(body): Form<RegisterPost>,
) -> Result<axum::response::Response, AppError> {
    // Validate inputs
    if body.email.is_empty() || body.password.is_empty() || body.display_name.is_empty() {
        return Ok(views::page(
            "Register",
            homepage_views::register(homepage_views::RegisterState::EmptyFields, &locale),
            &locale,
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
        return Ok(views::page(
            "Register",
            homepage_views::register(homepage_views::RegisterState::EmailTaken, &locale),
            &locale,
        )
        .into_response());
    }

    // If no Resend API key, skip verification (local dev mode)
    if state.resend_api_key.is_empty() {
        let user_id = state
            .db
            .create_user(&body.email, &body.password, &body.display_name)
            .await
            .reject("could not create user")?;

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

        return Ok((
            StatusCode::SEE_OTHER,
            [
                (SET_COOKIE, cookie.parse::<HeaderValue>().unwrap()),
                (LOCATION, HeaderValue::from_static("/")),
            ],
            "",
        )
            .into_response());
    }

    // Email verification flow
    let (_user_id, token) = state
        .db
        .create_unverified_user(&body.email, &body.password, &body.display_name)
        .await
        .reject("could not create user")?;

    let verification_url = format!("{}/verify-email/{}", state.base_url, token);

    if let Err(e) =
        crate::email::send_verification_email(&state.resend_api_key, &body.email, &verification_url)
            .await
    {
        tracing::error!("failed to send verification email to {}: {e}", body.email);
    }

    Ok(views::titled(
        "Check Your Email",
        homepage_views::check_email(&body.email, &locale),
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
    Form(body): Form<LoginPost>,
) -> Result<axum::response::Response, AppError> {
    let verified = state
        .db
        .verify_user_password(&body.email, &body.password)
        .await
        .reject("could not verify password")?;

    if verified {
        // Check email verification (skip if no API key configured)
        if !state.resend_api_key.is_empty() {
            let email_verified = state
                .db
                .is_email_verified(&body.email)
                .await
                .reject("could not check email verification")?;

            if !email_verified {
                return Ok(views::page(
                    "Log In",
                    homepage_views::login(homepage_views::LoginState::EmailNotVerified, &locale),
                    &locale,
                )
                .into_response());
            }
        }

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

        Ok((
            StatusCode::SEE_OTHER,
            [
                (SET_COOKIE, cookie.parse::<HeaderValue>().unwrap()),
                (LOCATION, HeaderValue::from_static("/")),
            ],
            "",
        )
            .into_response())
    } else {
        Ok(views::page(
            "Log In",
            homepage_views::login(homepage_views::LoginState::IncorrectPassword, &locale),
            &locale,
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
    headers.insert("HX-Redirect", names::LOGIN_URL.parse().unwrap());

    (headers, "")
}

async fn verify_email(
    State(state): State<AppState>,
    IsHtmx(is_htmx): IsHtmx,
    Locale(locale): Locale,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> Result<maud::Markup, AppError> {
    let verified = state
        .db
        .verify_email_token(&token)
        .await
        .reject("could not verify email token")?;

    if verified {
        Ok(views::render(
            is_htmx,
            "Email Verified",
            homepage_views::email_verified(&locale),
            &locale,
            None,
        ))
    } else {
        Ok(views::render(
            is_htmx,
            "Verification Failed",
            homepage_views::verification_failed(&locale),
            &locale,
            None,
        ))
    }
}

#[derive(Deserialize)]
struct ResendVerificationPost {
    email: String,
}

async fn resend_verification(
    State(state): State<AppState>,
    Locale(locale): Locale,
    Json(body): Json<ResendVerificationPost>,
) -> Result<axum::response::Response, AppError> {
    if state.resend_api_key.is_empty() {
        return Err(AppError::Input("email verification not configured"));
    }

    let token = state
        .db
        .regenerate_verification_token(&body.email)
        .await
        .reject("could not regenerate token")?;

    if let Some(token) = token {
        let verification_url = format!("{}/verify-email/{}", state.base_url, token);
        if let Err(e) = crate::email::send_verification_email(
            &state.resend_api_key,
            &body.email,
            &verification_url,
        )
        .await
        {
            tracing::error!("failed to resend verification email: {e}");
        }
    }

    // Always show success (don't leak whether email exists)
    Ok(views::titled(
        "Check Your Email",
        homepage_views::check_email(&body.email, &locale),
    )
    .into_response())
}

async fn forgot_password_page(IsHtmx(is_htmx): IsHtmx, Locale(locale): Locale) -> maud::Markup {
    views::render(
        is_htmx,
        "Forgot Password",
        homepage_views::forgot_password(homepage_views::ForgotPasswordState::NoError, &locale),
        &locale,
        None,
    )
}

#[derive(Deserialize)]
struct ForgotPasswordPost {
    email: String,
}

async fn forgot_password_post(
    State(state): State<AppState>,
    Locale(locale): Locale,
    Json(body): Json<ForgotPasswordPost>,
) -> Result<axum::response::Response, AppError> {
    if state.resend_api_key.is_empty() {
        return Ok(views::titled(
            "Forgot Password",
            homepage_views::forgot_password(
                homepage_views::ForgotPasswordState::EmailNotConfigured,
                &locale,
            ),
        )
        .into_response());
    }

    let token = state
        .db
        .create_password_reset_token(&body.email)
        .await
        .reject("could not create reset token")?;

    if let Some(token) = token {
        let reset_url = format!("{}/reset-password/{}", state.base_url, token);
        if let Err(e) =
            crate::email::send_password_reset_email(&state.resend_api_key, &body.email, &reset_url)
                .await
        {
            tracing::error!("failed to send password reset email to {}: {e}", body.email);
        }
    }

    // Always show "check your email" regardless of whether email exists
    Ok(views::titled(
        "Forgot Password",
        homepage_views::forgot_password(homepage_views::ForgotPasswordState::EmailSent, &locale),
    )
    .into_response())
}

async fn reset_password_page(
    State(state): State<AppState>,
    IsHtmx(is_htmx): IsHtmx,
    Locale(locale): Locale,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> Result<maud::Markup, AppError> {
    let valid = state
        .db
        .validate_password_reset_token(&token)
        .await
        .reject("could not validate reset token")?;

    if valid.is_some() {
        Ok(views::render(
            is_htmx,
            "Reset Password",
            homepage_views::reset_password(
                homepage_views::ResetPasswordState::Form,
                &token,
                &locale,
            ),
            &locale,
            None,
        ))
    } else {
        Ok(views::render(
            is_htmx,
            "Reset Password",
            homepage_views::reset_password(
                homepage_views::ResetPasswordState::InvalidToken,
                "",
                &locale,
            ),
            &locale,
            None,
        ))
    }
}

#[derive(Deserialize)]
struct ResetPasswordPost {
    token: String,
    password: String,
}

async fn reset_password_post(
    State(state): State<AppState>,
    Locale(locale): Locale,
    Json(body): Json<ResetPasswordPost>,
) -> Result<axum::response::Response, AppError> {
    if body.password.is_empty() {
        return Ok(views::titled(
            "Reset Password",
            homepage_views::reset_password(
                homepage_views::ResetPasswordState::EmptyPassword,
                &body.token,
                &locale,
            ),
        )
        .into_response());
    }

    let success = state
        .db
        .reset_password_with_token(&body.token, &body.password)
        .await
        .reject("could not reset password")?;

    if success {
        Ok(views::titled(
            "Reset Password",
            homepage_views::reset_password(
                homepage_views::ResetPasswordState::Success,
                "",
                &locale,
            ),
        )
        .into_response())
    } else {
        Ok(views::titled(
            "Reset Password",
            homepage_views::reset_password(
                homepage_views::ResetPasswordState::InvalidToken,
                "",
                &locale,
            ),
        )
        .into_response())
    }
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
