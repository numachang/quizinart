use crate::{db::Quiz, names, views::components, views::quiz as quiz_views};
use maud::{html, Markup};
use rust_i18n::t;

pub fn landing_page(locale: &str) -> Markup {
    html! {
        // Hero section
        section.landing-hero {
            h1 { (t!("landing.tagline", locale = locale)) }
            p.landing-hero-desc { (t!("landing.description", locale = locale)) }
            div.landing-cta {
                a role="button" href=(names::REGISTER_URL) {
                    (t!("landing.sign_up", locale = locale))
                }
                a role="button" href=(names::LOGIN_URL) class="outline" {
                    (t!("landing.log_in", locale = locale))
                }
            }
        }

        // Features section
        section.landing-features {
            h2 { (t!("landing.features_title", locale = locale)) }
            div.landing-features-grid {
                article.landing-feature-card {
                    span."material-symbols-rounded landing-feature-icon" { "tune" }
                    h3 { (t!("landing.feature_modes_title", locale = locale)) }
                    p { (t!("landing.feature_modes_desc", locale = locale)) }
                }
                article.landing-feature-card {
                    span."material-symbols-rounded landing-feature-icon" { "bookmark" }
                    h3 { (t!("landing.feature_bookmark_title", locale = locale)) }
                    p { (t!("landing.feature_bookmark_desc", locale = locale)) }
                }
                article.landing-feature-card {
                    span."material-symbols-rounded landing-feature-icon" { "history" }
                    h3 { (t!("landing.feature_sessions_title", locale = locale)) }
                    p { (t!("landing.feature_sessions_desc", locale = locale)) }
                }
                article.landing-feature-card {
                    span."material-symbols-rounded landing-feature-icon" { "analytics" }
                    h3 { (t!("landing.feature_dashboard_title", locale = locale)) }
                    p { (t!("landing.feature_dashboard_desc", locale = locale)) }
                }
                article.landing-feature-card {
                    span."material-symbols-rounded landing-feature-icon" { "store" }
                    h3 { (t!("landing.feature_marketplace_title", locale = locale)) }
                    p { (t!("landing.feature_marketplace_desc", locale = locale)) }
                }
                article.landing-feature-card {
                    span."material-symbols-rounded landing-feature-icon" { "translate" }
                    h3 { (t!("landing.feature_i18n_title", locale = locale)) }
                    p { (t!("landing.feature_i18n_desc", locale = locale)) }
                }
            }
        }

        // Bottom CTA
        section.landing-bottom-cta {
            h2 { (t!("landing.bottom_cta_title", locale = locale)) }
            p { (t!("landing.bottom_cta_desc", locale = locale)) }
            a role="button" href=(names::REGISTER_URL) {
                (t!("landing.sign_up", locale = locale))
            }
        }
    }
}

pub enum RegisterState {
    NoError,
    EmailTaken,
    EmptyFields,
    WeakPassword,
}

pub fn register(state: RegisterState, locale: &str) -> Markup {
    let error_msg = match state {
        RegisterState::NoError => None,
        RegisterState::EmailTaken => Some(t!("homepage.email_taken", locale = locale).to_string()),
        RegisterState::EmptyFields => {
            Some(t!("homepage.empty_fields", locale = locale).to_string())
        }
        RegisterState::WeakPassword => {
            Some(t!("homepage.weak_password", locale = locale).to_string())
        }
    };

    html! {
        h1 { (t!("homepage.register_title", locale = locale)) }
        p { (t!("homepage.register_desc", locale = locale)) }
        article style="width: fit-content;" {
            form action=(names::REGISTER_URL) method="post" {
                label {
                    (t!("homepage.email", locale = locale))
                    input name="email"
                          type="email"
                          autocomplete="email"
                          required="true"
                          placeholder=(t!("homepage.email", locale = locale))
                          aria-label=(t!("homepage.email", locale = locale));
                }
                label {
                    (t!("homepage.display_name", locale = locale))
                    input name="display_name"
                          type="text"
                          autocomplete="name"
                          required="true"
                          placeholder=(t!("homepage.display_name", locale = locale))
                          aria-label=(t!("homepage.display_name", locale = locale));
                }
                label {
                    (t!("homepage.password", locale = locale))
                    @if let Some(ref msg) = error_msg {
                        input name="password"
                              type="password"
                              autocomplete="new-password"
                              required="true"
                              placeholder=(t!("homepage.password", locale = locale))
                              aria-invalid="true"
                              aria-label=(t!("homepage.password", locale = locale));
                        small { (msg) }
                    } @else {
                        input name="password"
                              type="password"
                              autocomplete="new-password"
                              required="true"
                              placeholder=(t!("homepage.password", locale = locale))
                              aria-label=(t!("homepage.password", locale = locale));
                    }
                }
                button type="submit" { (t!("homepage.register_btn", locale = locale)) }
            }
            p {
                (t!("homepage.already_have_account", locale = locale))
                " "
                a href=(names::LOGIN_URL) { (t!("homepage.log_in", locale = locale)) }
            }
        }
    }
}

