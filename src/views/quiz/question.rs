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
}

pub struct AnswerData {
    pub quiz_name: String,
    pub question: QuestionModel,
    pub question_idx: i32,
    pub questions_count: i32,
    pub session_id: i32,
    pub quiz_id: i32,
    pub selected: Vec<i32>,
    pub from_context: Option<String>,
    pub current_idx: Option<i32>,
}

pub fn question(data: QuestionData, locale: &str) -> Markup {
    html! {
        p { (t!("quiz.doing_quiz", locale = locale)) mark { (data.quiz_name) } "." }
        article style="width: fit-content;" {
            p style="color: #666; font-size: 0.9rem; margin-bottom: 0.5rem;" {
                (t!("quiz.question_prefix", locale = locale))
                strong { (data.question_idx + 1) }
                (t!("quiz.question_of", locale = locale))
                (data.questions_count)
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
                    input type="submit" id="submit-btn" class="nav-btn" value=(t!("quiz.submit_answer", locale = locale)) disabled[!data.is_answered];
                }
            }
            script {
                "function enableNextButton() { document.getElementById('submit-btn').disabled = false; }"
            }
        }
    }
}

pub fn answer(data: AnswerData, locale: &str) -> Markup {
    let is_final = data.question_idx + 1 == data.questions_count;

    html! {
        p { (t!("quiz.doing_quiz", locale = locale)) mark { (data.quiz_name) } "." }
        article style="width: fit-content;" {
            p style="color: #666; font-size: 0.9rem; margin-bottom: 0.5rem;" {
                (t!("quiz.question_prefix", locale = locale))
                strong { (data.question_idx + 1) }
                (t!("quiz.question_of", locale = locale))
                (data.questions_count)
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
                           hx-disabled-elt="this" {
                        (t!("quiz.back_to_results", locale = locale))
                    }
                    @if let Some(current) = data.current_idx {
                        button class="nav-btn nav-btn-next"
                               hx-get=(format!("/question/{}?question_idx={}", data.session_id, current))
                               hx-push-url="true"
                               hx-target="main"
                               hx-disabled-elt="this" {
                            (t!("quiz.return_to_current", locale = locale))
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
                    @if is_final {
                        button class="nav-btn nav-btn-next"
                               hx-get=(names::results_url(data.session_id))
                               hx-push-url="true"
                               hx-target="main" hx-disabled-elt="this" { (t!("quiz.see_results", locale = locale)) }
                    } @else {
                        button class="nav-btn nav-btn-next"
                               hx-get=(names::quiz_page_url(data.quiz_id))
                               hx-target="main" hx-disabled-elt="this" { (t!("quiz.next", locale = locale)) }
                    }
                }
            }
        }
    }
}
