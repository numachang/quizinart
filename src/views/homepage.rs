use crate::{db::Quiz, names, views::components};
use maud::{html, Markup};
use rust_i18n::t;

pub enum RegisterState {
    NoError,
    EmailTaken,
    EmptyFields,
}

pub fn register(state: RegisterState, locale: &str) -> Markup {
    let error_msg = match state {
        RegisterState::NoError => None,
        RegisterState::EmailTaken => Some(t!("homepage.email_taken", locale = locale).to_string()),
        RegisterState::EmptyFields => {
            Some(t!("homepage.empty_fields", locale = locale).to_string())
        }
    };

    html! {
        h1 { (t!("homepage.register_title", locale = locale)) }
        p { (t!("homepage.register_desc", locale = locale)) }
        article style="width: fit-content;" {
            form hx-post=(names::REGISTER_URL)
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
                a href="/" { (t!("homepage.log_in", locale = locale)) }
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
            form hx-post=(names::LOGIN_URL)
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
                a href="/" { (t!("homepage.back_to_login", locale = locale)) }
            }
        }
    }
}

pub fn email_verified(locale: &str) -> Markup {
    html! {
        h1 { (t!("homepage.email_verified_title", locale = locale)) }
        p { (t!("homepage.email_verified_desc", locale = locale)) }
        p {
            a href="/" { (t!("homepage.log_in", locale = locale)) }
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
                    a href="/" { (t!("homepage.back_to_login", locale = locale)) }
                }
            }
        },
        ForgotPasswordState::EmailNotConfigured => html! {
            h1 { (t!("homepage.forgot_password_title", locale = locale)) }
            p { (t!("homepage.forgot_password_not_configured", locale = locale)) }
            p {
                a href="/" { (t!("homepage.back_to_login", locale = locale)) }
            }
        },
        ForgotPasswordState::EmailSent => html! {
            h1 { (t!("homepage.forgot_password_title", locale = locale)) }
            p { (t!("homepage.forgot_password_email_sent", locale = locale)) }
            p { (t!("homepage.forgot_password_email_sent_hint", locale = locale)) }
            p {
                a href="/" { (t!("homepage.back_to_login", locale = locale)) }
            }
        },
    }
}

pub enum ResetPasswordState {
    Form,
    InvalidToken,
    EmptyPassword,
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
                a href="/" { (t!("homepage.log_in", locale = locale)) }
            }
        },
    }
}

pub fn dashboard(quizzes: Vec<Quiz>, locale: &str) -> Markup {
    html! {
        h1 { (t!("homepage.dashboard", locale = locale)) }

        article style="width: fit-content;" {
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
                              aria-describedby="quiz-name-helper"
                              aria-label=(t!("homepage.quiz_name", locale = locale));
                        small id="quiz-name-helper" { (t!("homepage.quiz_name_hint", locale = locale)) }
                    }

                    label {
                        (t!("homepage.quiz_file", locale = locale))
                        input name="quiz_file"
                              type="file"
                              required="true"
                              aria-describedby="quiz-file-helper"
                              accept="application/json"
                              aria-label=(t!("homepage.quiz_file", locale = locale));
                        small id="quiz-file-helper" { (t!("homepage.quiz_file_hint", locale = locale)) }
                    }

                    input type="submit" value=(t!("homepage.create", locale = locale));
            }
        }

        div."quiz-grid" {
            @for quiz in quizzes {
                article {
                    h3 { (quiz.name) }
                    p { (quiz.count) (t!("homepage.questions_suffix", locale = locale)) }
                    div role="group" {
                        button
                            hx-trigger="click"
                            hx-target="main"
                            hx-swap="innerHTML"
                            hx-push-url="true"
                            hx-get=(names::quiz_dashboard_url(quiz.id)) { (t!("homepage.view", locale = locale)) }
                        button."contrast"
                            hx-trigger="click"
                            hx-target="closest article"
                            hx-swap="outerHTML"
                            hx-confirm=(t!("homepage.delete_confirm", locale = locale))
                            hx-delete=(names::delete_quiz_url(quiz.id)) { (t!("homepage.delete", locale = locale)) }
                    }
                }
            }
        }
    }
}
