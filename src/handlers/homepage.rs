use axum::{
    extract::{Form, State},
    http::{
        header::{LOCATION, SET_COOKIE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    extractors::{IsHtmx, Locale},
    names,
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
    use crate::services::auth::RegisterOutcome;

    let outcome = state
        .auth
        .register(&body.email, &body.password, &body.display_name)
        .await
        .reject("registration failed")?;

    match outcome {
        RegisterOutcome::LoggedIn(session_token) => {
            let cookie = utils::cookie(
                names::USER_SESSION_COOKIE_NAME,
                &session_token,
                state.secure_cookies,
            )
            .reject("could not build session cookie")?;
            Ok((
                StatusCode::SEE_OTHER,
                [
                    (SET_COOKIE, cookie),
                    (LOCATION, HeaderValue::from_static("/")),
                ],
                "",
            )
                .into_response())
        }
        RegisterOutcome::VerificationSent(email) => Ok(views::titled(
            "Check Your Email",
            homepage_views::check_email(&email, &locale),
        )
        .into_response()),
        RegisterOutcome::EmptyFields => Ok(views::page(
            "Register",
            homepage_views::register(homepage_views::RegisterState::EmptyFields, &locale),
            &locale,
        )
        .into_response()),
        RegisterOutcome::EmailTaken => Ok(views::page(
            "Register",
            homepage_views::register(homepage_views::RegisterState::EmailTaken, &locale),
            &locale,
        )
        .into_response()),
        RegisterOutcome::WeakPassword => Ok(views::page(
            "Register",
            homepage_views::register(homepage_views::RegisterState::WeakPassword, &locale),
            &locale,
        )
        .into_response()),
    }
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
    use crate::services::auth::LoginOutcome;

    let outcome = state
        .auth
        .login(&body.email, &body.password)
        .await
        .reject("login failed")?;

    match outcome {
        LoginOutcome::Success(session_token) => {
            let cookie = utils::cookie(
                names::USER_SESSION_COOKIE_NAME,
                &session_token,
                state.secure_cookies,
            )
            .reject("could not build session cookie")?;
            Ok((
                StatusCode::SEE_OTHER,
                [
                    (SET_COOKIE, cookie),
                    (LOCATION, HeaderValue::from_static("/")),
                ],
                "",
            )
                .into_response())
        }
        LoginOutcome::InvalidCredentials => Ok(views::page(
            "Log In",
            homepage_views::login(homepage_views::LoginState::IncorrectPassword, &locale),
            &locale,
        )
        .into_response()),
        LoginOutcome::EmailNotVerified => Ok(views::page(
            "Log In",
            homepage_views::login(homepage_views::LoginState::EmailNotVerified, &locale),
            &locale,
        )
        .into_response()),
    }
}

async fn logout_post(
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(session_id) = jar
        .get(names::USER_SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
    {
        let _ = state.auth.logout(&session_id).await;
    }

    // Clear both new and legacy session cookies
    let clear_user = utils::clear_cookie(names::USER_SESSION_COOKIE_NAME, state.secure_cookies)
        .reject("could not build clear-user cookie")?;
    let clear_admin = utils::clear_cookie(names::ADMIN_SESSION_COOKIE_NAME, state.secure_cookies)
        .reject("could not build clear-admin cookie")?;
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, clear_user);
    headers.append(SET_COOKIE, clear_admin);
    headers.insert("HX-Redirect", HeaderValue::from_static(names::LOGIN_URL));

    Ok((headers, ""))
}

async fn verify_email(
    State(state): State<AppState>,
    IsHtmx(is_htmx): IsHtmx,
    Locale(locale): Locale,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> Result<maud::Markup, AppError> {
    let verified = state
        .auth
        .verify_email(&token)
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
    if !state.auth.email_enabled() {
        return Err(AppError::Input("email verification not configured"));
    }

    state
        .auth
        .resend_verification(&body.email)
        .await
        .reject("could not resend verification")?;

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
    let sent = state
        .auth
        .forgot_password(&body.email)
        .await
        .reject("could not process password reset")?;

    let fp_state = if sent {
        homepage_views::ForgotPasswordState::EmailSent
    } else {
        homepage_views::ForgotPasswordState::EmailNotConfigured
    };

    Ok(views::titled(
        "Forgot Password",
        homepage_views::forgot_password(fp_state, &locale),
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
        .auth
        .validate_reset_token(&token)
        .await
        .reject("could not validate reset token")?;

    let rp_state = if valid {
        homepage_views::ResetPasswordState::Form
    } else {
        homepage_views::ResetPasswordState::InvalidToken
    };

    let token_str = if valid { &token } else { "" };

    Ok(views::render(
        is_htmx,
        "Reset Password",
        homepage_views::reset_password(rp_state, token_str, &locale),
        &locale,
        None,
    ))
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
    use crate::services::auth::ResetPasswordOutcome;

    let outcome = state
        .auth
        .reset_password(&body.token, &body.password)
        .await
        .reject("could not reset password")?;

    let (rp_state, token_str) = match outcome {
        ResetPasswordOutcome::Success => (homepage_views::ResetPasswordState::Success, ""),
        ResetPasswordOutcome::EmptyPassword => (
            homepage_views::ResetPasswordState::EmptyPassword,
            body.token.as_str(),
        ),
        ResetPasswordOutcome::WeakPassword => (
            homepage_views::ResetPasswordState::WeakPassword,
            body.token.as_str(),
        ),
        ResetPasswordOutcome::InvalidToken => {
            (homepage_views::ResetPasswordState::InvalidToken, "")
        }
    };

    Ok(views::titled(
        "Reset Password",
        homepage_views::reset_password(rp_state, token_str, &locale),
    )
    .into_response())
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
    let cookie = utils::cookie(names::LOCALE_COOKIE_NAME, locale, state.secure_cookies)
        .reject("could not build locale cookie")?;
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie);
    headers.insert("HX-Refresh", HeaderValue::from_static("true"));

    Ok((headers, ""))
}