pub enum LoginState {
    NoError,
    IncorrectPassword,
    EmailNotVerified,
}

pub fn login(state: LoginState, locale: &str) -> Markup {
    html! {
        h1 { (t!("homepage.welcome_back", locale = locale)) }
        p {
            (t!("homepage.login_desc", locale = locale))
        }
        article style="width: fit-content;" {
            form action=(names::LOGIN_URL) method="post" {
                label {
                    (t!("homepage.email", locale = locale))
                    input name="email"
                          type="email"
                          autocomplete="email"
                          required="true"
                          placeholder=(t!("homepage.email", locale = locale))
                          aria-label=(t!("homepage.email", locale = locale));
                }
                @match state {
                    LoginState::NoError => {
                        label {
                            (t!("homepage.password", locale = locale))
                            input name="password"
                                  type="password"
                                  autocomplete="current-password"
                                  required="true"
                                  placeholder=(t!("homepage.password", locale = locale))
                                  aria-label=(t!("homepage.password", locale = locale));
                        }
                    },
                    LoginState::IncorrectPassword => {
                        label {
                            (t!("homepage.password", locale = locale))
                            input name="password"
                                  type="password"
                                  autocomplete="current-password"
                                  required="true"
                                  placeholder=(t!("homepage.password", locale = locale))
                                  aria-invalid="true"
                                  aria-label=(t!("homepage.password", locale = locale));
                            small { (t!("homepage.incorrect_password", locale = locale)) }
                        }
                    },
                    LoginState::EmailNotVerified => {
                        label {
                            (t!("homepage.password", locale = locale))
                            input name="password"
                                  type="password"
                                  autocomplete="current-password"
                                  required="true"
                                  placeholder=(t!("homepage.password", locale = locale))
                                  aria-invalid="true"
                                  aria-label=(t!("homepage.password", locale = locale));
                            small { (t!("homepage.email_not_verified", locale = locale)) }
                        }
                    }
                }
                p style="margin-bottom: 0.5rem; font-size: 0.85rem;" {
                    (components::nav_link(
                        names::FORGOT_PASSWORD_URL,
                        html! { (t!("homepage.forgot_password", locale = locale)) },
                    ))
                }
                button type="submit" { (t!("homepage.log_in", locale = locale)) }
            }
            p {
                (t!("homepage.no_account", locale = locale))
                " "
                (components::nav_link(
                    "/register",
                    html! { (t!("homepage.register_btn", locale = locale)) },
                ))
            }
        }
    }
}

pub fn check_email(email: &str, locale: &str) -> Markup {
    html! {
        h1 { (t!("homepage.check_email_title", locale = locale)) }
        p { (t!("homepage.check_email_desc", locale = locale)) }
        p { strong { (email) } }
        p { (t!("homepage.check_email_hint", locale = locale)) }
        article style="width: fit-content;" {
            form hx-post=(names::RESEND_VERIFICATION_URL)
                 hx-ext="json-enc"
                 hx-target="main"
                 hx-swap="innerHTML" {
                input type="hidden" name="email" value=(email);
                button type="submit" class="outline" {
                    (t!("homepage.resend_email", locale = locale))
                }
            }
            p {
                a href=(names::LOGIN_URL) { (t!("homepage.back_to_login", locale = locale)) }
            }
        }
    }
}

