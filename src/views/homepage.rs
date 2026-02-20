use crate::{db::Quiz, names};
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
                 hx-disabled-elt="find input, find button"
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
                 hx-disabled-elt="find input, find button"
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
                    }
                }
                button type="submit" { (t!("homepage.log_in", locale = locale)) }
            }
            p {
                (t!("homepage.no_account", locale = locale))
                " "
                a href="/register"
                  hx-get="/register"
                  hx-target="main"
                  hx-swap="innerHTML" {
                    (t!("homepage.register_btn", locale = locale))
                }
            }
        }
    }
}

pub fn dashboard(quizzes: Vec<Quiz>, locale: &str) -> Markup {
    html! {
        div style="display: flex; justify-content: space-between; align-items: center;" {
            h1 { (t!("homepage.dashboard", locale = locale)) }
            button."contrast outline"
                   hx-post=(names::LOGOUT_URL)
                   hx-swap="none" {
                (t!("homepage.logout", locale = locale))
            }
        }

        article style="width: fit-content;" {
            form hx-post=(names::CREATE_QUIZ_URL)
                 hx-target="main"
                 enctype="multipart/form-data"
                 hx-disabled-elt="find input[type='text'], find input[type='file'], find input[type='submit']"
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
                            hx-disabled-elt="this"
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
