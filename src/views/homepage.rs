use crate::{db::Quiz, names};
use maud::{html, Markup};
use rust_i18n::t;

pub fn get_started(locale: &str) -> Markup {
    html! {
        h1 { (t!("homepage.welcome", locale = locale)) }
        p {
            (t!("homepage.welcome_desc_1", locale = locale))
            mark { (t!("homepage.quizinart", locale = locale)) }
            (t!("homepage.welcome_desc_2", locale = locale))
            strong { (t!("homepage.admin_password", locale = locale)) }
            (t!("homepage.welcome_desc_3", locale = locale))
        }
        article style="width: fit-content;" {
            form hx-post=(names::GET_STARTED_URL)
                 hx-ext="json-enc"
                 hx-target="main"
                 hx-disabled-elt="find input[type='password'], find input[type='submit']"
                 hx-swap="innerHTML" {
                label {
                    (t!("homepage.admin_password", locale = locale))
                    input name="admin_password"
                          type="password"
                          autocomplete="off"
                          placeholder=(t!("homepage.admin_password", locale = locale))
                          aria-describedby="password-helper"
                          aria-label=(t!("homepage.admin_password", locale = locale));
                    small id="password-helper" { (t!("homepage.password_hint", locale = locale)) }
                }
                input type="submit" value=(t!("homepage.get_started", locale = locale));
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
                 hx-disabled-elt="find input[type='password'], find input[type='submit']"
                 hx-swap="innerHTML" {
                @match state {
                    LoginState::NoError => {
                        label {
                            (t!("homepage.admin_password", locale = locale))
                            input name="admin_password"
                                  type="password"
                                  autocomplete="off"
                                  placeholder=(t!("homepage.admin_password", locale = locale))
                                  aria-describedby="password-helper"
                                  aria-label=(t!("homepage.admin_password", locale = locale));
                            small id="password-helper" {
                                (t!("homepage.password_hint_login", locale = locale))
                            }
                        }
                    },
                    LoginState::IncorrectPassword => {
                        label {
                            (t!("homepage.admin_password", locale = locale))
                            input name="admin_password"
                                  type="password"
                                  autocomplete="off"
                                  placeholder=(t!("homepage.admin_password", locale = locale))
                                  aria-describedby="password-helper"
                                  aria-invalid="true"
                                  aria-label=(t!("homepage.admin_password", locale = locale));
                            small id="password-helper" { (t!("homepage.incorrect_password", locale = locale)) }
                        }
                    }
                }
                input type="submit" value=(t!("homepage.log_in", locale = locale));
            }
        }
    }
}

pub fn dashboard(quizzes: Vec<Quiz>, locale: &str) -> Markup {
    html! {
        h1 { (t!("homepage.dashboard", locale = locale)) }

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
