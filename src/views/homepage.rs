use maud::{html, Markup};
use crate::{db::Quiz, names};

pub fn get_started() -> Markup {
    html! {
        h1 { "Welcome to Quizzy!" }
        p {
            "Seems like this is the first time you are using "
            mark { "Quizzy" }
            " for the first time. You will need to set an "
            strong { "admin password" }
            " to get started."
        }
        article style="width: fit-content;" {
            form hx-post=(names::GET_STARTED_URL)
                 hx-ext="json-enc"
                 hx-target="main"
                 hx-disabled-elt="find input[type='password'], find input[type='submit']"
                 hx-swap="innerHTML" {
                label {
                    "Admin Password"
                    input name="admin_password"
                          type="password"
                          autocomplete="off"
                          placeholder="Admin Password"
                          aria-describedby="password-helper"
                          aria-label="Your Password";
                    small id="password-helper" { "Be sure not to forget the password." }
                }
                input type="submit" value="Get Started";
            }
        }
    }
}

pub enum LoginState {
    NoError,
    IncorrectPassword,
}

pub fn login(state: LoginState) -> Markup {
    html! {
        h1 { "Welcome back to Quizzy!" }
        p {
            "Use the admin password you previously set to log back in to your dashboard."
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
                            "Admin Password"
                            input name="admin_password"
                                  type="password"
                                  autocomplete="off"
                                  placeholder="Admin Password"
                                  aria-describedby="password-helper"
                                  aria-label="Your Password";
                            small id="password-helper" {
                                "Use the admin password you set when you first used Quizzy."
                            }
                        }
                    },
                    LoginState::IncorrectPassword => {
                        label {
                            "Admin Password"
                            input name="admin_password"
                                  type="password"
                                  autocomplete="off"
                                  placeholder="Admin Password"
                                  aria-describedby="password-helper"
                                  aria-invalid="true"
                                  aria-label="Your Password";
                            small id="password-helper" { "Incorrect password" }
                        }
                    }
                }
                input type="submit" value="Log In";
            }
        }
    }
}

pub fn dashboard(quizzes: Vec<Quiz>) -> Markup {
    html! {
        h1 { "Dashboard" }

        article style="width: fit-content;" {
            form hx-post=(names::CREATE_QUIZ_URL)
                 hx-target="main"
                 enctype="multipart/form-data"
                 hx-disabled-elt="find input[type='text'], find input[type='file'], find input[type='submit']"
                 hx-swap="innerHTML" {
                    label {
                        "Quiz Name"
                        input name="quiz_name"
                              type="text"
                              required="true"
                              autocomplete="off"
                              placeholder="Quiz Name"
                              aria-describedby="quiz-name-helper"
                              aria-label="Your Quiz Name";
                        small id="quiz-name-helper" { "What do you want to call this quiz?" }
                    }

                    label {
                        "Quiz File"
                        input name="quiz_file"
                              type="file"
                              required="true"
                              aria-describedby="quiz-file-helper"
                              accept="application/json"
                              aria-label="Your Quiz File";
                        small id="quiz-file-helper" { "The JSON file that includes the questions in this quiz." }
                    }

                    input type="submit" value="Create";
            }
        }

        div."quiz-grid" {
            @for quiz in quizzes {
                article {
                    h3 { (quiz.name) }
                    p { (quiz.count) " questions." }
                    div role="group" {
                        button
                            hx-trigger="click"
                            hx-target="main"
                            hx-swap="innerHTML"
                            hx-push-url="true"
                            hx-get=(names::quiz_dashboard_url(quiz.id)) { "View" }
                        button."contrast"
                            hx-disabled-elt="this"
                            hx-trigger="click"
                            hx-target="closest article"
                            hx-swap="outerHTML"
                            hx-confirm="Are you sure you want to delete this Quiz?"
                            hx-delete=(names::delete_quiz_url(quiz.id)) { "Delete" }
                    }
                }
            }
        }
    }
}
