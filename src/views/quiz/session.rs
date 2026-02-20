use crate::names;
use maud::{html, Markup, PreEscaped};
use rust_i18n::t;

pub struct StartPageData {
    pub quiz_name: String,
    pub total_questions: i32,
    pub quiz_id: i32,
}

pub fn session_name_error_page(session_name: &str, quiz_id: i32, locale: &str) -> Markup {
    html! {
        article style="margin-top: 2rem;" {
            header {
                h2 { "\u{274C} " (t!("quiz.session_error_title", locale = locale)) }
            }
            p style="color: #d32f2f; font-weight: 500;" {
                (t!("quiz.session_error_msg_1", locale = locale))
                strong { (session_name) }
                (t!("quiz.session_error_msg_2", locale = locale))
            }
            p {
                (t!("quiz.session_error_hint", locale = locale))
            }
            hr;
            h3 { (t!("quiz.suggestions", locale = locale)) }
            ul {
                li { (t!("quiz.suggestion_date", locale = locale)) code { (session_name) "_2026_02_16" } }
                li { (t!("quiz.suggestion_number", locale = locale)) code { (session_name) "_2" } }
                li { (t!("quiz.suggestion_suffix", locale = locale)) code { (session_name) "_retry" } }
            }
            hr;
            button hx-get=(names::quiz_page_url(quiz_id))
                   hx-push-url="true"
                   hx-target="main"
                   style="width: fit-content; background-color: #007bff; color: white; margin-top: 1rem;" {
                (t!("quiz.try_again", locale = locale))
            }
        }
    }
}

pub fn start_page(data: StartPageData, locale: &str) -> Markup {
    html! {
        h1 { (t!("quiz.welcome", locale = locale)) }
        p {
            (t!("quiz.doing_quiz_intro_1", locale = locale))
            mark { (data.quiz_name) }
            (t!("quiz.doing_quiz_intro_2", locale = locale))
            (data.total_questions)
            (t!("quiz.doing_quiz_intro_3", locale = locale))
        }
        article style="width: fit-content;" {
            form hx-post=(names::start_session_url(data.quiz_id))
                 hx-ext="json-enc"
                 hx-target="main"
                 hx-swap="innerHTML" {
                label {
                    (t!("quiz.session_name", locale = locale))
                    input name="name"
                          id="session-name"
                          type="text"
                          autocomplete="off"
                          aria-describedby="name-helper"
                          aria-label=(t!("quiz.session_name", locale = locale))
                          pattern="[a-zA-Z0-9_\\-]+"
                          title=(t!("quiz.session_name_pattern_title", locale = locale))
                          required;
                    small id="name-helper" style="display: block; margin-top: 0.5rem; color: #666;" {
                        (t!("quiz.session_name_hint", locale = locale))
                    }
                }
                script {
                    (PreEscaped("(function(){var d=new Date(),y=d.getFullYear(),m=String(d.getMonth()+1).padStart(2,'0'),dd=String(d.getDate()).padStart(2,'0'),r=Math.random().toString(36).substring(2,8);document.getElementById('session-name').value=y+'-'+m+'-'+dd+'-'+r;})();"))
                }
                label {
                    (t!("quiz.question_count", locale = locale))
                    input name="question_count"
                          type="number"
                          min=(names::MIN_QUESTION_COUNT)
                          max=(names::MAX_QUESTION_COUNT)
                          value=(names::DEFAULT_QUESTION_COUNT)
                          aria-label=(t!("quiz.question_count", locale = locale))
                          required;
                    small style="display: block; margin-top: 0.5rem; color: #666;" {
                        (t!("quiz.question_count_hint",
                            min = names::MIN_QUESTION_COUNT,
                            max = names::MAX_QUESTION_COUNT,
                            default = names::DEFAULT_QUESTION_COUNT,
                            locale = locale))
                    }
                }
                fieldset {
                    legend { (t!("quiz.selection_mode", locale = locale)) }
                    label {
                        input type="radio" name="selection_mode" value="unanswered" checked;
                        (t!("quiz.mode_unanswered", locale = locale))
                    }
                    label {
                        input type="radio" name="selection_mode" value="incorrect";
                        (t!("quiz.mode_incorrect", locale = locale))
                    }
                    label {
                        input type="radio" name="selection_mode" value="random";
                        (t!("quiz.mode_random", locale = locale))
                    }
                }
                input type="submit" value=(t!("quiz.start", locale = locale));
            }
        }
    }
}