pub fn email_verified(locale: &str) -> Markup {
    html! {
        h1 { (t!("homepage.email_verified_title", locale = locale)) }
        p { (t!("homepage.email_verified_desc", locale = locale)) }
        p {
            a href=(names::LOGIN_URL) { (t!("homepage.log_in", locale = locale)) }
        }
    }
}

pub fn verification_failed(locale: &str) -> Markup {
    html! {
        h1 { (t!("homepage.verification_failed_title", locale = locale)) }
        p { (t!("homepage.verification_failed_desc", locale = locale)) }
        p {
            a href="/register" { (t!("homepage.register_btn", locale = locale)) }
        }
    }
}

pub enum ForgotPasswordState {
    NoError,
    EmailNotConfigured,
    EmailSent,
}

pub fn forgot_password(state: ForgotPasswordState, locale: &str) -> Markup {
    match state {
        ForgotPasswordState::NoError => html! {
            h1 { (t!("homepage.forgot_password_title", locale = locale)) }
            p { (t!("homepage.forgot_password_desc", locale = locale)) }
            article style="width: fit-content;" {
                form hx-post=(names::FORGOT_PASSWORD_URL)
                     hx-ext="json-enc"
                     hx-target="main"

                     hx-swap="innerHTML" {
                    label {
                        (t!("homepage.email", locale = locale))
                        input name="email"
                              type="email"
                              autocomplete="email"
                              required="true"
                              placeholder=(t!("homepage.email", locale = locale))
                              aria-label=(t!("homepage.email", locale = locale));
                    }
                    button type="submit" { (t!("homepage.forgot_password_btn", locale = locale)) }
                }
                p {
                    a href=(names::LOGIN_URL) { (t!("homepage.back_to_login", locale = locale)) }
                }
            }
        },
        ForgotPasswordState::EmailNotConfigured => html! {
            h1 { (t!("homepage.forgot_password_title", locale = locale)) }
            p { (t!("homepage.forgot_password_not_configured", locale = locale)) }
            p {
                a href=(names::LOGIN_URL) { (t!("homepage.back_to_login", locale = locale)) }
            }
        },
        ForgotPasswordState::EmailSent => html! {
            h1 { (t!("homepage.forgot_password_title", locale = locale)) }
            p { (t!("homepage.forgot_password_email_sent", locale = locale)) }
            p { (t!("homepage.forgot_password_email_sent_hint", locale = locale)) }
            p {
                a href=(names::LOGIN_URL) { (t!("homepage.back_to_login", locale = locale)) }
            }
        },
    }
}

pub enum ResetPasswordState {
    Form,
    InvalidToken,
    EmptyPassword,
    WeakPassword,
    Success,
}

