use crate::{db::models::SharedQuizInfo, names};
use maud::{html, Markup};
use rust_i18n::t;

pub fn marketplace_page(quizzes: &[SharedQuizInfo], user_quiz_ids: &[i32], locale: &str) -> Markup {
    html! {
        h1 { (t!("marketplace.title", locale = locale)) }
        @if quizzes.is_empty() {
            p { (t!("marketplace.empty", locale = locale)) }
        } @else {
            div class="quiz-grid" id="quiz-results" {
                (quiz_cards(quizzes, user_quiz_ids, locale))
            }
        }
    }
}

pub fn quiz_cards(quizzes: &[SharedQuizInfo], user_quiz_ids: &[i32], locale: &str) -> Markup {
    html! {
        @for quiz in quizzes {
            @let imported = user_quiz_ids.contains(&quiz.id);
            article class="quiz-card" {
                h3 { (quiz.name) }
                p {
                    small {
                        (t!("marketplace.by", locale = locale, owner = &quiz.owner_name))
                        " · "
                        (quiz.question_count)
                        (t!("share.questions_suffix", locale = locale))
                    }
                }
                @if imported {
                    button
                        hx-get=(names::quiz_dashboard_url(&quiz.public_id))
                        hx-push-url="true"
                        hx-target="main"
                        hx-swap="innerHTML"
                        class="card-actions"
                        style="width: fit-content;" {
                        (t!("marketplace.go_to_dashboard", locale = locale))
                    }
                } @else {
                    button
                        hx-post=(names::add_to_library_url(&quiz.public_id))
                        hx-target="main"
                        hx-swap="innerHTML"
                        class="card-actions"
                        style="width: fit-content;" {
                        (t!("marketplace.add_to_library", locale = locale))
                    }
                }
            }
        }
    }
}
