use crate::{db::QuestionModel, names};
use maud::{html, Markup};
use rust_i18n::t;

pub struct QuestionData {
    pub quiz_name: String,
    pub question: QuestionModel,
    pub question_idx: i32,
    pub questions_count: i32,
    pub is_answered: bool,
    pub selected_answers: Vec<i32>,
    pub is_resuming: bool,
    pub session_id: i32,
    pub question_id: i32,
    pub is_bookmarked: bool,
    pub quiz_id: String,
}

pub struct AnswerData {
    pub quiz_name: String,
    pub question: QuestionModel,
    pub question_idx: i32,
    pub questions_count: i32,
    pub session_id: i32,
    pub quiz_id: String,
    pub selected: Vec<i32>,
    pub from_context: Option<String>,
    pub current_idx: Option<i32>,
    pub question_id: i32,
    pub is_bookmarked: bool,
}

pub fn bookmark_button(
    session_id: i32,
    question_id: i32,
    is_bookmarked: bool,
    locale: &str,
) -> Markup {
    let title = if is_bookmarked {
        t!("quiz.unbookmark", locale = locale).to_string()
    } else {
        t!("quiz.bookmark", locale = locale).to_string()
    };
    let class = if is_bookmarked {
        "bookmark-btn active"
    } else {
        "bookmark-btn"
    };
    html! {
        button type="button" class=(class)
               hx-post=(format!("/toggle-bookmark/{session_id}/{question_id}"))
               hx-swap="outerHTML"
               title=(title) {
            "\u{1F516}"
        }
    }
}

pub fn question(data: QuestionData, locale: &str) -> Markup {
    html! {
        p { (t!("quiz.doing_quiz", locale = locale)) mark { (data.quiz_name) } "." }
        article style="width: fit-content;" {
            div style="display: flex; align-items: center; margin-bottom: 0.5rem;" {
                p style="color: #666; font-size: 0.9rem; margin-bottom: 0;" {
                    (t!("quiz.question_prefix", locale = locale))
                    strong { (data.question_idx + 1) }
                    (t!("quiz.question_of", locale = locale))
                    (data.questions_count)
                }
                span style="margin-left: auto;" {
                    (bookmark_button(data.session_id, data.question_id, data.is_bookmarked, locale))
                }
            }

            @if data.is_resuming {
                p style="color: #28a745; font-weight: 500; background-color: #d4edda; padding: 0.5rem; border-radius: 4px;" {
                    (t!("quiz.resuming", locale = locale))
                }
            }

            h3 { (data.question.question) }

            @if data.question.is_multiple_choice {
                p style="color: #0066cc; font-weight: 500;" { (t!("quiz.multiple_choice", locale = locale)) }
            }

            form hx-post=(names::SUBMIT_ANSWER_URL)
                 hx-target="main"
                 hx-swap="innerHTML"
                 id="question-form" {
                fieldset {
                    @for opt in data.question.options {
                        label {
                            @if data.question.is_multiple_choice {
                                @if data.selected_answers.contains(&opt.id) {
                                    input type="checkbox" name="options" value=(opt.id) onchange="enableNextButton()" checked;
                                } @else {
                                    input type="checkbox" name="options" value=(opt.id) onchange="enableNextButton()";
                                }
                            } @else {
                                @if data.selected_answers.contains(&opt.id) {
                                    input type="radio" name="option" value=(opt.id) onchange="enableNextButton()" checked;
                                } @else {
                                    input type="radio" name="option" value=(opt.id) onchange="enableNextButton()";
                                }
                            }
                            (opt.option)
                        }
                    }
                }
                div style="display: flex; gap: 1rem; margin-top: 1rem; align-items: center;" {
                    @if data.question_idx > 0 {
                        button type="button" class="nav-btn nav-btn-back"
                               hx-get=(format!("/question/{}?question_idx={}", data.session_id, data.question_idx - 1))
                               hx-target="main"
                               hx-swap="innerHTML" {
                            (t!("quiz.previous", locale = locale))
                        }
                    }
                    span style="margin-left: auto;" {
                        input type="submit" id="submit-btn" class="nav-btn" value=(t!("quiz.submit_answer", locale = locale)) disabled[!data.is_answered];
                    }
                }
            }
            script {
                "function enableNextButton() { document.getElementById('submit-btn').disabled = false; }"
            }
        }
        p style="margin-top: 0.5rem; font-size: 0.8rem;" {
            a onclick="document.getElementById('abandon-dialog').showModal()"
              style="color: #888; text-decoration: underline; cursor: pointer;" {
                (t!("quiz.abandon", locale = locale))
            }
        }
        dialog id="abandon-dialog" {
            article {
                p { (t!("quiz.abandon_confirm", locale = locale)) }
                footer style="display: flex; gap: 0.5rem; justify-content: flex-end;" {
                    button onclick="document.getElementById('abandon-dialog').close()"
                           class="secondary" {
                        (t!("quiz.abandon_cancel", locale = locale))
                    }
                    button hx-get=(names::abandon_quiz_url(&data.quiz_id))
                           hx-target="main" {
                        (t!("quiz.abandon", locale = locale))
                    }
                }
            }
        }
    }
}