pub fn reset_password(state: ResetPasswordState, token: &str, locale: &str) -> Markup {
    match state {
        ResetPasswordState::Form => html! {
            h1 { (t!("homepage.reset_password_title", locale = locale)) }
            p { (t!("homepage.reset_password_desc", locale = locale)) }
            article style="width: fit-content;" {
                form hx-post=(names::RESET_PASSWORD_URL)
                     hx-ext="json-enc"
                     hx-target="main"

                     hx-swap="innerHTML" {
                    input type="hidden" name="token" value=(token);
                    label {
                        (t!("homepage.new_password", locale = locale))
                        input name="password"
                              type="password"
                              autocomplete="new-password"
                              required="true"
                              placeholder=(t!("homepage.new_password", locale = locale))
                              aria-label=(t!("homepage.new_password", locale = locale));
                    }
                    button type="submit" { (t!("homepage.reset_password_btn", locale = locale)) }
                }
            }
        },
        ResetPasswordState::EmptyPassword => html! {
            h1 { (t!("homepage.reset_password_title", locale = locale)) }
            p { (t!("homepage.reset_password_desc", locale = locale)) }
            article style="width: fit-content;" {
                form hx-post=(names::RESET_PASSWORD_URL)
                     hx-ext="json-enc"
                     hx-target="main"

                     hx-swap="innerHTML" {
                    input type="hidden" name="token" value=(token);
                    label {
                        (t!("homepage.new_password", locale = locale))
                        input name="password"
                              type="password"
                              autocomplete="new-password"
                              required="true"
                              placeholder=(t!("homepage.new_password", locale = locale))
                              aria-invalid="true"
                              aria-label=(t!("homepage.new_password", locale = locale));
                        small { (t!("homepage.empty_fields", locale = locale)) }
                    }
                    button type="submit" { (t!("homepage.reset_password_btn", locale = locale)) }
                }
            }
        },
        ResetPasswordState::WeakPassword => html! {
            h1 { (t!("homepage.reset_password_title", locale = locale)) }
            p { (t!("homepage.reset_password_desc", locale = locale)) }
            article style="width: fit-content;" {
                form hx-post=(names::RESET_PASSWORD_URL)
                     hx-ext="json-enc"
                     hx-target="main"

                     hx-swap="innerHTML" {
                    input type="hidden" name="token" value=(token);
                    label {
                        (t!("homepage.new_password", locale = locale))
                        input name="password"
                              type="password"
                              autocomplete="new-password"
                              required="true"
                              placeholder=(t!("homepage.new_password", locale = locale))
                              aria-invalid="true"
                              aria-label=(t!("homepage.new_password", locale = locale));
                        small { (t!("homepage.weak_password", locale = locale)) }
                    }
                    button type="submit" { (t!("homepage.reset_password_btn", locale = locale)) }
                }
            }
        },
        ResetPasswordState::InvalidToken => html! {
            h1 { (t!("homepage.reset_token_invalid_title", locale = locale)) }
            p { (t!("homepage.reset_token_invalid_desc", locale = locale)) }
            p {
                a href=(names::FORGOT_PASSWORD_URL) { (t!("homepage.forgot_password_btn", locale = locale)) }
            }
        },
        ResetPasswordState::Success => html! {
            h1 { (t!("homepage.reset_password_success_title", locale = locale)) }
            p { (t!("homepage.reset_password_success_desc", locale = locale)) }
            p {
                a href=(names::LOGIN_URL) { (t!("homepage.log_in", locale = locale)) }
            }
        },
    }
}

pub fn quiz_list(quizzes: Vec<Quiz>, locale: &str) -> Markup {
    quiz_list_with_error(quizzes, locale, None)
}

pub fn quiz_list_with_error(quizzes: Vec<Quiz>, locale: &str, error: Option<&str>) -> Markup {
    html! {
        h1 { (t!("homepage.my_quizzes", locale = locale)) }

        @if let Some(msg) = error {
            article style="border-left: 4px solid #dc3545; padding: 0.75rem 1rem; margin-bottom: 1rem;" {
                p style="margin: 0; color: #dc3545;" { (msg) }
            }
        }

        div."quiz-grid" {
            @for quiz in quizzes {
                article."quiz-card" {
                    h3 style="display: flex; align-items: center; gap: 0.4rem;" {
                        a hx-get=(names::quiz_dashboard_url(&quiz.public_id))
                          hx-push-url="true"
                          hx-target="main"
                          href="#"
                          style="text-decoration: none; color: inherit;" {
                            (quiz.name)
                        }
                        span."card-actions material-symbols-rounded"
                             data-rename-name=(quiz.name)
                             data-rename-url=(names::rename_quiz_url(&quiz.public_id))
                             title=(t!("homepage.rename", locale = locale))
                             style="cursor: pointer; font-size: 0.7em; opacity: 0.4; transition: opacity 0.15s;" {
                            "edit"
                        }
                    }
                    p {
                        (quiz.count) (t!("homepage.questions_suffix", locale = locale))
                        @if !quiz.is_owner {
                            " · "
                            small { (t!("marketplace.by", locale = locale, owner = &quiz.owner_name)) }
                        }
                    }
                    div."card-actions" style="display: flex; justify-content: flex-end; gap: 0.75rem;" {
                        @if quiz.is_owner {
                            (quiz_views::share_toggle_icon(&quiz.public_id, quiz.is_shared, locale))
                        }
                        @if quiz.is_owner {
                            a."material-symbols-rounded"
                              hx-delete=(names::delete_quiz_url(&quiz.public_id))
                              hx-target="main"
                              hx-swap="innerHTML"
                              hx-confirm=(t!("homepage.delete_confirm", locale = locale))
                              title=(t!("homepage.delete", locale = locale))
                              style="cursor: pointer; color: var(--pico-del-color, #dc3545); font-size: 1.2rem; opacity: 0.5; transition: opacity 0.15s;" {
                                "delete"
                            }
                        } @else {
                            a."material-symbols-rounded"
                              hx-delete=(names::delete_quiz_url(&quiz.public_id))
                              hx-target="main"
                              hx-swap="innerHTML"
                              hx-confirm=(t!("homepage.remove_confirm", locale = locale))
                              title=(t!("homepage.remove_from_library", locale = locale))
                              style="cursor: pointer; color: var(--pico-del-color, #dc3545); font-size: 1.2rem; opacity: 0.5; transition: opacity 0.15s;" {
                                "playlist_remove"
                            }
                        }
                    }
                }
            }

            // Import from Marketplace card
            article."quiz-card" id="marketplace-card"
                   style="display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 120px; opacity: 0.6;" {
                h3 style="margin: 0;" {
                    a href=(names::MARKETPLACE_URL)
                      hx-get=(names::MARKETPLACE_URL)
                      hx-push-url="true"
                      hx-target="main"
                      style="text-decoration: none; color: inherit; display: flex; flex-direction: column; align-items: center;" {
                        span."material-symbols-rounded" style="font-size: 2.5rem;" { "store" }
                    }
                }
                p style="margin: 0.5rem 0 0; display: flex; align-items: center; gap: 0.2rem;" {
                    span."material-symbols-rounded" style="font-size: 1em;" { "add" }
                    (t!("homepage.from_marketplace", locale = locale))
                }
            }

            // Import from JSON file card
            article."quiz-card" id="upload-card"
                   style="display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 120px; opacity: 0.6;" {
                h3 style="margin: 0;" {
                    a href="#"
                      data-dialog-open="create-dialog"
                      style="text-decoration: none; color: inherit; display: flex; flex-direction: column; align-items: center;" {
                        span."material-symbols-rounded" style="font-size: 2.5rem;" { "upload_file" }
                    }
                }
                p style="margin: 0.5rem 0 0; display: flex; align-items: center; gap: 0.2rem;" {
                    span."material-symbols-rounded" style="font-size: 1em;" { "add" }
                    (t!("homepage.from_file", locale = locale))
                }
            }
        }

        dialog id="create-dialog" {
            article {
                header {
                    button aria-label="Close" rel="prev"
                           data-dialog-close="create-dialog" {}
                    h3 { (t!("homepage.import_quiz", locale = locale)) }
                }
                form hx-post=(names::CREATE_QUIZ_URL)
                     hx-target="main"
                     enctype="multipart/form-data"
                     hx-swap="innerHTML" {
                    label {
                        (t!("homepage.quiz_name", locale = locale))
                        input name="quiz_name"
                              type="text"
                              required="true"
                              autocomplete="off"
                              placeholder=(t!("homepage.quiz_name", locale = locale))
                              aria-label=(t!("homepage.quiz_name", locale = locale));
                    }
                    label {
                        (t!("homepage.quiz_file", locale = locale))
                        input name="quiz_file"
                              type="file"
                              required="true"
                              accept="application/json"
                              aria-label=(t!("homepage.quiz_file", locale = locale));
                    }
                    input type="submit" value=(t!("homepage.create", locale = locale));
                }
            }
        }

        dialog id="rename-dialog" {
            article {
                p { (t!("homepage.rename_prompt", locale = locale)) }
                input id="rename-input" type="text" required="true" autocomplete="off";
                input id="rename-url" type="hidden";
                footer style="display: flex; gap: 0.5rem; justify-content: flex-end;" {
                    button data-dialog-close="rename-dialog"
                           class="secondary" {
                        (t!("quiz.abandon_cancel", locale = locale))
                    }
                    button data-rename-submit="" {
                        (t!("homepage.rename", locale = locale))
                    }
                }
            }
        }
    }
}