pub fn answer(data: AnswerData, locale: &str) -> Markup {
    let is_final = data.question_idx + 1 == data.questions_count;

    html! {
        p { (t!("quiz.doing_quiz", locale = locale)) mark { (data.quiz_name) } "." }
        article style="width: fit-content;" {
            div style="display: flex; align-items: center; margin-bottom: 0.5rem;" {
                p style="color: #666; font-size: 0.9rem; margin-bottom: 0;" {
                    (t!("quiz.question_prefix", locale = locale))
                    strong { (data.question_idx + 1) }
                    (t!("quiz.question_of", locale = locale))
                    (data.questions_count)
                }
                span style="margin-left: auto;" {
                    (bookmark_button(data.session_id, data.question_id, data.is_bookmarked, locale))
                }
            }
            h3 { (data.question.question) }

            form {
                fieldset disabled="true" {
                    @for opt in data.question.options {
                        @let is_selected = data.selected.contains(&opt.id);
                        @let css_class = if opt.is_answer {
                            "option-correct"
                        } else if is_selected {
                            "option-incorrect"
                        } else {
                            "option-neutral"
                        };

                        div class=(css_class) {
                            label {
                                @if data.question.is_multiple_choice {
                                    @if is_selected {
                                        input type="checkbox" name="options[]" value=(opt.id) checked;
                                    } @else {
                                        input type="checkbox" name="options[]" value=(opt.id);
                                    }
                                } @else {
                                    @if is_selected {
                                        input type="radio" name="option" value=(opt.id) checked;
                                    } @else {
                                        input type="radio" name="option" value=(opt.id);
                                    }
                                }
                                (opt.option)
                                @if opt.is_answer {
                                    span class="badge-correct" { (t!("quiz.correct", locale = locale)) }
                                } @else if is_selected {
                                    span class="badge-incorrect" { (t!("quiz.incorrect", locale = locale)) }
                                }
                            }
                            @if let Some(explanation) = &opt.explanation {
                                div class="explanation" {
                                    (explanation)
                                }
                            }
                        }
                    }
                }
            }

            @if data.from_context.as_deref() == Some("report") {
                div style="display: flex; gap: 1rem; margin-top: 1rem; align-items: center;" {
                    button class="nav-btn nav-btn-back"
                           hx-get=(names::results_url(data.session_id))
                           hx-push-url="true"
                           hx-target="main"
                           {
                        (t!("quiz.back_to_results", locale = locale))
                    }
                    @if let Some(current) = data.current_idx {
                        span style="margin-left: auto;" {
                            button class="nav-btn nav-btn-next"
                                   hx-get=(format!("/question/{}?question_idx={}", data.session_id, current))
                                   hx-push-url="true"
                                   hx-target="main"
                                   {
                                (t!("quiz.return_to_current", locale = locale))
                            }
                        }
                    }
                }
            } @else {
                div style="display: flex; gap: 1rem; margin-top: 1rem; align-items: center;" {
                    @if data.question_idx > 0 {
                        button type="button" class="nav-btn nav-btn-back"
                               hx-get=(format!("/question/{}?question_idx={}", data.session_id, data.question_idx - 1))
                               hx-target="main"
                               hx-swap="innerHTML" {
                            (t!("quiz.previous", locale = locale))
                        }
                    }
                    span style="margin-left: auto;" {
                        @if is_final {
                            button class="nav-btn nav-btn-next"
                                   hx-get=(names::results_url(data.session_id))
                                   hx-push-url="true"
                                   hx-target="main" { (t!("quiz.see_results", locale = locale)) }
                        } @else {
                            button class="nav-btn nav-btn-next"
                                   hx-get=(names::quiz_page_url(&data.quiz_id))
                                   hx-target="main" { (t!("quiz.next", locale = locale)) }
                        }
                    }
                }
            }
        }
        p style="margin-top: 0.5rem; font-size: 0.8rem;" {
            a onclick="document.getElementById('abandon-dialog').showModal()"
              style="color: #888; text-decoration: underline; cursor: pointer;" {
                (t!("quiz.abandon", locale = locale))
            }
        }
        dialog id="abandon-dialog" {
            article {
                p { (t!("quiz.abandon_confirm", locale = locale)) }
                footer style="display: flex; gap: 0.5rem; justify-content: flex-end;" {
                    button onclick="document.getElementById('abandon-dialog').close()"
                           class="secondary" {
                        (t!("quiz.abandon_cancel", locale = locale))
                    }
                    button hx-get=(names::abandon_quiz_url(&data.quiz_id))
                           hx-target="main" {
                        (t!("quiz.abandon", locale = locale))
                    }
                }
            }
        }
    }
}
